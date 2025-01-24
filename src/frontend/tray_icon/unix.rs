use crate::frontend::tray_icon::load_icon;
use crate::frontend::{App, AppCommandOutput, T};
use ksni::menu::{MenuItem, StandardItem};
use ksni::{Handle, Icon, ToolTip, Tray, TrayMethods};
use relm4::{AsyncComponentSender, Sender};
use std::any::Any;
use tokio::task;
use tracing::{error, warn};

struct ShutdownWrapper {
    handle: Handle<TrayIcon>,
}

impl Drop for ShutdownWrapper {
    fn drop(&mut self) {
        let shutdown_awaiter = self.handle.shutdown();

        task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(shutdown_awaiter);
        })
    }
}

pub(in super::super) async fn spawn(sender: &AsyncComponentSender<App>) -> Option<Box<dyn Any>> {
    let icon = TrayIcon {
        sender: sender.command_sender().clone(),
    };

    match icon.spawn().await {
        Ok(handle) => Some(Box::new(ShutdownWrapper { handle })),
        Err(error) => {
            warn!(%error, "Tray icon not supported on this platform");
            None
        }
    }
}

pub(in super::super) struct TrayIcon {
    sender: Sender<AppCommandOutput>,
}

impl Tray for TrayIcon {
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").to_string()
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
                label: T.tray_icon_quit().to_string(),
                activate: Box::new(|this: &mut Self| {
                    if let Err(error) = this.sender.send(AppCommandOutput::Quit) {
                        error!(?error, "Failed to send tray icon notification");
                    }
                }),
                ..StandardItem::default()
            }
            .into(),
        ]
    }
}
