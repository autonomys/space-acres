use crate::frontend::tray_icon::load_icon;
use crate::frontend::{App, AppCommandOutput, T};
use futures::channel::oneshot;
use ksni::menu::{MenuItem, StandardItem};
use ksni::{Icon, ToolTip, Tray, TrayService};
use relm4::{AsyncComponentSender, Sender};
use std::any::Any;
use std::cell::RefCell;
use tracing::{error, warn};

pub(in super::super) async fn spawn(sender: &AsyncComponentSender<App>) -> Option<Box<dyn Any>> {
    let (initialized_sender, initialized_receiver) = oneshot::channel();

    sender.spawn_command(move |sender| {
        let icon = TrayIcon {
            sender,
            initialized: RefCell::new(Some(initialized_sender)),
        };

        let tray_service = TrayService::new(icon);

        if let Err(error) = tray_service.run() {
            warn!(%error, "Tray icon error");
        }
    });

    initialized_receiver
        .await
        .unwrap_or_default()
        .then_some(Box::new(()))
}

pub(in super::super) struct TrayIcon {
    sender: Sender<AppCommandOutput>,
    initialized: RefCell<Option<oneshot::Sender<bool>>>,
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
                    if let Err(error) = this.sender.send(AppCommandOutput::ShowWindow) {
                        error!(?error, "Failed to send tray icon notification");
                    }
                }),
                ..StandardItem::default()
            }
            .into(),
            StandardItem {
                label: T.tray_icon_close().to_string(),
                activate: Box::new(|this: &mut Self| {
                    if let Err(error) = this.sender.send(AppCommandOutput::HideWindow) {
                        error!(?error, "Failed to send tray icon notification");
                    }
                }),
                ..StandardItem::default()
            }
            .into(),
        ]
    }

    fn watcher_online(&self) {
        if let Some(initialized) = self.initialized.borrow_mut().take() {
            if let Err(_error) = initialized.send(true) {
                warn!("Failed to send initialized notification");
            }
        }
    }

    fn watcher_offine(&self) -> bool {
        warn!("Tray icon not supported on this platform");

        if let Some(initialized) = self.initialized.borrow_mut().take() {
            if let Err(_error) = initialized.send(false) {
                warn!("Failed to send initialized notification");
            }
        }
        false
    }
}
