#![windows_subsystem = "windows"]
#![feature(const_option, trait_alias, try_blocks)]

mod backend;

use crate::backend::config::{Farm, RawConfig};
use crate::backend::farmer::{PlottingKind, PlottingState};
use crate::backend::node::{SyncKind, SyncState};
use crate::backend::{
    BackendAction, BackendNotification, FarmerNotification, LoadingStep, NodeNotification,
};
use bytesize::ByteSize;
use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use gtk::prelude::*;
use relm4::prelude::*;
use relm4::{set_global_css, RELM_THREADS};
use relm4_components::open_dialog::{OpenDialog, *};
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;
use std::thread::available_parallelism;
use subspace_core_primitives::BlockNumber;
use subspace_farmer::utils::ss58::parse_ss58_reward_address;
use subspace_proof_of_space::chia::ChiaTable;
use tokio::runtime::Handle;
use tracing::warn;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

const GLOBAL_CSS: &str = include_str!("../res/app.css");
const ABOUT_IMAGE: &[u8] = include_bytes!("../res/about.png");
// 2 GB
const MIN_FARM_SIZE: u64 = 1000 * 1000 * 1000 * 2;

type PosTable = ChiaTable;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum MaybeValid<T> {
    Unknown(T),
    Valid(T),
    Invalid(T),
}

impl<T> Default for MaybeValid<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::Unknown(T::default())
    }
}

impl<T> Deref for MaybeValid<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let (MaybeValid::Unknown(inner) | MaybeValid::Valid(inner) | MaybeValid::Invalid(inner)) =
            self;

        inner
    }
}

impl<T> MaybeValid<T> {
    fn unknown(&self) -> bool {
        matches!(self, MaybeValid::Unknown(_))
    }

    fn valid(&self) -> bool {
        matches!(self, MaybeValid::Valid(_))
    }

    fn icon(&self) -> Option<&'static str> {
        match self {
            MaybeValid::Unknown(_) => None,
            MaybeValid::Valid(_) => Some("emblem-ok-symbolic"),
            MaybeValid::Invalid(_) => Some("window-close-symbolic"),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum DirectoryKind {
    NodePath,
    FarmPath(usize),
}

#[derive(Debug)]
enum ConfigurationEvent {
    RewardAddressChanged(String),
    OpenDirectory(DirectoryKind),
    DirectorySelected(PathBuf),
    FarmSizeChanged { farm_index: usize, size: String },
    Start,
}

#[derive(Debug)]
enum AppInput {
    BackendNotification(BackendNotification),
    Configuration(ConfigurationEvent),
    ShowAboutDialog,
    Ignore,
}

#[derive(Debug, Default)]
struct DiskFarm {
    path: MaybeValid<PathBuf>,
    size: MaybeValid<String>,
}

#[derive(Debug, Default)]
struct NodeState {
    best_block_number: BlockNumber,
    sync_state: Option<SyncState>,
}

#[derive(Debug, Default)]
struct FarmerState {
    /// One entry per farm
    plotting_state: Vec<Option<PlottingState>>,
}

enum View {
    Loading(String),
    Configuration {
        reward_address: MaybeValid<String>,
        node_path: MaybeValid<PathBuf>,
        farms: Vec<DiskFarm>,
        pending_directory_selection: Option<DirectoryKind>,
    },
    Running {
        node_state: NodeState,
        farmer_state: FarmerState,
    },
    Stopped(Option<anyhow::Error>),
    Error(anyhow::Error),
}

impl View {
    fn title(&self) -> &'static str {
        match self {
            Self::Loading(_) => "Loading",
            Self::Configuration { .. } => "Configuration",
            Self::Running { .. } => "Running",
            Self::Stopped(_) => "Stopped",
            Self::Error(_) => "Error",
        }
    }
}

// TODO: Efficient updates with tracker
struct App {
    current_view: View,
    backend_action_sender: mpsc::Sender<BackendAction>,
    open_dialog: Controller<OpenDialog>,
    menu_popover: gtk::Popover,
    about_dialog: gtk::AboutDialog,
}

#[relm4::component(async)]
impl AsyncComponent for App {
    type Init = ();
    type Input = AppInput;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Window {
            set_decorated: false,
            set_resizable: false,
            set_size_request: (800, 600),
            #[watch]
            set_title: Some(&format!("{} - Space Acres {}", model.current_view.title(), env!("CARGO_PKG_VERSION"))),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::HeaderBar {
                    pack_end = &gtk::MenuButton {
                        set_direction: gtk::ArrowType::None,
                        set_icon_name: "open-menu-symbolic",
                        #[wrap(Some)]
                        set_popover: menu_popover = &gtk::Popover {
                            set_halign: gtk::Align::End,
                            set_position: gtk::PositionType::Bottom,

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 5,

                                gtk::Button {
                                    set_label: "About",
                                    connect_clicked[sender] => move |_| {
                                        sender.input(AppInput::ShowAboutDialog);
                                    },
                                },
                            },
                        },
                    },
                },

                gtk::Box {
                    set_margin_all: 10,
                    set_orientation: gtk::Orientation::Vertical,

                    #[transition = "SlideLeftRight"]
                    match &model.current_view {
                        View::Loading(what) => gtk::Box {
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,
                            set_vexpand: true,
                            set_orientation: gtk::Orientation::Vertical,

                            gtk::Spinner {
                                start: (),
                                set_size_request: (50, 50),
                            },

                            gtk::Label {
                                #[watch]
                                set_label: what,
                            },
                        },
                        View::Configuration { reward_address, node_path, farms, .. } => gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,

                            gtk::ScrolledWindow {
                                set_vexpand: true,

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,

                                    gtk::ListBox {
                                        gtk::ListBoxRow {
                                            set_activatable: false,
                                            set_margin_bottom: 10,
                                            set_selectable: false,

                                            gtk::Box {
                                                set_orientation: gtk::Orientation::Vertical,
                                                set_spacing: 10,

                                                gtk::Label {
                                                    add_css_class: "heading",
                                                    set_halign: gtk::Align::Start,
                                                    set_label: "Rewards address",
                                                },

                                                gtk::Entry {
                                                    connect_activate[sender] => move |entry| {
                                                        sender.input(AppInput::Configuration(
                                                            ConfigurationEvent::RewardAddressChanged(entry.text().into())
                                                        ));
                                                    },
                                                    connect_changed[sender] => move |entry| {
                                                        sender.input(AppInput::Configuration(
                                                            ConfigurationEvent::RewardAddressChanged(entry.text().into())
                                                        ));
                                                    },
                                                    set_placeholder_text: Some(
                                                        "stB4S14whneyomiEa22Fu2PzVoibMB7n5PvBFUwafbCbRkC1K",
                                                    ),
                                                    #[watch]
                                                    set_secondary_icon_name: reward_address.icon(),
                                                    set_secondary_icon_activatable: false,
                                                    set_secondary_icon_sensitive: false,
                                                    #[track = "reward_address.unknown()"]
                                                    set_text: reward_address,
                                                    set_tooltip_markup: Some(
                                                        "Use Subwallet or polkadot{.js} extension or any other \
                                                        Substrate wallet to create it first (address for any Substrate \
                                                        chain in SS58 format works)"
                                                    ),
                                                },
                                            },
                                        },
                                        gtk::ListBoxRow {
                                            set_activatable: false,
                                            set_margin_bottom: 10,
                                            set_selectable: false,

                                            gtk::Box {
                                                set_orientation: gtk::Orientation::Vertical,
                                                set_spacing: 10,

                                                gtk::Label {
                                                    add_css_class: "heading",
                                                    set_halign: gtk::Align::Start,
                                                    set_label: "Node path",
                                                },

                                                gtk::Box {
                                                    add_css_class: "linked",

                                                    gtk::Entry {
                                                        set_can_focus: false,
                                                        set_editable: false,
                                                        set_hexpand: true,
                                                        set_placeholder_text: Some(
                                                            if cfg!(windows) {
                                                                "D:\\subspace-node"
                                                            } else {
                                                                "/media/subspace-node"
                                                            },
                                                        ),
                                                        #[watch]
                                                        set_secondary_icon_name: node_path.icon(),
                                                        set_secondary_icon_activatable: false,
                                                        set_secondary_icon_sensitive: false,
                                                        #[watch]
                                                        set_text: node_path.display().to_string().as_str(),
                                                        set_tooltip_markup: Some(
                                                            "Absolute path where node files will be stored, prepare to \
                                                            dedicate at least 100GiB of space for it, good quality SSD \
                                                            recommended"
                                                        ),
                                                    },

                                                    gtk::Button {
                                                        connect_clicked[sender] => move |_button| {
                                                            sender.input(AppInput::Configuration(
                                                                ConfigurationEvent::OpenDirectory(DirectoryKind::NodePath)
                                                            ));
                                                        },

                                                        gtk::Box {
                                                            set_spacing: 10,

                                                            gtk::Image {
                                                                set_icon_name: Some("folder-new-symbolic"),
                                                            },

                                                            gtk::Label {
                                                                set_label: "Select",
                                                            },
                                                        },
                                                    },
                                                },
                                            },
                                        },
                                        // TODO: Support more than one farm
                                        gtk::ListBoxRow {
                                            set_activatable: false,
                                            set_margin_bottom: 10,
                                            set_selectable: false,

                                            gtk::Box {
                                                set_orientation: gtk::Orientation::Vertical,
                                                set_spacing: 10,

                                                gtk::Label {
                                                    add_css_class: "heading",
                                                    set_halign: gtk::Align::Start,
                                                    set_label: "Path to farm and its size",
                                                },

                                                gtk::Box {
                                                    set_spacing: 10,

                                                    gtk::Box {
                                                        add_css_class: "linked",

                                                        gtk::Entry {
                                                            set_can_focus: false,
                                                            set_editable: false,
                                                            set_hexpand: true,
                                                            set_placeholder_text: Some(
                                                                if cfg!(windows) {
                                                                    "D:\\subspace-farm"
                                                                } else {
                                                                    "/media/subspace-farm"
                                                                },
                                                            ),
                                                            #[watch]
                                                            set_secondary_icon_name: farms.get(0).map(|farm| farm.path.icon()).unwrap_or_default(),
                                                            set_secondary_icon_activatable: false,
                                                            set_secondary_icon_sensitive: false,
                                                            #[watch]
                                                            set_text: farms.get(0).map(|farm| farm.path.display().to_string()).unwrap_or_default().as_str(),
                                                            set_tooltip_markup: Some(
                                                                "Absolute path where farm files will be stored, any \
                                                                SSD works, high endurance not necessary"
                                                            ),
                                                        },

                                                        gtk::Button {
                                                            connect_clicked[sender] => move |_button| {
                                                                sender.input(AppInput::Configuration(
                                                                    ConfigurationEvent::OpenDirectory(
                                                                        DirectoryKind::FarmPath(0)
                                                                    )
                                                                ));
                                                            },

                                                            gtk::Box {
                                                                set_spacing: 10,

                                                                gtk::Image {
                                                                    set_icon_name: Some("folder-new-symbolic"),
                                                                },

                                                                gtk::Label {
                                                                    set_label: "Select",
                                                                },
                                                            },
                                                        },
                                                    },

                                                    gtk::Entry {
                                                        connect_activate[sender] => move |entry| {
                                                            sender.input(AppInput::Configuration(
                                                                ConfigurationEvent::FarmSizeChanged {
                                                                    farm_index: 0,
                                                                    size: entry.text().into()
                                                                }
                                                            ));
                                                        },
                                                        connect_changed[sender] => move |entry| {
                                                            sender.input(AppInput::Configuration(
                                                                ConfigurationEvent::FarmSizeChanged {
                                                                    farm_index: 0,
                                                                    size: entry.text().into()
                                                                }
                                                            ));
                                                        },
                                                        set_placeholder_text: Some(
                                                            "4T, 2.5TB, 500GiB, etc.",
                                                        ),
                                                        #[watch]
                                                        set_secondary_icon_name: farms.get(0).map(|farm| farm.size.icon()).unwrap_or_default(),
                                                        set_secondary_icon_activatable: false,
                                                        set_secondary_icon_sensitive: false,
                                                        #[track = "farms.get(0).map(|farm| farm.size.unknown()).unwrap_or_default()"]
                                                        set_text: farms.get(0).map(|farm| farm.size.as_str()).unwrap_or_default(),
                                                        set_tooltip_markup: Some(
                                                            "Size of the farm in whichever units you prefer, any \
                                                            amount of space above 2 GB works"
                                                        ),
                                                    },
                                                },
                                            },
                                        },
                                    },

                                    gtk::Box {
                                        set_halign: gtk::Align::End,

                                        gtk::Button {
                                            add_css_class: "suggested-action",
                                            connect_clicked[sender] => move |_button| {
                                                sender.input(AppInput::Configuration(
                                                    ConfigurationEvent::Start
                                                ));
                                            },
                                            set_margin_top: 20,
                                            #[watch]
                                            set_sensitive: {
                                                // TODO
                                                reward_address.valid()
                                                    && node_path.valid()
                                                    && !farms.is_empty()
                                                    && farms.iter().all(|farm| {
                                                        farm.path.valid() && farm.size.valid()
                                                    })
                                            },

                                            gtk::Label {
                                                set_label: "Start",
                                                set_margin_all: 10,
                                            },
                                        },
                                    },
                                },
                            },
                        },
                        View::Running { node_state, farmer_state } => gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,

                            gtk::Box {
                                set_height_request: 100,
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 10,

                                gtk::Label {
                                    add_css_class: "heading",
                                    set_halign: gtk::Align::Start,
                                    set_label: "Consensus node",
                                },

                                // TODO: Match only because `if let Some(x) = y` is not yet supported here: https://github.com/Relm4/Relm4/issues/582
                                #[transition = "SlideUpDown"]
                                match &node_state.sync_state {
                                    Some(sync_state) => gtk::Box {
                                        set_orientation: gtk::Orientation::Vertical,
                                        set_spacing: 10,

                                        gtk::Box {
                                            gtk::Label {
                                                set_halign: gtk::Align::Start,

                                                #[watch]
                                                set_label: &{
                                                    let kind = match sync_state.kind {
                                                        SyncKind::Dsn => "Syncing from DSN",
                                                        SyncKind::Regular => "Regular sync",
                                                    };

                                                    format!(
                                                        "{} #{}/{}{}",
                                                        kind,
                                                        node_state.best_block_number,
                                                        sync_state.target,
                                                        sync_state.speed
                                                            .map(|speed| format!(", {:.2} blocks/s", speed))
                                                            .unwrap_or_default(),
                                                    )
                                                },
                                            },

                                            gtk::Spinner {
                                                set_margin_start: 5,
                                                start: (),
                                            },
                                        },

                                        gtk::ProgressBar {
                                            #[watch]
                                            set_fraction: node_state.best_block_number as f64 / sync_state.target as f64,
                                        },
                                    },
                                    None => gtk::Box {
                                        gtk::Label {
                                            #[watch]
                                            set_label: &format!("Synced, best block #{}", node_state.best_block_number),
                                        }
                                    },
                                },
                            },

                            gtk::Separator {
                                set_margin_all: 10,
                            },

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 10,
                                set_valign: gtk::Align::Start,

                                gtk::Label {
                                    add_css_class: "heading",
                                    set_halign: gtk::Align::Start,
                                    set_label: "Farmer",
                                },

                                // TODO: Render all farms, not just the first one
                                // TODO: Match only because `if let Some(x) = y` is not yet supported here: https://github.com/Relm4/Relm4/issues/582
                                #[transition = "SlideUpDown"]
                                match (&farmer_state.plotting_state[0], node_state.sync_state.is_none()) {
                                    (Some(plotting_state), true) => gtk::Box {
                                        set_orientation: gtk::Orientation::Vertical,

                                        gtk::Box {
                                            set_spacing: 10,

                                            gtk::Label {
                                                set_halign: gtk::Align::Start,

                                                #[watch]
                                                set_label: &{
                                                    let kind = match plotting_state.kind {
                                                        PlottingKind::Initial => "Initial plotting, not farming",
                                                        PlottingKind::Replotting => "Replotting, farming",
                                                    };

                                                    format!(
                                                        "{} {:.2}%{}",
                                                        kind,
                                                        plotting_state.progress,
                                                        plotting_state.speed
                                                            .map(|speed| format!(", {:.2} sectors/h", 3600.0 / speed))
                                                            .unwrap_or_default(),
                                                    )
                                                },
                                                set_tooltip: "Farming starts after initial plotting is complete",
                                            },

                                            gtk::Spinner {
                                                set_margin_start: 5,
                                                start: (),
                                            },
                                        },

                                        gtk::ProgressBar {
                                            #[watch]
                                            set_fraction: plotting_state.progress as f64 / 100.0,
                                        },
                                    },
                                    (None, true) => gtk::Box {
                                        gtk::Label {
                                            #[watch]
                                            set_label: "Farming",
                                        }
                                    },
                                    _ => gtk::Box {
                                        gtk::Label {
                                            #[watch]
                                            set_label: "Waiting for node to sync first",
                                        }
                                    },
                                },
                            },
                        }
                        View::Stopped(Some(error)) => {
                            // TODO: Better error handling
                            gtk::Label {
                                #[watch]
                                set_label: &format!("Stopped with error: {error}"),
                            }
                        }
                        View::Stopped(None) => {
                            gtk::Label {
                                set_label: "Stopped 🛑",
                            }
                        }
                        View::Error(error) => {
                            // TODO: Better error handling
                            gtk::Label {
                                #[watch]
                                set_label: &format!("Error: {error}"),
                            }
                        },
                    },
                },
            }
        }
    }

    async fn init(
        _init: (),
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let (action_sender, action_receiver) = mpsc::channel(1);
        let (notification_sender, mut notification_receiver) = mpsc::channel(100);

        // Create backend in dedicated thread
        tokio::task::spawn_blocking(move || {
            Handle::current().block_on(backend::create(action_receiver, notification_sender));
        });

        // Forward backend notifications as application inputs
        tokio::spawn({
            let sender = sender.clone();

            async move {
                while let Some(notification) = notification_receiver.next().await {
                    sender.input(AppInput::BackendNotification(notification));
                }
            }
        });

        let open_dialog = OpenDialog::builder()
            .transient_for_native(&root)
            .launch(OpenDialogSettings {
                folder_mode: true,
                accept_label: "Select".to_string(),
                ..OpenDialogSettings::default()
            })
            .forward(sender.input_sender(), |response| match response {
                OpenDialogResponse::Accept(path) => {
                    AppInput::Configuration(ConfigurationEvent::DirectorySelected(path))
                }
                OpenDialogResponse::Cancel => AppInput::Ignore,
            });

        let about_dialog = gtk::AboutDialog::builder()
            .title("About")
            .program_name("Space Acres")
            .version(env!("CARGO_PKG_VERSION"))
            .authors(env!("CARGO_PKG_AUTHORS").split(':').collect::<Vec<_>>())
            // TODO: Use https://gitlab.gnome.org/GNOME/gtk/-/merge_requests/6643 once available
            .license("Zero-Clause BSD: https://opensource.org/license/0bsd/")
            .website(env!("CARGO_PKG_REPOSITORY"))
            .comments(env!("CARGO_PKG_DESCRIPTION"))
            .logo(&gtk::gdk::Texture::for_pixbuf(
                &gtk::gdk_pixbuf::Pixbuf::from_read(ABOUT_IMAGE)
                    .expect("Statically correct image; qed"),
            ))
            .transient_for(&root)
            .build();
        about_dialog.connect_close_request(|about_dialog| {
            about_dialog.hide();
            gtk::glib::Propagation::Stop
        });

        let mut model = App {
            current_view: View::Loading(String::new()),
            backend_action_sender: action_sender,
            open_dialog,
            // Hack to initialize a field before this data structure is used
            menu_popover: gtk::Popover::default(),
            about_dialog,
        };

        let widgets = view_output!();

        model.menu_popover = widgets.menu_popover.clone();

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        input: Self::Input,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match input {
            AppInput::BackendNotification(notification) => {
                self.process_backend_notification(notification);
            }
            AppInput::Configuration(event) => {
                self.process_configuration_event(event).await;
            }
            AppInput::ShowAboutDialog => {
                self.menu_popover.hide();
                self.about_dialog.show();
            }
            AppInput::Ignore => {
                // Ignore
            }
        }
    }
}

impl App {
    fn process_backend_notification(&mut self, notification: BackendNotification) {
        match notification {
            // TODO: Render progress
            BackendNotification::Loading { step, progress: _ } => {
                self.set_loading_string(match step {
                    LoadingStep::LoadingConfiguration => "Loading configuration...",
                    LoadingStep::ReadingConfiguration => "Reading configuration...",
                    LoadingStep::ConfigurationReadSuccessfully { .. } => {
                        "Configuration read successfully"
                    }
                    LoadingStep::CheckingConfiguration => "Checking configuration...",
                    LoadingStep::ConfigurationIsValid => "Configuration is valid",
                    LoadingStep::DecodingChainSpecification => "Decoding chain specification...",
                    LoadingStep::DecodedChainSpecificationSuccessfully => {
                        "Decoded chain specification successfully"
                    }
                    LoadingStep::CheckingNodePath => "Checking node path...",
                    LoadingStep::CreatingNodePath => "Creating node path...",
                    LoadingStep::NodePathReady => "Node path ready",
                    LoadingStep::PreparingNetworkingStack => "Preparing networking stack...",
                    LoadingStep::ReadingNetworkKeypair => "Reading network keypair...",
                    LoadingStep::GeneratingNetworkKeypair => "Generating network keypair...",
                    LoadingStep::WritingNetworkKeypair => "Writing network keypair to disk...",
                    LoadingStep::InstantiatingNetworkingStack => {
                        "Instantiating networking stack..."
                    }
                    LoadingStep::NetworkingStackCreatedSuccessfully => {
                        "Networking stack created successfully"
                    }
                    LoadingStep::CreatingConsensusNode => "Creating consensus node...",
                    LoadingStep::ConsensusNodeCreatedSuccessfully => {
                        "Consensus node created successfully"
                    }
                    LoadingStep::CreatingFarmer => "Creating farmer...",
                    LoadingStep::FarmerCreatedSuccessfully => "Farmer created successfully",
                });
            }
            BackendNotification::NotConfigured => {
                // TODO: Welcome screen first
                self.current_view = View::Configuration {
                    reward_address: MaybeValid::Unknown(String::new()),
                    node_path: MaybeValid::Unknown(PathBuf::new()),
                    farms: Vec::new(),
                    pending_directory_selection: None,
                };
            }
            BackendNotification::ConfigurationIsInvalid { .. } => {
                // TODO: Toast with configuration error, render old values with corresponding validity status once
                //  notification has that information
                self.current_view = View::Configuration {
                    reward_address: MaybeValid::Unknown(String::new()),
                    node_path: MaybeValid::Unknown(PathBuf::new()),
                    farms: Vec::new(),
                    pending_directory_selection: None,
                };
            }
            BackendNotification::Running {
                config,
                best_block_number,
            } => {
                let num_farms = config.farms.len();
                self.current_view = View::Running {
                    node_state: NodeState {
                        best_block_number,
                        sync_state: None,
                    },
                    farmer_state: FarmerState {
                        plotting_state: vec![None; num_farms],
                    },
                };
            }
            BackendNotification::Node(node_notification) => {
                if let View::Running { node_state, .. } = &mut self.current_view {
                    match node_notification {
                        NodeNotification::Syncing(sync_state) => {
                            node_state.sync_state = Some(sync_state);
                        }
                        NodeNotification::Synced => {
                            node_state.sync_state = None;
                        }
                        NodeNotification::BlockImported { number } => {
                            node_state.best_block_number = number;
                        }
                    }
                }
            }
            BackendNotification::Farmer(farmer_notification) => {
                if let View::Running { farmer_state, .. } = &mut self.current_view {
                    match farmer_notification {
                        FarmerNotification::Plotting { farm_index, state } => {
                            if let Some(plotting_state) =
                                farmer_state.plotting_state.get_mut(farm_index)
                            {
                                plotting_state.replace(state);
                            } else {
                                warn!(%farm_index, "Unexpected plotting farm index");
                            }
                        }
                        FarmerNotification::Plotted { farm_index } => {
                            if let Some(plotting_state) =
                                farmer_state.plotting_state.get_mut(farm_index)
                            {
                                plotting_state.take();
                            } else {
                                warn!(%farm_index, "Unexpected plotted farm index");
                            }
                        }
                    }
                }
            }
            BackendNotification::Stopped { error } => {
                self.current_view = View::Stopped(error);
            }
            BackendNotification::IrrecoverableError { error } => {
                self.current_view = View::Error(error);
            }
        }
    }

    fn set_loading_string(&mut self, s: &'static str) {
        self.current_view = View::Loading(s.to_string());
    }

    async fn process_configuration_event(&mut self, event: ConfigurationEvent) {
        if let View::Configuration {
            reward_address,
            node_path,
            farms,
            pending_directory_selection,
        } = &mut self.current_view
        {
            match event {
                ConfigurationEvent::RewardAddressChanged(new_reward_address) => {
                    let new_reward_address = new_reward_address.trim();
                    *reward_address = if parse_ss58_reward_address(new_reward_address).is_ok() {
                        MaybeValid::Valid(new_reward_address.to_string())
                    } else {
                        MaybeValid::Invalid(new_reward_address.to_string())
                    };
                }
                ConfigurationEvent::OpenDirectory(directory_kind) => {
                    pending_directory_selection.replace(directory_kind);
                    self.open_dialog.emit(OpenDialogMsg::Open);
                }
                ConfigurationEvent::DirectorySelected(path) => {
                    match pending_directory_selection.take() {
                        Some(DirectoryKind::NodePath) => {
                            *node_path = MaybeValid::Valid(path);
                        }
                        Some(DirectoryKind::FarmPath(farm_index)) => {
                            if let Some(farm) = farms.get_mut(farm_index) {
                                farm.path = MaybeValid::Valid(path);
                            } else {
                                farms.push(DiskFarm {
                                    path: MaybeValid::Valid(path),
                                    size: Default::default(),
                                })
                            }
                        }
                        None => {
                            warn!(
                                directory = %path.display(),
                                "Directory selected, but no pending selection found",
                            );
                        }
                    }
                }
                ConfigurationEvent::FarmSizeChanged { farm_index, size } => {
                    let size = if ByteSize::from_str(&size)
                        .map(|size| size.as_u64() >= MIN_FARM_SIZE)
                        .unwrap_or_default()
                    {
                        MaybeValid::Valid(size)
                    } else {
                        MaybeValid::Invalid(size)
                    };
                    if let Some(farm) = farms.get_mut(farm_index) {
                        farm.size = size;
                    } else {
                        farms.push(DiskFarm {
                            path: MaybeValid::default(),
                            size,
                        })
                    }
                }
                ConfigurationEvent::Start => {
                    let config = RawConfig::V0 {
                        reward_address: String::clone(reward_address),
                        node_path: PathBuf::clone(node_path),
                        farms: farms
                            .iter()
                            .map(|farm| Farm {
                                path: PathBuf::clone(&farm.path),
                                size: String::clone(&farm.size),
                            })
                            .collect(),
                    };
                    if let Err(error) = self
                        .backend_action_sender
                        .send(BackendAction::NewConfig { config })
                        .await
                    {
                        self.current_view = View::Error(anyhow::anyhow!(
                            "Failed to send config to backend: {error}"
                        ));
                    }
                }
            }
        }
    }
}

fn main() {
    // TODO: Log into files
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                // TODO: Workaround for https://github.com/tokio-rs/tracing/issues/2214, also on
                //  Windows terminal doesn't support the same colors as bash does
                .with_ansi(if cfg!(windows) {
                    false
                } else {
                    supports_color::on(supports_color::Stream::Stderr).is_some()
                })
                .with_filter(
                    EnvFilter::builder()
                        .with_default_directive(LevelFilter::INFO.into())
                        .from_env_lossy(),
                ),
        )
        .init();

    // The default in `relm4` is `1`, set this back to Tokio's default
    RELM_THREADS
        .set(
            available_parallelism()
                .map(|cores| cores.get())
                .unwrap_or(1),
        )
        .expect("The first thing in the app, is not set; qed");

    let app = RelmApp::new("network.subspace.space_acres");

    set_global_css(GLOBAL_CSS);

    // Prefer dark theme in cross-platform way if environment is configured that way
    if let Some(settings) = gtk::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(!matches!(
            dark_light::detect(),
            dark_light::Mode::Light
        ));
    }

    app.run_async::<App>(());
}
