use ratatui::{
    style::Stylize,
    text::{Line, Span, Text},
};

#[derive(Debug, Clone)]
pub struct UsageInfoLine {
    pub keys: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct UsageInfo {
    pub description: Option<String>,
    pub lines: Vec<UsageInfoLine>,
}

pub trait HasUsageInfo {
    fn usage_info(&self) -> UsageInfo;
}

fn key_to_span<'a>(key: &String) -> Span<'a> {
    Span::from(format!("({})", key)).bold()
}

pub fn widget_usage_to_text<'a>(usage: UsageInfo) -> Text<'a> {
    let mut lines: Vec<Line> = vec![];
    if let Some(description) = usage.description {
        lines.push(Line::from(description));
    }

    for wuk in usage.lines {
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
