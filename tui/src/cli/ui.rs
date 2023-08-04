use ratatui::{prelude::*, widgets::*};

use crate::app::{App, InputMode};

pub(crate) fn render_app_too_frame<B: Backend>(frame: &mut Frame<B>, app: &App) {
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
                Constraint::Length(10),
            ]
            .as_ref(),
        )
        .split(left) else {
            panic!("The left layout should have 2 chunks")
        };

    let room_list: Vec<ListItem> = vec!["test", "room1", "room2"]
        .iter()
        .map(|room_name| {
            let content = Line::from(Span::raw(format!("#{room_name}")));

            ListItem::new(content)
        })
        .collect();
    let room_list =
        List::new(room_list).block(Block::default().borders(Borders::ALL).title("Rooms"));
    frame.render_widget(room_list, container_room_list);

    let user_info = Paragraph::new("User: @jjohndoejohndoejohndoejohndoejohndoeohndoe").block(
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

    let text = Text::from(Line::from(vec![
        "on ".into(),
        "#room1".bold(),
        " for ".into(),
        "interesting talks about life".italic(),
    ]));
    let help_message = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Active Room Information"),
    );
    frame.render_widget(help_message, container_highlight);

    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let content = Line::from(Span::raw(format!("{i}: {m}")));
            ListItem::new(content)
        })
        .collect();
    let messages =
        List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
    frame.render_widget(messages, container_messages);

    let input = Paragraph::new(app.input.as_str())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Input"));
    frame.render_widget(input, container_input);
    match app.input_mode {
        InputMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Editing => {
            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            frame.set_cursor(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                container_input.x + app.cursor_position as u16 + 1,
                // Move one line down, from the border to the input line
                container_input.y + 1,
            )
        }
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

    let room_users_list_items: Vec<ListItem> = vec!["jjohndoe", "jane", "john"]
        .iter()
        .map(|user_name| {
            let content = Line::from(Span::raw(format!("@{user_name}")));

            ListItem::new(content)
        })
        .collect();
    let room_users_list = List::new(room_users_list_items)
        .block(Block::default().borders(Borders::ALL).title("Room Users"));

    frame.render_widget(room_users_list, container_room_users);

    let mut usage_text = Text::from(vec![
        Line::from(vec![
            "(Ctrl + C)".bold(),
            " or ".into(),
            "(q)".bold(),
            " to exit".into(),
        ]),
        Line::from(vec!["(e)".bold(), " to start editing".into()]),
    ]);
    usage_text.patch_style(Style::default().add_modifier(Modifier::RAPID_BLINK));
    let usage =
        Paragraph::new(usage_text).block(Block::default().borders(Borders::ALL).title("Usage"));
    frame.render_widget(usage, container_usage);
}
