use crate::backend::config::{Farm, RawConfig};
use bytesize::ByteSize;
use gtk::prelude::*;
use relm4::prelude::*;
use relm4_components::open_dialog::{
    OpenDialog, OpenDialogMsg, OpenDialogResponse, OpenDialogSettings,
};
use relm4_icons::icon_name;
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;
use subspace_farmer::utils::ss58::parse_ss58_reward_address;
use tracing::{debug, warn};

// 2 GB
const MIN_FARM_SIZE: u64 = 1000 * 1000 * 1000 * 2;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DirectoryKind {
    NodePath,
    FarmPath(usize),
}

#[derive(Debug)]
pub enum ConfigurationInput {
    RewardAddressChanged(String),
    OpenDirectory(DirectoryKind),
    DirectorySelected(PathBuf),
    FarmSizeChanged { farm_index: usize, size: String },
    Reconfigure(RawConfig),
    Start,
    Cancel,
    Save,
    Ignore,
}

#[derive(Debug)]
pub enum ConfigurationOutput {
    StartWithNewConfig(RawConfig),
    ConfigUpdate(RawConfig),
    Close,
}

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
            MaybeValid::Valid(_) => Some(icon_name::CHECKMARK),
            MaybeValid::Invalid(_) => Some(icon_name::CROSS),
        }
    }
}

#[derive(Debug, Default)]
struct DiskFarm {
    path: MaybeValid<PathBuf>,
    size: MaybeValid<String>,
}

#[derive(Debug)]
pub struct ConfigurationView {
    reward_address: MaybeValid<String>,
    node_path: MaybeValid<PathBuf>,
    farms: Vec<DiskFarm>,
    pending_directory_selection: Option<DirectoryKind>,
    open_dialog: Controller<OpenDialog>,
    reconfiguration: bool,
}

#[relm4::component(pub)]
impl Component for ConfigurationView {
    type Init = gtk::Window;
    type Input = ConfigurationInput;
    type Output = ConfigurationOutput;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            gtk::ScrolledWindow {
                set_vexpand: true,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 20,

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
                                        sender.input(ConfigurationInput::RewardAddressChanged(
                                            entry.text().into()
                                        ));
                                    },
                                    connect_changed[sender] => move |entry| {
                                        sender.input(ConfigurationInput::RewardAddressChanged(
                                            entry.text().into()
                                        ));
                                    },
                                    set_placeholder_text: Some(
                                        "stB4S14whneyomiEa22Fu2PzVoibMB7n5PvBFUwafbCbRkC1K",
                                    ),
                                    set_primary_icon_name: Some(icon_name::WALLET2),
                                    set_primary_icon_activatable: false,
                                    set_primary_icon_sensitive: false,
                                    #[watch]
                                    set_secondary_icon_name: model.reward_address.icon(),
                                    set_secondary_icon_activatable: false,
                                    set_secondary_icon_sensitive: false,
                                    #[track = "model.reward_address.unknown()"]
                                    set_text: &model.reward_address,
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
                                        set_primary_icon_name: Some(icon_name::SSD),
                                        set_primary_icon_activatable: false,
                                        set_primary_icon_sensitive: false,
                                        #[watch]
                                        set_secondary_icon_name: model.node_path.icon(),
                                        set_secondary_icon_activatable: false,
                                        set_secondary_icon_sensitive: false,
                                        #[watch]
                                        set_text: model.node_path.display().to_string().as_str(),
                                        set_tooltip_markup: Some(
                                            "Absolute path where node files will be stored, prepare to \
                                            dedicate at least 100 GiB of space for it, good quality SSD \
                                            recommended"
                                        ),
                                    },

                                    gtk::Button {
                                        connect_clicked => ConfigurationInput::OpenDirectory(
                                            DirectoryKind::NodePath
                                        ),
                                        set_label: "Select",
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
                                            set_primary_icon_name: Some(icon_name::SSD),
                                            set_primary_icon_activatable: false,
                                            set_primary_icon_sensitive: false,
                                            #[watch]
                                            set_secondary_icon_name: model.farms.get(0).map(|farm| farm.path.icon()).unwrap_or_default(),
                                            set_secondary_icon_activatable: false,
                                            set_secondary_icon_sensitive: false,
                                            #[watch]
                                            set_text: model.farms.get(0).map(|farm| farm.path.display().to_string()).unwrap_or_default().as_str(),
                                            set_tooltip_markup: Some(
                                                "Absolute path where farm files will be stored, any \
                                                SSD works, high endurance not necessary"
                                            ),
                                        },

                                        gtk::Button {
                                            connect_clicked => ConfigurationInput::OpenDirectory(
                                                DirectoryKind::FarmPath(0)
                                            ),
                                            set_label: "Select",
                                        },
                                    },

                                    gtk::Entry {
                                        connect_activate[sender] => move |entry| {
                                            sender.input(ConfigurationInput::FarmSizeChanged {
                                                farm_index: 0,
                                                size: entry.text().into()
                                            });
                                        },
                                        connect_changed[sender] => move |entry| {
                                            sender.input(ConfigurationInput::FarmSizeChanged {
                                                farm_index: 0,
                                                size: entry.text().into()
                                            });
                                        },
                                        set_placeholder_text: Some(
                                            "4T, 2.5TB, 500GiB, etc.",
                                        ),
                                        set_primary_icon_name: Some(icon_name::SIZE_HORIZONTALLY),
                                        set_primary_icon_activatable: false,
                                        set_primary_icon_sensitive: false,
                                        #[watch]
                                        set_secondary_icon_name: model.farms.get(0).map(|farm| farm.size.icon()).unwrap_or_default(),
                                        set_secondary_icon_activatable: false,
                                        set_secondary_icon_sensitive: false,
                                        #[track = "model.farms.get(0).map(|farm| farm.size.unknown()).unwrap_or_default()"]
                                        set_text: model.farms.get(0).map(|farm| farm.size.as_str()).unwrap_or_default(),
                                        set_tooltip_markup: Some(
                                            "Size of the farm in whichever units you prefer, any \
                                            amount of space above 2 GB works"
                                        ),
                                    },
                                },
                            },
                        },
                    },

                    if model.reconfiguration {
                        gtk::Box {
                            set_halign: gtk::Align::End,
                            set_spacing: 10,

                            gtk::Button {
                                connect_clicked => ConfigurationInput::Cancel,

                                gtk::Label {
                                    set_label: "Cancel",
                                    set_margin_all: 10,
                                },
                            },

                            gtk::Button {
                                add_css_class: "suggested-action",
                                connect_clicked => ConfigurationInput::Save,
                                #[watch]
                                set_sensitive:
                                    model.reward_address.valid()
                                        && model.node_path.valid()
                                        && !model.farms.is_empty()
                                        && model.farms.iter().all(|farm| {
                                            farm.path.valid() && farm.size.valid()
                                        }),

                                gtk::Label {
                                    set_label: "Save",
                                    set_margin_all: 10,
                                },
                            },
                        }
                    } else {
                        gtk::Box {
                            set_halign: gtk::Align::End,

                            gtk::Button {
                                add_css_class: "suggested-action",
                                connect_clicked => ConfigurationInput::Start,
                                #[watch]
                                set_sensitive:
                                    model.reward_address.valid()
                                        && model.node_path.valid()
                                        && !model.farms.is_empty()
                                        && model.farms.iter().all(|farm| {
                                            farm.path.valid() && farm.size.valid()
                                        }),

                                gtk::Label {
                                    set_label: "Start",
                                    set_margin_all: 10,
                                },
                            },
                        }
                    },
                },
            },
        }
    }

    fn init(
        parent_root: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let open_dialog = OpenDialog::builder()
            .transient_for_native(&parent_root)
            .launch(OpenDialogSettings {
                folder_mode: true,
                accept_label: "Select".to_string(),
                ..OpenDialogSettings::default()
            })
            .forward(sender.input_sender(), |response| match response {
                OpenDialogResponse::Accept(path) => ConfigurationInput::DirectorySelected(path),
                OpenDialogResponse::Cancel => ConfigurationInput::Ignore,
            });

        let model = Self {
            reward_address: Default::default(),
            node_path: Default::default(),
            farms: Default::default(),
            pending_directory_selection: Default::default(),
            open_dialog,
            reconfiguration: false,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        self.process_input(input, sender);
    }
}

impl ConfigurationView {
    fn process_input(&mut self, input: ConfigurationInput, sender: ComponentSender<Self>) {
        match input {
            ConfigurationInput::RewardAddressChanged(new_reward_address) => {
                let new_reward_address = new_reward_address.trim();
                self.reward_address = if parse_ss58_reward_address(new_reward_address).is_ok() {
                    MaybeValid::Valid(new_reward_address.to_string())
                } else {
                    MaybeValid::Invalid(new_reward_address.to_string())
                };
            }
            ConfigurationInput::OpenDirectory(directory_kind) => {
                self.pending_directory_selection.replace(directory_kind);
                self.open_dialog.emit(OpenDialogMsg::Open);
            }
            ConfigurationInput::DirectorySelected(path) => {
                match self.pending_directory_selection.take() {
                    Some(DirectoryKind::NodePath) => {
                        self.node_path = MaybeValid::Valid(path);
                    }
                    Some(DirectoryKind::FarmPath(farm_index)) => {
                        if let Some(farm) = self.farms.get_mut(farm_index) {
                            farm.path = MaybeValid::Valid(path);
                        } else {
                            self.farms.push(DiskFarm {
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
            ConfigurationInput::FarmSizeChanged { farm_index, size } => {
                let size = if ByteSize::from_str(&size)
                    .map(|size| size.as_u64() >= MIN_FARM_SIZE)
                    .unwrap_or_default()
                {
                    MaybeValid::Valid(size)
                } else {
                    MaybeValid::Invalid(size)
                };
                if let Some(farm) = self.farms.get_mut(farm_index) {
                    farm.size = size;
                } else {
                    self.farms.push(DiskFarm {
                        path: MaybeValid::default(),
                        size,
                    })
                }
            }
            ConfigurationInput::Reconfigure(raw_config) => {
                // `Unknown` is a hack to make it actually render the first time
                self.reward_address = MaybeValid::Unknown(raw_config.reward_address().to_string());
                self.node_path = MaybeValid::Valid(raw_config.node_path().clone());
                self.farms = raw_config
                    .farms()
                    .iter()
                    .map(|farm| DiskFarm {
                        path: MaybeValid::Valid(farm.path.clone()),
                        // `Unknown` is a hack to make it actually render the first time
                        size: MaybeValid::Unknown(farm.size.clone()),
                    })
                    .collect();
                self.reconfiguration = true;
            }
            ConfigurationInput::Start => {
                if sender
                    .output(ConfigurationOutput::StartWithNewConfig(
                        self.create_raw_config(),
                    ))
                    .is_err()
                {
                    debug!("Failed to send ConfigurationOutput::StartWithNewConfig");
                }
            }
            ConfigurationInput::Cancel => {
                if sender.output(ConfigurationOutput::Close).is_err() {
                    debug!("Failed to send ConfigurationOutput::Close");
                }
            }
            ConfigurationInput::Save => {
                if sender
                    .output(ConfigurationOutput::ConfigUpdate(self.create_raw_config()))
                    .is_err()
                {
                    debug!("Failed to send ConfigurationOutput::ConfigUpdate");
                }
            }
            ConfigurationInput::Ignore => {
                // Ignore
            }
        }
    }

    /// Create raw config from own state
    fn create_raw_config(&self) -> RawConfig {
        RawConfig::V0 {
            reward_address: String::clone(&self.reward_address),
            node_path: PathBuf::clone(&self.node_path),
            farms: self
                .farms
                .iter()
                .map(|farm| Farm {
                    path: PathBuf::clone(&farm.path),
                    size: String::clone(&farm.size),
                })
                .collect(),
        }
    }
}
