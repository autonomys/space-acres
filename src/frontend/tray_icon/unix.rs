use crate::frontend::{load_icon, App, AppInput, T};
use relm4::AsyncComponentSender;

#[derive(Clone)]
pub(in super::super) struct TrayIcon {
    sender: AsyncComponentSender<App>,
}

impl TrayIcon {
    pub(in super::super) fn init(sender: AsyncComponentSender<App>) -> Result<Self, ()> {
        let icon = Self { sender };

        let tray_service = ksni::TrayService::new(icon.clone());

        tray_service.spawn();

        Ok(icon)
    }
}

impl ksni::Tray for TrayIcon {
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").to_string()
    }

    fn icon_name(&self) -> String {
        "space-acres".to_string()
    }

    fn title(&self) -> String {
        "Space Acres".to_string()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        let icon_img = load_icon();

        let (width, height) = icon_img.dimensions();

        vec![ksni::Icon {
            width: width as i32,
            height: height as i32,
            data: icon_img.into_raw().to_vec(),
        }]
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            title: "Space Acres".to_string(),
            ..Default::default()
        }
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;

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
