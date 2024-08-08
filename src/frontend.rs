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
use betrayer::{Icon, Menu, MenuItem, TrayEvent, TrayIconBuilder};
use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use gtk::gio::{Notification, NotificationPriority};
use gtk::prelude::*;
use relm4::actions::{RelmAction, RelmActionGroup};
use relm4::prelude::*;
use relm4::{Sender, ShutdownReceiver};
use relm4_icons::icon_name;
use std::cell::Cell;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::rc::Rc;
use std::{env, fmt};
use tracing::{debug, error, info, warn};

pub const GLOBAL_CSS: &str = include_str!("../res/app.css");
const ICON: &[u8] = include_bytes!("../res/icon.png");
const ABOUT_IMAGE: &[u8] = include_bytes!("../res/about.png");

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum TrayMenuSignal {
    Open,
    Close,
}

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
    Exit,
}

#[derive(Debug)]
pub enum AppCommandOutput {
    BackendNotification(BackendNotification),
    Restart,
}

enum View {
    Welcome,
    Upgrade { chain_name: String },
    Loading,
    Configuration,
    Reconfiguration,
    Running,
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
            Self::Stopped(_) => T.stopped_title(),
            Self::Error(_) => T.error_title(),
        }
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
enum StatusBarNotification {
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

impl StatusBarNotification {
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
    pub backend_fut: Pin<Box<dyn Future<Output = Result<(), futures::channel::oneshot::Canceled>>>>,
    pub backend_action_sender: mpsc::Sender<BackendAction>,
    pub backend_notification_receiver: mpsc::Receiver<BackendNotification>,
}

pub struct AppInit {
    pub app_data_dir: Option<PathBuf>,
    pub exit_status_code: Rc<Cell<AppStatusCode>>,
    pub minimize_on_start: bool,
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
    status_bar_notification: StatusBarNotification,
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
    // Stored here so `Drop` is called on this future as well, preventing exit until everything shuts down gracefully
    #[do_not_track]
    _background_tasks: Box<dyn Future<Output = ()>>,
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
                                set_from_pixbuf: Some(
                                    &gtk::gdk_pixbuf::Pixbuf::from_read(ABOUT_IMAGE)
                                        .expect("Statically correct image; qed")
                                ),
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
                                set_from_pixbuf: Some(
                                    &gtk::gdk_pixbuf::Pixbuf::from_read(ABOUT_IMAGE)
                                        .expect("Statically correct image; qed")
                                ),
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
                        #[track = "model.changed_status_bar_notification()"]
                        set_visible: !model.status_bar_notification.is_none(),

                        gtk::Label {
                            #[track = "model.changed_status_bar_notification()"]
                            set_css_classes: &[model.status_bar_notification.css_class()],
                            #[track = "model.changed_status_bar_notification()"]
                            set_label: model.status_bar_notification.message(),
                        },

                        gtk::Button {
                            add_css_class: "suggested-action",
                            connect_clicked => AppInput::Restart,
                            set_label: &T.status_bar_button_restart(),
                            #[track = "model.changed_status_bar_notification()"]
                            set_visible: model.status_bar_notification.restart_button(),
                        },

                        gtk::Button {
                            connect_clicked => AppInput::CloseStatusBarWarning,
                            set_label: &T.status_bar_button_ok(),
                            #[track = "model.changed_status_bar_notification()"]
                            set_visible: model.status_bar_notification.ok_button(),
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
            .logo(&gtk::gdk::Texture::for_pixbuf(
                &gtk::gdk_pixbuf::Pixbuf::from_read(ABOUT_IMAGE)
                    .expect("Statically correct image; qed"),
            ))
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
            gtk::glib::Propagation::Stop
        });

        #[cfg(any(target_os = "macos", target_os = "linux"))]
        let tray_img = {
            let img = image::load_from_memory_with_format(ICON, image::ImageFormat::Png)
                .expect("Tray icon is a valid PNG; qed");
            Icon::from_rgba(img.to_rgba8().into_vec(), img.width(), img.height())
                .expect("Betrayer normalization tray icon; qed")
        };

        #[cfg(target_os = "windows")]
        let tray_img = Icon::from_resource(1, None).expect("Tray icon is a valid ICO; qed");

        // TODO: Re-enable macOS once https://github.com/subspace/space-acres/issues/183 and/or
        //  https://github.com/subspace/space-acres/issues/222 are resolved
        let tray_icon = if cfg!(target_os = "macos") {
            None
        } else {
            TrayIconBuilder::new()
                .with_icon(tray_img)
                .with_tooltip("Space Acres")
                .with_menu(Menu::new([
                    MenuItem::button(T.tray_icon_open(), TrayMenuSignal::Open),
                    MenuItem::button(T.tray_icon_close(), TrayMenuSignal::Close),
                ]))
                .build({
                    let sender = sender.clone();
                    move |tray_event| {
                        if let TrayEvent::Menu(signal) = tray_event {
                            match signal {
                                TrayMenuSignal::Open => sender.input(AppInput::ShowWindow),
                                TrayMenuSignal::Close => sender.input(AppInput::Exit),
                            }
                        }
                    }
                })
                .map_err(|error| {
                    warn!(%error, "Unable to create tray icon");
                })
                .ok()
        };
        let has_tray_icon = tray_icon.is_some();

        let model = Self {
            current_view: View::Loading,
            current_raw_config: None,
            status_bar_notification: StatusBarNotification::None,
            backend_action_sender,
            new_version,
            loading_view,
            configuration_view,
            running_view,
            about_dialog,
            app_data_dir,
            exit_status_code,
            loaded: false,
            _background_tasks: Box::new(async move {
                match backend_fut.await {
                    Ok(()) => {
                        info!("Backend exited");
                    }
                    Err(_) => {
                        error!("Backend spawning failed");
                    }
                }
            }),
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
                sender.input(AppInput::Exit);
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
                    AppInput::Exit
                });
                gtk::glib::Propagation::Stop
            }
        });
        if has_tray_icon {
            root.connect_hide({
                let notification_shown_already = Cell::new(false);

                move |_window| {
                    if !notification_shown_already.replace(true) {
                        let icon = gtk::gdk_pixbuf::Pixbuf::from_read(ICON)
                            .expect("Statically correct image; qed");
                        let notification =
                            Notification::new(&T.notification_app_minimized_to_tray());
                        notification.set_body(Some(&T.notification_app_minimized_to_tray_body()));
                        // TODO: This icon is not rendered properly for some reason
                        notification.set_icon(&icon);
                        notification.set_priority(NotificationPriority::Low);
                        relm4::main_application().send_notification(None, &notification);
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
                relm4::main_application().quit();
            }
            AppInput::CloseStatusBarWarning => {
                self.set_status_bar_notification(StatusBarNotification::None);
            }
            AppInput::HideWindow => {
                root.hide();
            }
            AppInput::ShowWindow => {
                root.present();
            }
            AppInput::Exit => {
                relm4::main_application().quit();
            }
        }
    }

    async fn update_cmd(
        &mut self,
        input: Self::CommandOutput,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        // Reset changes
        self.reset();

        self.process_command(input);
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

    fn process_backend_notification(&mut self, notification: BackendNotification) {
        debug!(?notification, "New backend notification");

        match notification {
            BackendNotification::Loading(step) => {
                self.set_current_view(View::Loading);
                self.set_status_bar_notification(StatusBarNotification::None);
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
                self.set_status_bar_notification(StatusBarNotification::Warning {
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
                    self.set_status_bar_notification(StatusBarNotification::Warning {
                        message: T
                            .status_bar_message_restart_is_needed_for_configuration()
                            .to_string(),
                        ok: false,
                        restart: true,
                    });
                }
                Err(error) => {
                    self.set_status_bar_notification(StatusBarNotification::Error(
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
                self.set_current_view(View::Stopped(error));
            }
            BackendNotification::IrrecoverableError { error } => {
                self.set_current_view(View::Error(error.to_string()));
            }
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

    fn process_command(&mut self, input: AppCommandOutput) {
        match input {
            AppCommandOutput::BackendNotification(notification) => {
                self.process_backend_notification(notification);
            }
            AppCommandOutput::Restart => {
                self.exit_status_code.set(AppStatusCode::Restart);
                relm4::main_application().quit();
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
