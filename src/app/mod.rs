use iced::{event, event::Status, keyboard::{self, key::Named}, widget::{column, container, row, text, mouse_area, text_input}, Background, Border, Element, Event, Length, Subscription, Task};
use iced_layershell::to_layer_message;
use crate::history::ClipEntry;
use crate::theme::Theme;

const VIEW_SIZE: usize = 8;
const POPUP_W: f32 = 520.0;

pub struct Aplec {
    pub all: Vec<ClipEntry>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub view_start: usize,
    pub query: String,
    pub theme: Theme,
}

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    Close,
    Absorb,
    Activate(usize),
    IcedEvent(Event, Status),
    QueryChanged(String),
    QuerySubmit,
}

fn refilter(state: &mut Aplec) {
    state.filtered.clear();
    if state.query.is_empty() {
        for i in 0..state.all.len() {
            state.filtered.push(i);
        }
    } else {
        let q = state.query.to_lowercase();
        for (i, entry) in state.all.iter().enumerate() {
            if entry.content.to_lowercase().contains(&q) {
                state.filtered.push(i);
            }
        }
    }
    state.selected = 0;
    state.view_start = 0;
}

async fn do_copy(content: String) {
    use tokio::io::AsyncWriteExt;
    if let Ok(mut child) = tokio::process::Command::new("wl-copy")
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(content.as_bytes()).await;
        }
        let _ = child.wait().await;
    }
}

pub fn boot() -> (Aplec, Task<Message>) {
    let all = crate::history::load();
    let mut filtered = Vec::new();
    for i in 0..all.len() {
        filtered.push(i);
    }
    let state = Aplec {
        all,
        filtered,
        selected: 0,
        view_start: 0,
        query: String::new(),
        theme: Theme::load(),
    };
    (state, Task::none())
}

pub fn namespace() -> String {
    "aplec".to_string()
}

pub fn update(state: &mut Aplec, msg: Message) -> Task<Message> {
    match msg {
        Message::Close => Task::none(),
        Message::Absorb => Task::none(),
        Message::Activate(idx) => {
            if let Some(&entry_idx) = state.filtered.get(idx) {
                if let Some(entry) = state.all.get(entry_idx) {
                    let content = entry.content.clone();
                    return Task::perform(do_copy(content), |_| Message::Close);
                }
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
                            keyboard::Key::Named(Named::ArrowDown) => {
                                if !state.filtered.is_empty() {
                                    state.selected = (state.selected + 1).min(state.filtered.len() - 1);
                                    if state.selected >= state.view_start + VIEW_SIZE {
                                        state.view_start = state.selected - VIEW_SIZE + 1;
                                    }
                                }
                            }
                            keyboard::Key::Named(Named::ArrowUp) => {
                                if !state.filtered.is_empty() && state.selected > 0 {
                                    state.selected -= 1;
                                    if state.selected < state.view_start {
                                        state.view_start = state.selected;
                                    }
                                }
                            }
                            keyboard::Key::Named(Named::Enter) => {
                                return Task::done(Message::Activate(state.selected));
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            Task::none()
        }
        Message::QueryChanged(value) => {
            state.query = value;
            refilter(state);
            Task::none()
        }
        Message::QuerySubmit => {
            if !state.filtered.is_empty() {
                return Task::done(Message::Activate(state.selected));
            }
            Task::none()
        }
        _ => Task::none(),
    }
}

pub fn view(state: &Aplec) -> Element<Message> {
    let search = text_input("Search...", &state.query)
        .on_input(Message::QueryChanged)
        .on_submit(Message::QuerySubmit)
        .padding(10)
        .size(14);

    let list_items: Vec<Element<Message>> = state.filtered
        .iter()
        .skip(state.view_start)
        .take(VIEW_SIZE)
        .enumerate()
        .map(|(i, &idx)| {
            let global_idx = state.view_start + i;
            let is_selected = global_idx == state.selected;
            let entry = &state.all[idx];
            let preview = if entry.content.len() > 60 {
                format!("{}...", &entry.content[..57])
            } else {
                entry.content.clone()
            };
            
            let row = row![
                text(preview)
                    .size(13)
                    .color(if is_selected { state.theme.text } else { state.theme.text_dim })
            ]
            .padding(8)
            .spacing(8)
            .align_y(iced::Alignment::Center);

            mouse_area(row)
                .on_press(Message::Activate(global_idx))
                .into()
        })
        .collect();

    let list = column(list_items)
        .spacing(4)
        .padding(8);

    let content = column![
        search,
        list,
    ]
    .spacing(12)
    .padding(16);

    container(content)
        .width(Length::Fixed(POPUP_W))
        .style(move |_| iced::widget::container::Style {
            background: Some(Background::Color(state.theme.background)),
            border: Border { color: state.theme.border, width: 1.0, radius: 8.0.into() },
            text_color: None,
            shadow: iced::Shadow::default(),
            snap: false,
        })
        .into()
}

pub fn subscription(_: &Aplec) -> Subscription<Message> {
    event::listen_with(|event, status, _id| Some(Message::IcedEvent(event, status)))
}

