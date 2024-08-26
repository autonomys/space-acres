pub mod configuration;
pub mod loading;
pub mod new_version;
pub mod running;
pub mod translations;
mod widgets;

use crate::backend::config::RawConfig;
use crate::backend::farmer::FarmerAction;
use crate::backend::{wipe, BackendAction, BackendNotification};
use crate::frontend::configuration::{ConfigurationInput, ConfigurationOutput, ConfigurationView};
use crate::frontend::loading::{LoadingInput, LoadingView};
use crate::frontend::new_version::NewVersion;
use crate::frontend::running::{RunningInit, RunningInput, RunningOutput, RunningView};
use crate::frontend::translations::{AsDefaultStr, T};
use crate::AppStatusCode;
use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use gtk::glib;
use gtk::prelude::*;
use image::{ImageBuffer, Rgba};
use notify_rust::Notification;
use relm4::actions::{RelmAction, RelmActionGroup};
use relm4::prelude::*;
use relm4::{Sender, ShutdownReceiver};
use relm4_icons::icon_name;
use std::cell::{Cell, LazyCell};
use std::future::Future;
use std::path::PathBuf;
use std::rc::Rc;
use std::{env, fmt};
use tracing::{debug, error, warn};

pub const GLOBAL_CSS: &str = include_str!("../res/app.css");
const ICON: &[u8] = include_bytes!("../res/icon.png");
const ABOUT_IMAGE: &[u8] = include_bytes!("../res/about.png");

#[cfg(all(unix, not(target_os = "macos")))]
#[thread_local]
static PIXBUF_ICON: LazyCell<gtk::gdk_pixbuf::Pixbuf> = LazyCell::new(|| {
    gtk::gdk_pixbuf::Pixbuf::from_read(ICON).expect("Statically correct image; qed")
});

trait NotificationExt {
    fn with_typical_options(&mut self) -> &mut Self;
}

impl NotificationExt for Notification {
    fn with_typical_options(&mut self) -> &mut Self {
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            use notify_rust::Image;

            let image = if PIXBUF_ICON.has_alpha() {
                Image::from_rgba(
                    PIXBUF_ICON.width(),
                    PIXBUF_ICON.height(),
                    PIXBUF_ICON.read_pixel_bytes().to_vec(),
                )
                .expect("Image is statically correct; qed")
            } else {
                Image::from_rgb(
                    PIXBUF_ICON.width(),
                    PIXBUF_ICON.height(),
                    PIXBUF_ICON.read_pixel_bytes().to_vec(),
                )
                .expect("Image is statically correct; qed")
            };

            // This is how we set an icon on Linux
            self.image_data(image);
        }
        #[cfg(windows)]
        {
            // UUID comes from https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid
            // and the whole things is auto-generated for application's Start icon (see
            // get-StartApps in PowerShell), this is the `AppUserModelId` on Windows.
            const APP_USER_MODEL_ID: &str =
                "{6D809377-6AF0-444B-8957-A3773F02200E}\\Space Acres\\bin\\space-acres.exe";
            // This is how we'll get proper icon and application name on Windows (only when
            // installed though)
            self.app_id(APP_USER_MODEL_ID);
        }
        #[cfg(target_os = "macos")]
        {
            static INIT_APPLICATION_ONCE: std::sync::Once = std::sync::Once::new();

            INIT_APPLICATION_ONCE.call_once(|| {
                // Our bundle identifier for macOS package for notifications
                if let Err(error) = notify_rust::set_application("xyz.autonomys.space-acres") {
                    error!(%error, "Failed to set application bundle identifier")
                }
            })
        }

        self
    }
}

pub(crate) fn load_icon() -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    image::load_from_memory_with_format(ICON, image::ImageFormat::Png)
        .expect("Statically correct image; qed")
        .to_rgba8()
}

#[cfg(all(unix, not(target_os = "macos")))]
mod generic_tray_icon {
    use super::{load_icon, App, AppInput, T};
    use relm4::AsyncComponentSender;

    #[derive(Clone)]
    pub struct TrayIcon {
        sender: AsyncComponentSender<App>,
    }

    impl TrayIcon {
        pub(crate) fn init(sender: AsyncComponentSender<App>) -> Result<Self, ()> {
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
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
mod generic_tray_icon {
    use super::{load_icon, App, AppInput, T};
    use relm4::AsyncComponentSender;
    use tracing::warn;

    pub struct TrayIcon {
        _icon: tray_icon::TrayIcon,
        sender: AsyncComponentSender<App>,
    }

    impl TrayIcon {
        pub(crate) fn init(sender: AsyncComponentSender<App>) -> Result<Self, ()> {
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
}

#[thread_local]
static PIXBUF_ABOUT_IMG: LazyCell<gtk::gdk_pixbuf::Pixbuf> = LazyCell::new(|| {
    gtk::gdk_pixbuf::Pixbuf::from_read(ABOUT_IMAGE).expect("Statically correct image; qed")
});

#[derive(Debug)]
pub enum AppInput {
    Configuration(ConfigurationOutput),
    Running(RunningOutput),
    OpenLogsFolder,
    ChangeConfiguration,
    OpenFeedbackLink,
    OpenCommunityHelpLink,
    ShowAboutDialog,
    InitialConfiguration,
    StartUpgrade,
    Restart,
    CloseStatusBarWarning,
    HideWindow,
    ShowWindow,
    ShutDown,
}

#[derive(Debug)]
pub enum AppCommandOutput {
    BackendNotification(BackendNotification),
    Restart,
    Quit,
}

enum View {
    Welcome,
    Upgrade { chain_name: String },
    Loading,
    Configuration,
    Reconfiguration,
    Running,
    ShuttingDown,
    Stopped(Option<anyhow::Error>),
    Error(String),
}

impl View {
    fn title(&self) -> impl fmt::Display {
        match self {
            Self::Welcome => T.welcome_title(),
            Self::Upgrade { .. } => T.upgrade_title(),
            Self::Loading => T.loading_title(),
            Self::Configuration => T.configuration_title(),
            Self::Reconfiguration => T.reconfiguration_title(),
            Self::Running => T.running_title(),
            Self::ShuttingDown => T.shutting_down_title(),
            Self::Stopped(_) => T.stopped_title(),
            Self::Error(_) => T.error_title(),
        }
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
enum StatusBarContents {
    #[default]
    None,
    Warning {
        message: String,
        /// Whether to show ok button
        ok: bool,
        /// Whether to show restart button
        restart: bool,
    },
    Error(String),
}

impl StatusBarContents {
    fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    fn css_class(&self) -> &'static str {
        match self {
            Self::None => "label",
            Self::Warning { .. } => "warning-label",
            Self::Error(_) => "error-label",
        }
    }

    fn message(&self) -> &str {
        match self {
            Self::None => "",
            Self::Warning { message, .. } | Self::Error(message) => message.as_str(),
        }
    }

    fn ok_button(&self) -> bool {
        match self {
            Self::Warning { ok, .. } => *ok,
            _ => false,
        }
    }

    fn restart_button(&self) -> bool {
        match self {
            Self::Warning { restart, .. } => *restart,
            _ => false,
        }
    }
}

pub struct RunBackendResult {
    pub backend_fut: Box<dyn Future<Output = ()> + Send>,
    pub backend_action_sender: mpsc::Sender<BackendAction>,
    pub backend_notification_receiver: mpsc::Receiver<BackendNotification>,
}

pub struct AppInit {
    pub app_data_dir: Option<PathBuf>,
    pub exit_status_code: Rc<Cell<AppStatusCode>>,
    pub minimize_on_start: bool,
    pub crash_notification: bool,
    pub run_backend: fn() -> RunBackendResult,
}

relm4::new_action_group!(MainMenu, "main_menu");
relm4::new_stateless_action!(MainMenuShowLogs, MainMenu, "show_logs");
relm4::new_stateless_action!(
    MainMenuChangeConfiguration,
    MainMenu,
    "change_configuration"
);
relm4::new_stateless_action!(MainMenuShareFeedback, MainMenu, "share_feedback");
relm4::new_stateless_action!(MainMenuAbout, MainMenu, "about");
relm4::new_stateless_action!(MainMenuExit, MainMenu, "exit");

#[tracker::track]
pub struct App {
    #[no_eq]
    current_view: View,
    current_raw_config: Option<RawConfig>,
    status_bar_contents: StatusBarContents,
    #[do_not_track]
    backend_action_sender: mpsc::Sender<BackendAction>,
    #[do_not_track]
    new_version: Controller<NewVersion>,
    #[do_not_track]
    loading_view: Controller<LoadingView>,
    #[do_not_track]
    configuration_view: AsyncController<ConfigurationView>,
    #[do_not_track]
    running_view: Controller<RunningView>,
    #[do_not_track]
    about_dialog: gtk::AboutDialog,
    #[do_not_track]
    app_data_dir: Option<PathBuf>,
    #[do_not_track]
    exit_status_code: Rc<Cell<AppStatusCode>>,
    #[do_not_track]
    loaded: bool,
    #[do_not_track]
    backend_fut: Option<Box<dyn Future<Output = ()> + Send>>,
    // Keep it around so it doesn't disappear
    #[do_not_track]
    _tray_icon: Option<generic_tray_icon::TrayIcon>,
}

#[relm4::component(pub async)]
impl AsyncComponent for App {
    type Init = AppInit;
    type Input = AppInput;
    type Output = ();
    type CommandOutput = AppCommandOutput;

    view! {
        gtk::Window {
            set_decorated: false,
            set_resizable: false,
            set_size_request: (800, 600),
            #[track = "model.changed_current_view()"]
            set_title: Some(&format!("{} - Space Acres {}", model.current_view.title(), env!("CARGO_PKG_VERSION"))),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::HeaderBar {
                    pack_end = &gtk::Box {
                        set_spacing: 10,

                        model.new_version.widget().clone(),

                        // TODO: Two menu buttons is a hack for not showing configuration in some
                        //  cases, would be nice to just hide corresponding menu item instead
                        gtk::MenuButton {
                            set_direction: gtk::ArrowType::None,
                            set_icon_name: icon_name::MENU_LARGE,
                            set_popover: Some(&gtk::PopoverMenu::from_model(Some(&main_menu_without_change_configuration))),
                            #[track = "model.changed_current_raw_config()"]
                            set_visible: model.current_raw_config.is_none(),
                        },

                        gtk::MenuButton {
                            set_direction: gtk::ArrowType::None,
                            set_icon_name: icon_name::MENU_LARGE,
                            set_popover: Some(&gtk::PopoverMenu::from_model(Some(&main_menu))),
                            #[track = "model.changed_current_raw_config()"]
                            set_visible: model.current_raw_config.is_some(),
                        },
                    },
                },

                gtk::Box {
                    set_margin_all: 10,
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 10,

                    #[transition = "SlideLeftRight"]
                    match &model.current_view {
                        View::Welcome => gtk::Box {
                            set_margin_all: 10,
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 20,

                            gtk::Image {
                                set_height_request: 256,
                                set_from_pixbuf: Some(&*PIXBUF_ABOUT_IMG),
                            },

                            gtk::Label {
                                set_label: &T.welcome_message(),
                                set_wrap: true,
                            },

                            gtk::Box {
                                set_halign: gtk::Align::End,


                                gtk::Button {
                                    add_css_class: "suggested-action",
                                    connect_clicked => AppInput::InitialConfiguration,

                                    gtk::Label {
                                        set_label: &T.welcome_button_continue(),
                                        set_margin_all: 10,
                                    },
                                },
                            },
                        },
                        View::Upgrade { chain_name } => gtk::Box {
                            set_margin_all: 10,
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 20,

                            gtk::Image {
                                set_height_request: 256,
                                set_from_pixbuf: Some(&*PIXBUF_ABOUT_IMG),
                            },

                            gtk::Label {
                                set_label: &T.upgrade_message(),
                                set_wrap: true,
                            },

                            gtk::Box {
                                set_halign: gtk::Align::End,


                                gtk::Button {
                                    add_css_class: "destructive-action",
                                    connect_clicked => AppInput::StartUpgrade,

                                    gtk::Label {
                                        #[track = "model.changed_current_view()"]
                                        set_label: T.upgrade_button_upgrade(chain_name).as_str(),
                                        set_margin_all: 10,
                                    },
                                },
                            },
                        },
                        View::Loading => model.loading_view.widget().clone(),
                        View::Configuration | View::Reconfiguration => model.configuration_view.widget().clone(),
                        View::Running=> model.running_view.widget().clone(),
                        View::ShuttingDown=> gtk::Box {
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,
                            set_vexpand: true,
                            set_orientation: gtk::Orientation::Vertical,

                            gtk::Spinner {
                                start: (),
                                set_size_request: (50, 50),
                            },

                            gtk::Label {
                                set_label: &T.shutting_down_description(),
                            },
                        },
                        View::Stopped(Some(error)) => gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 20,
                            set_valign: gtk::Align::Center,

                            gtk::Label {
                                #[track = "model.changed_current_view()"]
                                set_label: T.stopped_message_with_error(error.to_string()).as_str(),
                            },

                            gtk::Box {
                                set_halign: gtk::Align::Center,
                                set_spacing: 10,

                                gtk::Button {
                                    set_label: &T.stopped_button_show_logs(),
                                    connect_clicked => AppInput::OpenLogsFolder,
                                },

                                gtk::Button {
                                    set_label: &T.stopped_button_help_from_community(),
                                    connect_clicked => AppInput::OpenCommunityHelpLink,
                                },
                            },
                        },
                        View::Stopped(None) => {
                            gtk::Label {
                                set_label: &T.stopped_message(),
                            }
                        },
                        View::Error(error) => gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 20,
                            set_valign: gtk::Align::Center,

                            gtk::Label {
                                #[track = "model.changed_current_view()"]
                                set_label: &T.error_message(error.to_string()).as_str(),
                            },

                            gtk::Box {
                                set_halign: gtk::Align::Center,
                                set_spacing: 10,

                                gtk::Button {
                                    set_label: &T.error_button_show_logs(),
                                    connect_clicked => AppInput::OpenLogsFolder,
                                },

                                gtk::Button {
                                    set_label: &T.error_button_help_from_community(),
                                    connect_clicked => AppInput::OpenCommunityHelpLink,
                                },
                            },
                        },
                    },

                    gtk::Box {
                        set_halign: gtk::Align::Center,
                        set_spacing: 10,
                        #[track = "model.changed_status_bar_contents()"]
                        set_visible: !model.status_bar_contents.is_none(),

                        gtk::Label {
                            #[track = "model.changed_status_bar_contents()"]
                            set_css_classes: &[model.status_bar_contents.css_class()],
                            #[track = "model.changed_status_bar_contents()"]
                            set_label: model.status_bar_contents.message(),
                        },

                        gtk::Button {
                            add_css_class: "suggested-action",
                            connect_clicked => AppInput::Restart,
                            set_label: &T.status_bar_button_restart(),
                            #[track = "model.changed_status_bar_contents()"]
                            set_visible: model.status_bar_contents.restart_button(),
                        },

                        gtk::Button {
                            connect_clicked => AppInput::CloseStatusBarWarning,
                            set_label: &T.status_bar_button_ok(),
                            #[track = "model.changed_status_bar_contents()"]
                            set_visible: model.status_bar_contents.ok_button(),
                        },
                    },
                },
            }
        }
    }

    menu! {
        main_menu_without_change_configuration: {
            &T.main_menu_show_logs() => MainMenuShowLogs,
            &T.main_menu_share_feedback() => MainMenuShareFeedback,
            &T.main_menu_about() => MainMenuAbout,
            &T.main_menu_exit() => MainMenuExit,
        },

        main_menu: {
            &T.main_menu_show_logs() => MainMenuShowLogs,
            &T.main_menu_change_configuration() => MainMenuChangeConfiguration,
            &T.main_menu_share_feedback() => MainMenuShareFeedback,
            &T.main_menu_about() => MainMenuAbout,
            &T.main_menu_exit() => MainMenuExit,
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let AppInit {
            app_data_dir,
            exit_status_code,
            minimize_on_start,
            crash_notification,
            run_backend,
        } = init;
        let RunBackendResult {
            backend_fut,
            backend_action_sender,
            mut backend_notification_receiver,
        } = run_backend();

        // Forward backend notifications
        sender.command(move |sender, shutdown_receiver| {
            shutdown_receiver
                .register(async move {
                    while let Some(notification) = backend_notification_receiver.next().await {
                        if let Err(error) =
                            sender.send(AppCommandOutput::BackendNotification(notification))
                        {
                            error!(?error, "Failed to forward backend notification");
                            break;
                        }
                    }
                })
                .drop_on_shutdown()
        });

        let new_version = NewVersion::builder().launch(()).detach();

        let loading_view = LoadingView::builder().launch(()).detach();

        let configuration_view = ConfigurationView::builder()
            .launch(root.clone())
            .forward(sender.input_sender(), AppInput::Configuration);

        let running_view = RunningView::builder()
            .launch(RunningInit {
                // Not paused on start
                plotting_paused: false,
            })
            .forward(sender.input_sender(), AppInput::Running);

        let about_dialog = gtk::AboutDialog::builder()
            .title("About")
            .program_name("Space Acres")
            .version(env!("CARGO_PKG_VERSION"))
            .authors(env!("CARGO_PKG_AUTHORS").split(':').collect::<Vec<_>>())
            // TODO: Use https://gitlab.gnome.org/GNOME/gtk/-/merge_requests/6643 once available
            .license("Zero-Clause BSD: https://opensource.org/license/0bsd/")
            .website(env!("CARGO_PKG_REPOSITORY"))
            .website_label("GitHub")
            .comments(env!("CARGO_PKG_DESCRIPTION"))
            .logo(&gtk::gdk::Texture::for_pixbuf(&PIXBUF_ABOUT_IMG))
            .system_information({
                let config_directory = dirs::config_local_dir()
                    .map(|config_local_dir| {
                        config_local_dir
                            .join(env!("CARGO_PKG_NAME"))
                            .display()
                            .to_string()
                    })
                    .unwrap_or_else(|| "Unknown".to_string());
                let data_directory = dirs::data_local_dir()
                    .map(|data_local_dir| {
                        data_local_dir
                            .join(env!("CARGO_PKG_NAME"))
                            .display()
                            .to_string()
                    })
                    .unwrap_or_else(|| "Unknown".to_string());

                T.about_system_information(config_directory, data_directory)
                    .as_str()
            })
            .transient_for(&root)
            .build();

        about_dialog.connect_close_request(|about_dialog| {
            about_dialog.hide();
            glib::Propagation::Stop
        });

        let tray_icon = generic_tray_icon::TrayIcon::init(sender.clone()).ok();

        let has_tray_icon = tray_icon.is_some();

        let model = Self {
            current_view: View::Loading,
            current_raw_config: None,
            status_bar_contents: if crash_notification {
                StatusBarContents::Warning {
                    message: T.status_bar_message_restarted_after_crash().to_string(),
                    ok: true,
                    restart: false,
                }
            } else {
                StatusBarContents::None
            },
            backend_action_sender,
            new_version,
            loading_view,
            configuration_view,
            running_view,
            about_dialog,
            app_data_dir,
            exit_status_code,
            loaded: false,
            backend_fut: Some(backend_fut),
            _tray_icon: tray_icon,
            tracker: u8::MAX,
        };

        let widgets = view_output!();

        let mut menu_actions_group = RelmActionGroup::<MainMenu>::new();
        menu_actions_group.add_action(RelmAction::<MainMenuShowLogs>::new_stateless({
            let sender = sender.clone();

            move |_| {
                sender.input(AppInput::OpenLogsFolder);
            }
        }));
        menu_actions_group.add_action(RelmAction::<MainMenuChangeConfiguration>::new_stateless({
            let sender = sender.clone();

            move |_| {
                sender.input(AppInput::ChangeConfiguration);
            }
        }));
        menu_actions_group.add_action(RelmAction::<MainMenuShareFeedback>::new_stateless({
            let sender = sender.clone();

            move |_| {
                sender.input(AppInput::OpenFeedbackLink);
            }
        }));
        menu_actions_group.add_action(RelmAction::<MainMenuAbout>::new_stateless({
            let sender = sender.clone();

            move |_| {
                sender.input(AppInput::ShowAboutDialog);
            }
        }));
        menu_actions_group.add_action(RelmAction::<MainMenuExit>::new_stateless({
            let sender = sender.clone();

            move |_| {
                sender.input(AppInput::ShutDown);
            }
        }));
        menu_actions_group.register_for_widget(&root);

        if minimize_on_start {
            if has_tray_icon {
                root.hide()
            } else {
                root.minimize()
            }
        }

        relm4::main_application().connect_activate({
            let root = root.clone();

            move |_application| {
                root.present();
            }
        });

        root.connect_close_request({
            let sender = sender.clone();

            move |_root| {
                sender.input(if has_tray_icon {
                    AppInput::HideWindow
                } else {
                    AppInput::ShutDown
                });
                glib::Propagation::Stop
            }
        });

        if has_tray_icon {
            root.connect_hide({
                let sender = sender.clone();
                let notification_shown_already = Cell::new(false);

                move |_window| {
                    if !notification_shown_already.replace(true) {
                        sender.spawn_command(|_sender| {
                            let mut notification = Notification::new();
                            notification
                                .summary(&T.notification_app_minimized_to_tray())
                                .body(&T.notification_app_minimized_to_tray_body())
                                .with_typical_options();
                            #[cfg(all(unix, not(target_os = "macos")))]
                            notification.urgency(notify_rust::Urgency::Low);
                            if let Err(error) = notification.show() {
                                warn!(%error, "Failed to show desktop notification");
                            }
                        });
                    }
                }
            });
        }

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        input: Self::Input,
        sender: AsyncComponentSender<Self>,
        root: &Self::Root,
    ) {
        // Reset changes
        self.reset();

        match input {
            AppInput::OpenLogsFolder => {
                self.open_log_folder();
            }
            AppInput::Configuration(configuration_output) => {
                self.process_configuration_output(configuration_output)
                    .await;
            }
            AppInput::Running(running_output) => {
                self.process_running_output(running_output).await;
            }
            AppInput::ChangeConfiguration => {
                let configuration_already_opened = matches!(
                    self.current_view,
                    View::Configuration | View::Reconfiguration
                );
                if !configuration_already_opened
                    && let Some(raw_config) = self.current_raw_config.clone()
                {
                    self.configuration_view
                        .emit(ConfigurationInput::Reinitialize {
                            raw_config,
                            reconfiguration: true,
                        });
                    self.set_current_view(View::Reconfiguration);
                }
            }
            AppInput::OpenFeedbackLink => {
                if let Err(error) = open::that_detached("https://linktr.ee/autonomys_network") {
                    error!(%error, "Failed to open share feedback page in default browser");
                }
            }
            AppInput::OpenCommunityHelpLink => {
                if let Err(error) = open::that_detached("https://docs.autonomys.xyz/docs/farming-&-staking/farming/space-acres/space-acres-install#troubleshooting") {
                    error!(%error, "Failed to open share community help in default browser");
                }
            }
            AppInput::ShowAboutDialog => {
                self.about_dialog.show();
            }
            AppInput::InitialConfiguration => {
                self.set_current_view(View::Configuration);
            }
            AppInput::StartUpgrade => {
                let raw_config = self
                    .current_raw_config
                    .clone()
                    .expect("Must have raw config when corresponding button is clicked; qed");
                sender.command(move |sender, shutdown_receiver| async move {
                    Self::do_upgrade(sender, shutdown_receiver, raw_config).await;
                });
                self.set_current_view(View::Loading);
            }
            AppInput::Restart => {
                self.exit_status_code.set(AppStatusCode::Restart);
                // Delegate to exit to do the rest
                sender.input(AppInput::ShutDown);
            }
            AppInput::CloseStatusBarWarning => {
                self.set_status_bar_contents(StatusBarContents::None);
            }
            AppInput::HideWindow => {
                root.hide();
            }
            AppInput::ShowWindow => {
                root.present();
            }
            AppInput::ShutDown => {
                self.set_current_view(View::ShuttingDown);
                // Make sure user sees that shutdown is happening in case it is called from tray
                // icon
                root.present();

                let backend_fut = self.backend_fut.take();
                sender.spawn_oneshot_command(|| {
                    drop(backend_fut);
                    AppCommandOutput::Quit
                });
            }
        }
    }

    async fn update_cmd(
        &mut self,
        input: Self::CommandOutput,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        // Reset changes
        self.reset();

        self.process_command(input, sender);
    }
}

impl App {
    fn open_log_folder(&mut self) {
        let Some(app_data_dir) = &self.app_data_dir else {
            return;
        };
        if let Err(error) = open::that_detached(app_data_dir) {
            error!(%error, path = %app_data_dir.display(), "Failed to open logs folder");
        }
    }

    async fn process_configuration_output(&mut self, configuration_output: ConfigurationOutput) {
        match configuration_output {
            ConfigurationOutput::StartWithNewConfig(raw_config) => {
                if let Err(error) = self
                    .backend_action_sender
                    .send(BackendAction::NewConfig { raw_config })
                    .await
                {
                    self.set_current_view(View::Error(
                        T.error_message_failed_to_send_config_to_backend(error.to_string())
                            .to_string(),
                    ));
                }
            }
            ConfigurationOutput::ConfigUpdate(raw_config) => {
                self.get_mut_current_raw_config()
                    .replace(raw_config.clone());
                // Config is updated when application is already running, switch to corresponding screen
                self.set_current_view(View::Running);
                if let Err(error) = self
                    .backend_action_sender
                    .send(BackendAction::NewConfig { raw_config })
                    .await
                {
                    self.set_current_view(View::Error(
                        T.error_message_failed_to_send_config_to_backend(error.to_string())
                            .to_string(),
                    ));
                }
            }
            ConfigurationOutput::Back => {
                // Back to welcome screen
                self.set_current_view(View::Welcome);
            }
            ConfigurationOutput::Close => {
                // Configuration view is closed when application is already running, switch to
                // corresponding screen
                if self.loaded {
                    self.set_current_view(View::Running);
                } else {
                    self.set_current_view(View::Loading);
                }
            }
        }
    }

    async fn process_running_output(&mut self, running_output: RunningOutput) {
        match running_output {
            RunningOutput::PausePlotting(pause_plotting) => {
                if let Err(error) = self
                    .backend_action_sender
                    .send(BackendAction::Farmer(FarmerAction::PausePlotting(
                        pause_plotting,
                    )))
                    .await
                {
                    self.set_current_view(View::Error(
                        T.error_message_failed_to_send_pause_plotting_to_backend(error.to_string())
                            .to_string(),
                    ));
                }
            }
        }
    }

    fn process_command(&mut self, input: AppCommandOutput, sender: AsyncComponentSender<Self>) {
        match input {
            AppCommandOutput::BackendNotification(notification) => {
                self.process_backend_notification(notification, sender);
            }
            AppCommandOutput::Restart => {
                sender.input(AppInput::Restart);
            }
            AppCommandOutput::Quit => {
                relm4::main_application().quit();
            }
        }
    }

    fn process_backend_notification(
        &mut self,
        notification: BackendNotification,
        sender: AsyncComponentSender<Self>,
    ) {
        debug!(?notification, "New backend notification");

        match notification {
            BackendNotification::Loading(step) => {
                self.set_current_view(View::Loading);
                self.loading_view.emit(LoadingInput::BackendLoading(step));
            }
            BackendNotification::ConfigurationFound { raw_config } => {
                self.get_mut_current_raw_config()
                    .replace(raw_config.clone());
            }
            BackendNotification::IncompatibleChain { compatible_chain } => {
                self.set_current_view(View::Upgrade {
                    chain_name: compatible_chain,
                });
            }
            BackendNotification::NotConfigured => {
                if self.current_raw_config.is_none() {
                    self.set_current_view(View::Welcome);
                } else {
                    self.set_current_view(View::Configuration);
                }
            }
            BackendNotification::ConfigurationIsInvalid { error } => {
                if let Some(raw_config) = self.current_raw_config.clone() {
                    self.configuration_view
                        .emit(ConfigurationInput::Reinitialize {
                            raw_config,
                            reconfiguration: false,
                        });
                }
                self.set_status_bar_contents(StatusBarContents::Warning {
                    message: T
                        .status_bar_message_configuration_is_invalid(error.to_string())
                        .as_str()
                        .to_string(),
                    ok: true,
                    restart: false,
                });
            }
            BackendNotification::ConfigSaveResult(result) => match result {
                Ok(()) => {
                    self.set_status_bar_contents(StatusBarContents::Warning {
                        message: T
                            .status_bar_message_restart_is_needed_for_configuration()
                            .to_string(),
                        ok: false,
                        restart: true,
                    });
                }
                Err(error) => {
                    self.set_status_bar_contents(StatusBarContents::Error(
                        T.status_bar_message_failed_to_save_configuration(error.to_string())
                            .to_string(),
                    ));
                }
            },
            BackendNotification::Running {
                config,
                raw_config,
                best_block_number,
                reward_address_balance,
                initial_farm_states,
                cache_percentage,
                chain_info,
                chain_constants,
            } => {
                self.loaded = true;
                self.get_mut_current_raw_config()
                    .replace(raw_config.clone());
                self.set_current_view(View::Running);
                self.running_view.emit(RunningInput::Initialize {
                    best_block_number,
                    reward_address_balance,
                    initial_farm_states,
                    cache_percentage,
                    config,
                    raw_config,
                    chain_info,
                    chain_constants,
                });
            }
            BackendNotification::Node(node_notification) => {
                self.running_view
                    .emit(RunningInput::NodeNotification(node_notification));
            }
            BackendNotification::Farmer(farmer_notification) => {
                self.running_view
                    .emit(RunningInput::FarmerNotification(farmer_notification));
            }
            BackendNotification::Stopped { error } => {
                sender.spawn_command(|_sender| {
                    let mut notification = Notification::new();
                    notification
                        .summary(&T.notification_stopped_with_error())
                        .body(&T.notification_stopped_with_error_body())
                        .with_typical_options();
                    #[cfg(all(unix, not(target_os = "macos")))]
                    notification.urgency(notify_rust::Urgency::Critical);
                    if let Err(error) = notification.show() {
                        warn!(%error, "Failed to show desktop notification");
                    }
                });

                self.set_current_view(View::Stopped(error));
            }
            BackendNotification::IrrecoverableError { error } => {
                self.set_current_view(View::Error(error.to_string()));
            }
        }
    }

    async fn do_upgrade(
        sender: Sender<AppCommandOutput>,
        shutdown_receiver: ShutdownReceiver,
        raw_config: RawConfig,
    ) {
        shutdown_receiver
            .register(async move {
                let (mut backend_notification_sender, mut backend_notification_receiver) =
                    mpsc::channel(100);

                tokio::spawn({
                    let sender = sender.clone();

                    async move {
                        while let Some(notification) = backend_notification_receiver.next().await {
                            if sender
                                .send(AppCommandOutput::BackendNotification(notification))
                                .is_err()
                            {
                                return;
                            }
                        }
                    }
                });

                if let Err(error) = wipe(&raw_config, &mut backend_notification_sender).await {
                    error!(%error, "Wiping error");
                }

                let _ = sender.send(AppCommandOutput::Restart);
            })
            .drop_on_shutdown()
            .await
    }
}
