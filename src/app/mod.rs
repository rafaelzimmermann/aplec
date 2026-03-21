use iced::{event, event::Status, keyboard::{self, key::Named}, widget::{column, container, text}, Alignment, Background, Border, Element, Event, Length, Padding, Subscription, Task};
use iced_layershell::to_layer_message;
use crate::history::ClipEntry;
use crate::theme::{Placement, Theme};

const POPUP_W: f32 = 520.0;

pub struct Aplec {
    pub all: Vec<ClipEntry>,
    pub index: usize,
    pub theme: Theme,
}

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    Close,
    Activate(usize),
    IcedEvent(Event, Status),
}


pub fn boot() -> (Aplec, Task<Message>) {
    let all = crate::history::load();
    let state = Aplec { all, index: 0, theme: Theme::load() };
    (state, Task::none())
}

pub fn namespace() -> String {
    "aplec".to_string()
}

pub fn update(state: &mut Aplec, msg: Message) -> Task<Message> {
    match msg {
        Message::Close => std::process::exit(0),
        Message::Activate(idx) => {
            if let Some(entry) = state.all.get(idx) {
                use std::io::Write;
                let content = entry.content.clone();
                if let Ok(mut child) = std::process::Command::new("wl-copy")
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                {
                    if let Some(mut stdin) = child.stdin.take() {
                        let _ = stdin.write_all(content.as_bytes());
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                std::process::exit(0);
            }
            Task::none()
        }
        Message::IcedEvent(event, status) => {
            if status == Status::Ignored {
                match event {
                    Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers: _, .. }) => {
                        match key.as_ref() {
                            keyboard::Key::Named(Named::Escape) => {
                                return Task::done(Message::Close);
                            }
                            keyboard::Key::Named(Named::ArrowLeft) => {
                                state.index = state.index.saturating_sub(1);
                            }
                            keyboard::Key::Named(Named::ArrowRight) => {
                                state.index = (state.index + 1).min(state.all.len().saturating_sub(1));
                            }
                            keyboard::Key::Named(Named::Enter) => {
                                return Task::done(Message::Activate(state.index));
                            }
                            _ => {}
                        }
                    }
                    Event::Mouse(iced::mouse::Event::ButtonPressed(_)) => {
                        return Task::done(Message::Close);
                    }
                    _ => {}
                }
            }
            Task::none()
        }
        _ => Task::none(),
    }
}

pub fn view(state: &Aplec) -> Element<'_, Message> {
    let card = if state.all.is_empty() {
        column![text("No clipboard history")]
            .width(Length::Fill)
            .align_x(iced::Alignment::Center)
    } else {
        let counter = text(format!("[{} / {}]", state.index + 1, state.all.len()))
            .size(12)
            .color(state.theme.text_dim);

        let content = text(&state.all[state.index].content)
            .size(14)
            .color(state.theme.text);

        let hint = text("← →  Enter to paste   Esc to close")
            .size(11)
            .color(state.theme.text_dim);

        column![counter, content, hint]
            .width(Length::Fill)
            .spacing(12)
            .padding(20)
    };

    let inner = container(card)
        .width(Length::Fixed(POPUP_W))
        .style(move |_| iced::widget::container::Style {
            background: Some(Background::Color(state.theme.background)),
            border: Border { color: state.theme.border, width: 1.0, radius: 8.0.into() },
            text_color: None,
            shadow: iced::Shadow::default(),
            snap: false,
        });

    let m = state.theme.margin.max(0) as f32;
    let (ax, ay, padding) = match state.theme.placement {
        Placement::Center       => (Alignment::Center, Alignment::Center, Padding::ZERO),
        Placement::TopLeft      => (Alignment::Start,  Alignment::Start,  Padding { top: m, left: m,  ..Padding::ZERO }),
        Placement::TopCenter    => (Alignment::Center, Alignment::Start,  Padding { top: m,            ..Padding::ZERO }),
        Placement::TopRight     => (Alignment::End,    Alignment::Start,  Padding { top: m, right: m, ..Padding::ZERO }),
        Placement::BottomLeft   => (Alignment::Start,  Alignment::End,    Padding { bottom: m, left: m,  ..Padding::ZERO }),
        Placement::BottomCenter => (Alignment::Center, Alignment::End,    Padding { bottom: m,            ..Padding::ZERO }),
        Placement::BottomRight  => (Alignment::End,    Alignment::End,    Padding { bottom: m, right: m, ..Padding::ZERO }),
    };

    container(inner)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(padding)
        .align_x(ax)
        .align_y(ay)
        .into()
}

pub fn subscription(_: &Aplec) -> Subscription<Message> {
    event::listen_with(|event, status, _id| Some(Message::IcedEvent(event, status)))
}

#[cfg(test)]
mod tests {
    #[test]
    fn nav_right_increments_index() {
        let len = 5usize;
        let mut index = 2usize;
        index = (index + 1).min(len.saturating_sub(1));
        assert_eq!(index, 3);
    }

    #[test]
    fn nav_right_clamps_at_end() {
        let len = 5usize;
        let mut index = 4usize;
        index = (index + 1).min(len.saturating_sub(1));
        assert_eq!(index, 4);
    }

    #[test]
    fn nav_left_decrements_index() {
        let mut index = 3usize;
        index = index.saturating_sub(1);
        assert_eq!(index, 2);
    }

    #[test]
    fn nav_left_clamps_at_zero() {
        let mut index = 0usize;
        index = index.saturating_sub(1);
        assert_eq!(index, 0);
    }

    #[test]
    fn nav_right_on_empty_history_is_zero() {
        let len = 0usize;
        let mut index = 0usize;
        index = (index + 1).min(len.saturating_sub(1));
        assert_eq!(index, 0);
    }
}
