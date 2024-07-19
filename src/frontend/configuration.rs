mod farm;

use crate::backend::config::{NetworkConfiguration, RawConfig};
use crate::frontend::configuration::farm::{
    FarmWidget, FarmWidgetInit, FarmWidgetInput, FarmWidgetOutput,
};
use gtk::prelude::*;
use relm4::factory::FactoryVecDeque;
use relm4::prelude::*;
use relm4_components::open_dialog::{
    OpenDialog, OpenDialogMsg, OpenDialogResponse, OpenDialogSettings,
};
use relm4_icons::icon_name;
use std::ops::Deref;
use std::path::PathBuf;
use subspace_farmer::utils::ss58::parse_ss58_reward_address;
use tracing::{debug, warn};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DirectoryKind {
    NodePath,
    FarmPath(DynamicIndex),
}

#[derive(Debug)]
pub enum ConfigurationInput {
    AddFarm,
    RewardAddressChanged(String),
    OpenDirectory(DirectoryKind),
    DirectorySelected(PathBuf),
    SubstratePortChanged(u16),
    SubspacePortChanged(u16),
    FasterNetworkingChanged(bool),
    Delete(DynamicIndex),
    Reconfigure(RawConfig),
    Start,
    Back,
    Cancel,
    Save,
    Ignore,
}

#[derive(Debug)]
pub enum ConfigurationOutput {
    StartWithNewConfig(RawConfig),
    ConfigUpdate(RawConfig),
    Back,
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
        let (Self::Unknown(inner) | Self::Valid(inner) | Self::Invalid(inner)) = self;

        inner
    }
}

impl<T> MaybeValid<T> {
    fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }

    fn is_valid(&self) -> bool {
        matches!(self, Self::Valid(_))
    }

    fn icon(&self) -> Option<&'static str> {
        match self {
            Self::Unknown(_) => None,
            Self::Valid(_) => Some(icon_name::CHECKMARK),
            Self::Invalid(_) => Some(icon_name::CROSS),
        }
    }
}

#[derive(Debug)]
struct NetworkConfigurationWrapper {
    substrate_port: MaybeValid<u16>,
    subspace_port: MaybeValid<u16>,
    faster_networking: bool,
}

impl Default for NetworkConfigurationWrapper {
    fn default() -> Self {
        Self::from(NetworkConfiguration::default())
    }
}

impl From<NetworkConfiguration> for NetworkConfigurationWrapper {
    fn from(config: NetworkConfiguration) -> Self {
        // `Unknown` is a hack to make it actually render the first time
        Self {
            substrate_port: MaybeValid::Unknown(config.substrate_port),
            subspace_port: MaybeValid::Unknown(config.subspace_port),
            faster_networking: config.faster_networking,
        }
    }
}

#[derive(Debug)]
pub struct ConfigurationView {
    reward_address: MaybeValid<String>,
    node_path: MaybeValid<PathBuf>,
    farms: FactoryVecDeque<FarmWidget>,
    network_configuration: NetworkConfigurationWrapper,
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
                        gtk::ListBoxRow {
                            set_activatable: false,
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
                                    #[track = "model.reward_address.is_unknown()"]
                                    set_text: &model.reward_address,
                                    set_tooltip_markup: Some(
                                        "Use Subwallet or polkadot{.js} extension or any other \
                                        Substrate wallet to create it first (address for any Substrate \
                                        chain in SS58 format works)"
                                    ),
                                },
                            },
                        },
                    },

                    // TODO: This should be the same list box as above, but then farms will
                    //  unfortunately render before other fields
                    #[local_ref]
                    configuration_list_box -> gtk::ListBox {
                    },

                    gtk::Expander {
                        set_label: Some("Advanced configuration"),

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 10,

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_margin_top: 10,
                                set_spacing: 10,

                                gtk::Label {
                                    add_css_class: "heading",
                                    set_halign: gtk::Align::Start,
                                    set_label: "Network configuration",
                                },

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 10,

                                    gtk::Box {
                                        set_spacing: 10,

                                        gtk::Label {
                                            set_label: "Substrate (blockchain) P2P port (TCP):"
                                        },
                                        gtk::SpinButton {
                                            connect_value_changed[sender] => move |entry| {
                                                sender.input(ConfigurationInput::SubstratePortChanged(
                                                    entry.value().round() as u16
                                                ));
                                            },
                                            set_adjustment: &gtk::Adjustment::new(
                                                0.0,
                                                0.0,
                                                u16::MAX as f64,
                                                1.0,
                                                0.0,
                                                0.0,
                                            ),
                                            set_tooltip: &format!(
                                                "Default port number is {}",
                                                NetworkConfiguration::default().substrate_port
                                            ),
                                            #[track = "model.network_configuration.substrate_port.is_unknown()"]
                                            set_value: *model.network_configuration.substrate_port as f64,
                                            set_width_chars: 5,
                                        },
                                    },

                                    gtk::Box {
                                        set_spacing: 10,

                                        gtk::Label {
                                            set_label: "Subspace (DSN) P2P port (TCP):"
                                        },
                                        gtk::SpinButton {
                                            connect_value_changed[sender] => move |entry| {
                                                sender.input(ConfigurationInput::SubspacePortChanged(
                                                    entry.value().round() as u16
                                                ));
                                            },
                                            set_adjustment: &gtk::Adjustment::new(
                                                0.0,
                                                0.0,
                                                u16::MAX as f64,
                                                1.0,
                                                0.0,
                                                0.0,
                                            ),
                                            set_tooltip: &format!(
                                                "Default port number is {}",
                                                NetworkConfiguration::default().subspace_port
                                            ),
                                            #[track = "model.network_configuration.subspace_port.is_unknown()"]
                                            set_value: *model.network_configuration.subspace_port as f64,
                                            set_width_chars: 5,
                                        },
                                    },

                                    gtk::Box {
                                        set_spacing: 10,

                                        gtk::Label {
                                            set_label: "Faster networking:"
                                        },
                                        gtk::Switch {
                                            connect_state_set[sender] => move |_switch, state| {
                                                sender.input(ConfigurationInput::FasterNetworkingChanged(
                                                    state
                                                ));

                                                gtk::glib::Propagation::Proceed
                                            },
                                            #[watch]
                                            set_active: model.network_configuration.faster_networking,
                                            set_tooltip:
                                                "By default networking is optimized for consumer routers, but if you have more powerful setup, faster networking may improve sync speed and other processes",
                                        },
                                    },
                                },
                            },
                        },
                    },

                    gtk::Box {
                        gtk::Box {
                            set_halign: gtk::Align::Start,
                            set_hexpand: true,
                            set_spacing: 10,

                            gtk::Button {
                                connect_clicked => ConfigurationInput::AddFarm,

                                gtk::Label {
                                    set_label: "Add farm",
                                    set_margin_all: 10,
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
                                    set_sensitive: model.reward_address.is_valid()
                                        && model.node_path.is_valid()
                                        && !model.farms.is_empty()
                                        && model.farms.iter().all(FarmWidget::valid),

                                    gtk::Label {
                                        set_label: "Save",
                                        set_margin_all: 10,
                                    },
                                },
                            }
                        } else {
                            gtk::Box {
                                set_halign: gtk::Align::End,
                                set_spacing: 10,

                                gtk::Button {
                                    connect_clicked => ConfigurationInput::Back,

                                    gtk::Label {
                                        set_label: "Back",
                                        set_margin_all: 10,
                                    },
                                },

                                gtk::Button {
                                    add_css_class: "suggested-action",
                                    connect_clicked => ConfigurationInput::Start,
                                    #[watch]
                                    set_sensitive:
                                        model.reward_address.is_valid()
                                            && model.node_path.is_valid()
                                            && !model.farms.is_empty()
                                            && model.farms.iter().all(FarmWidget::valid),

                                    gtk::Label {
                                        set_label: "Start",
                                        set_margin_all: 10,
                                    },
                                },
                            }
                        },
                    },
                },
            },
        }
    }

    fn init(
        parent_root: Self::Init,
        root: Self::Root,
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

        let mut farms = FactoryVecDeque::builder()
            .launch(gtk::ListBox::new())
            .forward(sender.input_sender(), |output| match output {
                FarmWidgetOutput::OpenDirectory(index) => {
                    ConfigurationInput::OpenDirectory(DirectoryKind::FarmPath(index))
                }
                FarmWidgetOutput::ValidityUpdate => ConfigurationInput::Ignore,
                FarmWidgetOutput::Delete(index) => ConfigurationInput::Delete(index),
            });

        farms.guard().push_back(FarmWidgetInit::default());

        let model = Self {
            reward_address: Default::default(),
            node_path: Default::default(),
            farms,
            network_configuration: Default::default(),
            pending_directory_selection: Default::default(),
            open_dialog,
            reconfiguration: false,
        };

        let configuration_list_box = model.farms.widget();
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
            ConfigurationInput::AddFarm => {
                self.farms.guard().push_back(FarmWidgetInit::default());
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
                    Some(DirectoryKind::FarmPath(index)) => {
                        self.farms.send(
                            index.current_index(),
                            FarmWidgetInput::DirectorySelected(path),
                        );
                    }
                    None => {
                        warn!(
                            directory = %path.display(),
                            "Directory selected, but no pending selection found",
                        );
                    }
                }
            }
            ConfigurationInput::SubstratePortChanged(port) => {
                self.network_configuration.substrate_port = MaybeValid::Valid(port);
            }
            ConfigurationInput::SubspacePortChanged(port) => {
                self.network_configuration.subspace_port = MaybeValid::Valid(port);
            }
            ConfigurationInput::FasterNetworkingChanged(faster_networking) => {
                self.network_configuration.faster_networking = faster_networking;
            }
            ConfigurationInput::Delete(index) => {
                let mut farms = self.farms.guard();
                farms.remove(index.current_index());
                // Force re-rendering of all farms
                farms.iter_mut().for_each(|_| {
                    // Nothing
                });
            }
            ConfigurationInput::RewardAddressChanged(new_reward_address) => {
                let new_reward_address = new_reward_address.trim();
                self.reward_address = if parse_ss58_reward_address(new_reward_address).is_ok() {
                    MaybeValid::Valid(new_reward_address.to_string())
                } else {
                    MaybeValid::Invalid(new_reward_address.to_string())
                };
            }
            ConfigurationInput::Reconfigure(raw_config) => {
                // `Unknown` is a hack to make it actually render the first time
                self.reward_address = MaybeValid::Unknown(raw_config.reward_address().to_string());
                self.node_path = MaybeValid::Valid(raw_config.node_path().clone());
                {
                    let mut farms = self.farms.guard();
                    farms.clear();
                    for farm in raw_config.farms() {
                        farms.push_back(FarmWidgetInit {
                            path: MaybeValid::Valid(farm.path.clone()),
                            // `Unknown` is a hack to make it actually render the first time
                            size: MaybeValid::Unknown(farm.size.clone()),
                        });
                    }
                }
                self.network_configuration =
                    NetworkConfigurationWrapper::from(raw_config.network());
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
            ConfigurationInput::Back => {
                if sender.output(ConfigurationOutput::Back).is_err() {
                    debug!("Failed to send ConfigurationOutput::Back");
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
            farms: self.farms.iter().map(FarmWidget::farm).collect(),
            network: NetworkConfiguration {
                substrate_port: *self.network_configuration.substrate_port,
                subspace_port: *self.network_configuration.subspace_port,
                faster_networking: self.network_configuration.faster_networking,
            },
        }
    }
}
