use crate::frontend::tray_icon::load_icon;
use crate::frontend::{App, AppCommandOutput, T};
use relm4::AsyncComponentSender;
use std::any::Any;
use std::error::Error;
use tracing::warn;
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

pub(in super::super) async fn spawn(sender: &AsyncComponentSender<App>) -> Option<Box<dyn Any>> {
    let init_result: Result<TrayIcon, Box<dyn Error>> = try {
        let icon_img = load_icon();
        let width = icon_img.width();
        let height = icon_img.height();
        let icon = tray_icon::Icon::from_rgba(icon_img.into_raw(), width, height)
            .expect("Statically correct image; qed");

        let menu_open = &MenuItem::new(&*T.tray_icon_open(), true, None);
        let menu_open_id = menu_open.id().clone();
        let menu_close = &MenuItem::new(&*T.tray_icon_close(), true, None);
        let menu_close_id = menu_close.id().clone();

        let menu = Menu::with_items(&[menu_open, menu_close])
            .map_err(|error| format!("Failed to create tray icon menu: {error}"))?;

        let icon = TrayIconBuilder::new()
            .with_tooltip("Space Acres")
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .build()
            .map_err(|error| format!("Failed to create tray icon: {error}"))?;

        let menu_events = MenuEvent::receiver();
        sender.spawn_command(move |sender| {
            while let Ok(event) = menu_events.recv() {
                let output = if event.id == menu_open_id {
                    AppCommandOutput::ShowWindow
                } else if event.id == menu_close_id {
                    AppCommandOutput::Quit
                } else {
                    continue;
                };

                if let Err(error) = sender.send(output) {
                    warn!(?error, "Failed to send tray icon notification");
                    break;
                }
            }
        });

        let tray_icon_events = TrayIconEvent::receiver();
        sender.spawn_command(move |sender| {
            while let Ok(event) = tray_icon_events.recv() {
                let output = if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event
                {
                    AppCommandOutput::ShowHideToggle
                } else {
                    continue;
                };

                if let Err(error) = sender.send(output) {
                    warn!(?error, "Failed to send tray icon notification");
                    break;
                }
            }
        });

        icon
    };

    match init_result {
        Ok(tray_icon) => Some(Box::new(tray_icon)),
        Err(error) => {
            warn!(%error, "Tray icon error");
            None
        }
    }
}
