use crate::frontend::{load_icon, App, AppInput, T};
use relm4::AsyncComponentSender;
use tracing::warn;

pub(in super::super) struct TrayIcon {
    _icon: tray_icon::TrayIcon,
    sender: AsyncComponentSender<App>,
}

impl TrayIcon {
    pub(in super::super) fn init(sender: AsyncComponentSender<App>) -> Result<Self, ()> {
        let icon_img = load_icon();

        let (width, height) = icon_img.dimensions();

        let menu_open = &tray_icon::menu::MenuItem::new(&*T.tray_icon_open(), true, None);
        let menu_close = &tray_icon::menu::MenuItem::new(&*T.tray_icon_close(), true, None);

        let menu = tray_icon::menu::Menu::with_items(&[menu_open, menu_close])
            .inspect_err(|error| {
                warn!(%error, "Unable to create tray icon menu");
            })
            .map_err(|_| ())?;

        let icon = tray_icon::TrayIconBuilder::new()
            .with_tooltip("Space Acres")
            .with_icon(
                tray_icon::Icon::from_rgba(icon_img.clone().into_raw().to_vec(), width, height)
                    .expect("Statically correct image; qed"),
            )
            .with_menu(std::boxed::Box::new(menu))
            .build()
            .map_err(|error| {
                warn!(%error, "Unable to create tray icon");
            })?;

        let menu_event = tray_icon::menu::MenuEvent::receiver();
        let menu_close_id = menu_close.id().clone();
        let menu_open_id = menu_open.id().clone();

        let tray_icon = Self {
            _icon: icon,
            sender,
        };
        let sender = tray_icon.sender.clone();

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
