use crate::frontend::tray_icon::load_icon;
use crate::frontend::{App, AppInput, T};
use relm4::AsyncComponentSender;
use tracing::warn;
use tray_icon::menu::{Menu, MenuItem};
use tray_icon::TrayIconBuilder;

pub(in super::super) struct TrayIcon {
    _icon: tray_icon::TrayIcon,
}

impl TrayIcon {
    pub(in super::super) fn new(sender: AsyncComponentSender<App>) -> Result<Self, ()> {
        let icon_img = load_icon();
        let width = icon_img.width();
        let height = icon_img.height();
        let icon = tray_icon::Icon::from_rgba(icon_img.into_raw(), width, height)
            .expect("Statically correct image; qed");

        let menu_open = &MenuItem::new(&*T.tray_icon_open(), true, None);
        let menu_close = &MenuItem::new(&*T.tray_icon_close(), true, None);

        let menu = Menu::with_items(&[menu_open, menu_close])
            .inspect_err(|error| {
                warn!(%error, "Unable to create tray icon menu");
            })
            .map_err(|_| ())?;

        let icon = TrayIconBuilder::new()
            .with_tooltip("Space Acres")
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .build()
            .map_err(|error| {
                warn!(%error, "Unable to create tray icon");
            })?;

        let menu_event = tray_icon::menu::MenuEvent::receiver();
        let menu_close_id = menu_close.id().clone();
        let menu_open_id = menu_open.id().clone();

        let tray_icon = Self { _icon: icon };

        sender.clone().spawn_command(move |_sender| {
            while let Ok(event) = menu_event.recv() {
                let input = if event.id == menu_open_id {
                    Some(AppInput::ShowWindow)
                } else if event.id == menu_close_id {
                    Some(AppInput::HideWindow)
                } else {
                    None
                };

                if let Some(input) = input {
                    sender.input(input);
                }
            }
        });

        Ok(tray_icon)
    }
}
