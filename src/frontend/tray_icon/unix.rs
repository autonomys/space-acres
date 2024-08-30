use crate::frontend::tray_icon::load_icon;
use crate::frontend::{App, AppInput, T};
use ksni::menu::{MenuItem, StandardItem};
use ksni::{Icon, ToolTip, Tray, TrayService};
use relm4::AsyncComponentSender;

#[derive(Clone)]
pub(in super::super) struct TrayIcon {
    sender: AsyncComponentSender<App>,
}

impl TrayIcon {
    pub(in super::super) fn new(sender: AsyncComponentSender<App>) -> Result<Self, ()> {
        let icon = Self { sender };

        let tray_service = TrayService::new(icon.clone());

        tray_service.spawn();

        Ok(icon)
    }
}

impl Tray for TrayIcon {
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").to_string()
    }

    fn icon_name(&self) -> String {
        "space-acres".to_string()
    }

    fn title(&self) -> String {
        "Space Acres".to_string()
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        let icon_img = load_icon();
        let width = icon_img.width() as i32;
        let height = icon_img.height() as i32;

        vec![Icon {
            width,
            height,
            data: icon_img.into_raw(),
        }]
    }

    fn tool_tip(&self) -> ToolTip {
        ToolTip {
            title: "Space Acres".to_string(),
            ..Default::default()
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        vec![
            StandardItem {
                label: T.tray_icon_open().to_string(),
                activate: Box::new(|this: &mut Self| {
                    this.sender.input(AppInput::ShowWindow);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: T.tray_icon_close().to_string(),
                activate: Box::new(|this: &mut Self| {
                    this.sender.input(AppInput::HideWindow);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}
