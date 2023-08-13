use ratatui::{prelude::*, widgets::*};

use crate::app::MessageBoxItem;

use super::{
    chat_page::{ChatPage, Section},
    widget_handler::WidgetUsage,
};

const NO_ROOM_SELECTED_MESSAGE: &str = "Join at least one room to start chatting!";

fn key_to_span<'a>(key: &String) -> Span<'a> {
    Span::from(format!("({})", key)).bold()
}

fn widget_usage_to_text<'a>(usage: WidgetUsage) -> Text<'a> {
    let mut lines: Vec<Line> = vec![];
    if let Some(description) = usage.description {
        lines.push(Line::from(description));
    }

    for wuk in usage.keys {
        let mut bindings: Vec<Span> = match wuk.keys.len() {
            0 => vec![],
            1 => vec![key_to_span(&wuk.keys[0])],
            2 => vec![
                key_to_span(&wuk.keys[0]),
                " or ".into(),
                key_to_span(&wuk.keys[1]),
            ],
            _ => {
                let mut bindings: Vec<Span> = Vec::with_capacity(wuk.keys.len() * 2);

                for key in wuk.keys.iter().take(wuk.keys.len() - 1) {
                    bindings.push(key_to_span(key));
                    bindings.push(", ".into());
                }

                bindings.push("or".into());
                bindings.push(key_to_span(wuk.keys.last().unwrap()));

                bindings
            }
        };

        bindings.push(Span::from(format!(" {}", wuk.description)));

        lines.push(Line::from(bindings));
    }

    Text::from(lines)
}

// TODO: move the message list to listview and make it scrollable
fn calculate_message_list_offset(height: u16, messages_len: usize) -> usize {
    // minus 2 for borders
    messages_len.saturating_sub(height as usize - 2)
}

impl ChatPage {
    fn calculate_border_color(&self, section: Section) -> Color {
        match (self.active_section.as_ref(), &self.last_hovered_section) {
            (Some(active_section), _) if active_section.eq(&section) => Color::Yellow,
            (_, last_hovered_section) if last_hovered_section.eq(&section) => Color::Blue,
            _ => Color::Reset,
        }
    }
}

pub(crate) fn render_app_too_frame<B: Backend>(frame: &mut Frame<B>, chat_page: &ChatPage) {
    let [left, middle, right] = *Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ]
            .as_ref(),
        )
        .split(frame.size()) else {
            panic!("The main layout should have 3 chunks")
        };

    let [container_room_list, container_user_info] = *Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(1),
                Constraint::Length(4),
            ]
            .as_ref(),
        )
        .split(left) else {
            panic!("The left layout should have 2 chunks")
        };

    let active_room = chat_page.active_room();
    let room_list: Vec<ListItem> = chat_page
        .room_list
        .rooms()
        .iter()
        .map(|room_state| {
            let room_tag = format!(
                "#{}{}",
                room_state.name,
                if room_state.has_unread { "*" } else { "" }
            );
            let content = Line::from(Span::raw(room_tag));

            let style = if chat_page.room_list.list_state.selected().is_none()
                && active_room.is_some()
                && active_room.as_ref().unwrap().eq(&room_state.name)
            {
                Style::default().add_modifier(Modifier::BOLD)
            } else if room_state.has_unread {
                Style::default().add_modifier(Modifier::RAPID_BLINK | Modifier::ITALIC)
            } else {
                Style::default()
            };

            ListItem::new(content).style(style.bg(Color::Reset))
        })
        .collect();

    let room_list = List::new(room_list)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().fg(chat_page.calculate_border_color(Section::RoomList)))
                .title("Rooms"),
        )
        .highlight_style(
            Style::default()
                // yellow that would work for both dark / light modes
                .bg(Color::Rgb(255, 223, 102))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">");

    let mut app_room_list_state = chat_page.room_list.list_state.clone();
    frame.render_stateful_widget(room_list, container_room_list, &mut app_room_list_state);

    let user_info = Paragraph::new(Text::from(vec![
        Line::from(format!("User: @{}", chat_page.username())),
        Line::from(format!("Chatting for: {} secs", chat_page.timer())),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("User Information"),
    );
    frame.render_widget(user_info, container_user_info);

    let [container_highlight, container_messages, container_input] = *Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(middle) else {
            panic!("The middle layout should have 3 chunks")
        };

    let top_line = if let Some(room_data) = active_room
        .as_ref()
        .and_then(|active_room| chat_page.get_room_data(active_room))
    {
        Line::from(vec![
            "on ".into(),
            Span::from(format!("#{}", room_data.name)).bold(),
            " for ".into(),
            Span::from(format!(r#""{}""#, room_data.description)).italic(),
        ])
    } else {
        Line::from(NO_ROOM_SELECTED_MESSAGE)
    };
    let text = Text::from(top_line);

    let help_message = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Active Room Information"),
    );
    frame.render_widget(help_message, container_highlight);

    let messages = if let Some(active_room) = active_room.as_ref() {
        chat_page
            .get_room_data(active_room)
            .map(|room_data| {
                let message_offset = calculate_message_list_offset(
                    container_messages.height,
                    room_data.messages.len(),
                );

                room_data.messages[message_offset..]
                    .iter()
                    .map(|mbi| {
                        let line = match mbi {
                            MessageBoxItem::Message { username, content } => {
                                Line::from(Span::raw(format!("@{}: {}", username, content)))
                            }
                            MessageBoxItem::Notification(content) => {
                                Line::from(Span::raw(content.clone()).italic())
                            }
                        };

                        ListItem::new(line)
                    })
                    .collect::<Vec<ListItem>>()
            })
            .unwrap_or_default()
    } else {
        vec![ListItem::new(Line::from(NO_ROOM_SELECTED_MESSAGE))]
    };
    let messages =
        List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
    frame.render_widget(messages, container_messages);

    let input = Paragraph::new(chat_page.input_box.text.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .fg(chat_page.calculate_border_color(Section::MessageInput))
                .title("Input"),
        );
    frame.render_widget(input, container_input);

    // Cursor is hidden by default, so we need to make it visible if the input box is selected
    if let Some(Section::MessageInput) = chat_page.active_section {
        // Make the cursor visible and ask ratatui to put it at the specified coordinates after
        // rendering
        frame.set_cursor(
            // Draw the cursor at the current position in the input field.
            // This position is can be controlled via the left and right arrow key
            container_input.x + chat_page.input_box.cursor_position as u16 + 1,
            // Move one line down, from the border to the input line
            container_input.y + 1,
        )
    }

    let [container_room_users, container_usage] = *Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(1),
                Constraint::Length(10),
            ]
            .as_ref(),
        )
        .split(right) else {
            panic!("The left layout should have 2 chunks")
        };

    let room_users_list_items: Vec<ListItem> = if let Some(active_room) = active_room.as_ref() {
        chat_page
            .get_room_data(active_room)
            .map(|room_data| {
                room_data
                    .users
                    .iter()
                    .map(|user_name| ListItem::new(Line::from(Span::raw(format!("@{user_name}")))))
                    .collect::<Vec<ListItem<'_>>>()
            })
            .unwrap_or_default()
    } else {
        vec![]
    };

    let room_users_list = List::new(room_users_list_items)
        .block(Block::default().borders(Borders::ALL).title("Room Users"));

    frame.render_widget(room_users_list, container_room_users);

    let mut usage_text: Text = widget_usage_to_text(chat_page.usage());
    usage_text.patch_style(Style::default().add_modifier(Modifier::RAPID_BLINK));
    let usage = Paragraph::new(usage_text)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title("Usage"));
    frame.render_widget(usage, container_usage);
}
