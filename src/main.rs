use iced_layershell::{
    reexport::{Anchor, KeyboardInteractivity, Layer},
    settings::{LayerShellSettings, StartMode},
    Settings,
};

mod app;
mod daemon;
mod history;
mod theme;

fn main() -> iced_layershell::Result {
    let args: Vec<String> = std::env::args().collect();

    if args.get(1).map(|s| s.as_str()) == Some("daemon") {
        tokio::runtime::Runtime::new().unwrap().block_on(crate::daemon::run());
        Ok(())
    } else {
        let layer_settings = LayerShellSettings {
            anchor: Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right,
            layer: Layer::Overlay,
            exclusive_zone: -1,
            size: None,
            margin: (0, 0, 0, 0),
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            start_mode: StartMode::Active,
            events_transparent: false,
        };

        let settings = Settings {
            layer_settings,
            id: Some("aplec".into()),
            ..Default::default()
        };

        iced_layershell::application(app::boot, app::namespace, app::update, app::view)
            .subscription(app::subscription)
            .style(|_state: &app::Aplec, _theme: &iced::Theme| iced::theme::Style {
                background_color: iced::Color::TRANSPARENT,
                text_color: iced::Color::WHITE,
            })
            .settings(settings)
            .run()
    }
}
