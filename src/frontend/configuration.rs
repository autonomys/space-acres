mod farm;
mod utils;

use crate::backend::config::{NetworkConfiguration, RawConfig};
use crate::frontend::configuration::farm::{
    FarmWidget, FarmWidgetInit, FarmWidgetInput, FarmWidgetOutput,
};
use crate::frontend::configuration::utils::is_directory_writable;
use crate::frontend::translations::{AsDefaultStr, T};
use gtk::prelude::*;
use relm4::factory::AsyncFactoryVecDeque;
use relm4::prelude::*;
use relm4_components::open_dialog::{
    OpenDialog, OpenDialogMsg, OpenDialogResponse, OpenDialogSettings,
};
use relm4_icons::icon_name;
use std::ops::Deref;
use std::path::PathBuf;
use subspace_farmer::utils::ss58::parse_ss58_reward_address;
use tracing::{debug, error, warn};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DirectoryKind {
    NodePath,
    FarmPath(DynamicIndex),
}

#[derive(Debug)]
pub enum ConfigurationInput {
    AddFarm,
    RewardAddressChanged(String),
    CreateWallet,
    OpenDirectory(DirectoryKind),
    DirectorySelected(PathBuf),
    SubstratePortChanged(u16),
    SubspacePortChanged(u16),
    FasterNetworkingChanged(bool),
    Delete(DynamicIndex),
    Reinitialize {
        raw_config: RawConfig,
        reconfiguration: bool,
    },
    Help,
    Start,
    Back,
    Cancel,
    Save,
    UpdateFarms,
    Ignore,
}

#[derive(Debug)]
pub enum ConfigurationOutput {
    StartWithNewConfig(RawConfig),
    ConfigUpdate(RawConfig),
    Back,
    Close,
}

#[tracker::track]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct MaybeValid<T>
where
    T: PartialEq,
{
    value: T,
    is_valid: bool,
}

impl<T> Deref for MaybeValid<T>
where
    T: PartialEq,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> MaybeValid<T>
where
    T: PartialEq,
{
    /// Initialize as yes with tracker set to changed
    fn yes(value: T) -> Self {
        Self {
            value,
            is_valid: true,
            tracker: u8::MAX,
        }
    }

    /// Initialize as no with tracker set to changed
    fn no(value: T) -> Self {
        Self {
            value,
            is_valid: false,
            tracker: u8::MAX,
        }
    }

    fn icon(&self) -> Option<&'static str> {
        if self.is_valid {
            Some(icon_name::CHECKMARK)
        } else {
            Some(icon_name::CROSS)
        }
    }
}

#[tracker::track]
#[derive(Debug)]
struct NetworkConfigurationWrapper {
    substrate_port: u16,
    subspace_port: u16,
    faster_networking: bool,
}

impl Default for NetworkConfigurationWrapper {
    fn default() -> Self {
        Self::from(NetworkConfiguration::default())
    }
}

impl From<NetworkConfiguration> for NetworkConfigurationWrapper {
    fn from(config: NetworkConfiguration) -> Self {
        Self {
            substrate_port: config.substrate_port,
            subspace_port: config.subspace_port,
            faster_networking: config.faster_networking,
            tracker: u8::MAX,
        }
    }
}

#[tracker::track]
#[derive(Debug)]
pub struct ConfigurationView {
    #[do_not_track]
    reward_address: MaybeValid<String>,
    #[do_not_track]
    node_path: MaybeValid<PathBuf>,
    #[no_eq]
    farms: AsyncFactoryVecDeque<FarmWidget>,
    #[do_not_track]
    network_configuration: NetworkConfigurationWrapper,
    #[do_not_track]
    pending_directory_selection: Option<DirectoryKind>,
    #[do_not_track]
    open_dialog: Controller<OpenDialog>,
    #[do_not_track]
    reconfiguration: bool,
}

#[relm4::component(pub async)]
impl AsyncComponent for ConfigurationView {
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
                                    set_label: &T.configuration_node_path(),
                                },

                                gtk::Box {
                                    add_css_class: "linked",

                                    gtk::Entry {
                                        set_can_focus: false,
                                        #[track = "model.node_path.changed_is_valid()"]
                                        set_css_classes: if model.node_path.is_valid {
                                            &["valid-input"]
                                        } else {
                                            &["invalid-input"]
                                        },
                                        set_editable: false,
                                        set_hexpand: true,
                                        set_placeholder_text: Some(
                                            T
                                                .configuration_node_path_placeholder(
                                                    if cfg!(windows) {
                                                        "D:\\subspace-node"
                                                    } else if cfg!(target_os = "macos") {
                                                        "/Volumes/Subspace/subspace-node"
                                                    } else {
                                                        "/media/subspace-node"
                                                    },
                                                )
                                                .as_str(),
                                        ),
                                        set_primary_icon_name: Some(icon_name::SSD),
                                        set_primary_icon_activatable: false,
                                        set_primary_icon_sensitive: false,
                                        #[track = "model.node_path.changed_is_valid()"]
                                        set_secondary_icon_name: model.node_path.icon(),
                                        set_secondary_icon_activatable: false,
                                        set_secondary_icon_sensitive: false,
                                        #[track = "model.node_path.changed_value()"]
                                        set_text: model.node_path.display().to_string().as_str(),
                                        set_tooltip_markup: Some(
                                            &T.configuration_node_path_tooltip()
                                        ),
                                    },

                                    gtk::Button {
                                        connect_clicked => ConfigurationInput::OpenDirectory(
                                            DirectoryKind::NodePath
                                        ),
                                        set_label: &T.configuration_node_path_button_select(),
                                    },
                                },

                                gtk::Label {
                                    add_css_class: "error-label",
                                    set_halign: gtk::Align::Start,
                                    set_label: &T.configuration_node_path_error_doesnt_exist_or_write_permissions(),
                                    #[track = "self.node_path.changed_is_valid()"]
                                    set_visible: !model.node_path.is_valid && model.node_path.value != PathBuf::new(),
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
                                    set_label: &T.configuration_reward_address(),
                                },

                                gtk::Box {
                                    add_css_class: "linked",

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
                                        #[track = "model.reward_address.changed_is_valid()"]
                                        set_css_classes: if model.reward_address.is_valid {
                                            &["valid-input"]
                                        } else {
                                            &["invalid-input"]
                                        },
                                        set_hexpand: true,
                                        set_placeholder_text: Some(
                                            T
                                                .configuration_reward_address_placeholder(
                                                    "stB4S14whneyomiEa22Fu2PzVoibMB7n5PvBFUwafbCbRkC1K"
                                                )
                                                .as_str(),
                                        ),
                                        set_primary_icon_name: Some(icon_name::WALLET2),
                                        set_primary_icon_activatable: false,
                                        set_primary_icon_sensitive: false,
                                        #[track = "model.reward_address.changed_is_valid()"]
                                        set_secondary_icon_name: model.reward_address.icon(),
                                        set_secondary_icon_activatable: false,
                                        set_secondary_icon_sensitive: false,
                                        #[track = "model.reward_address.changed_value()"]
                                        set_text: &model.reward_address,
                                        set_tooltip_markup: Some(
                                            &T.configuration_reward_address_tooltip()
                                        ),
                                    },

                                    gtk::Button {
                                        connect_clicked => ConfigurationInput::CreateWallet,
                                        set_label: &T.configuration_reward_address_button_create_wallet(),
                                    },
                                },

                                gtk::Label {
                                    add_css_class: "error-label",
                                    set_halign: gtk::Align::Start,
                                    set_label: &T.configuration_reward_address_error_evm_address(),
                                    #[track = "model.reward_address.changed_value() || self.reward_address.changed_is_valid()"]
                                    set_visible: !model.reward_address.is_valid && model.reward_address.value.starts_with("0x"),
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
                        set_label: Some(&T.configuration_advanced()),

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
                                    set_label: &T.configuration_advanced_network(),
                                },

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 10,

                                    gtk::Box {
                                        set_spacing: 10,

                                        gtk::Label {
                                            set_label: &T.configuration_advanced_network_substrate_port()
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
                                            set_tooltip: T
                                                .configuration_advanced_network_default_port_number_tooltip(
                                                    NetworkConfiguration::default().substrate_port
                                                )
                                                .as_str(),
                                            #[track = "model.network_configuration.changed_substrate_port()"]
                                            set_value: model.network_configuration.substrate_port as f64,
                                            set_width_chars: 5,
                                        },
                                    },

                                    gtk::Box {
                                        set_spacing: 10,

                                        gtk::Label {
                                            set_label: &T.configuration_advanced_network_subspace_port(),
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
                                            set_tooltip: T
                                                .configuration_advanced_network_default_port_number_tooltip(
                                                    NetworkConfiguration::default().subspace_port
                                                )
                                                .as_str(),
                                            #[track = "model.network_configuration.changed_subspace_port()"]
                                            set_value: model.network_configuration.subspace_port as f64,
                                            set_width_chars: 5,
                                        },
                                    },

                                    gtk::Box {
                                        set_spacing: 10,

                                        gtk::Label {
                                            set_label: &T.configuration_advanced_network_faster_networking(),
                                        },
                                        gtk::Switch {
                                            connect_state_set[sender] => move |_switch, state| {
                                                sender.input(ConfigurationInput::FasterNetworkingChanged(
                                                    state
                                                ));

                                                gtk::glib::Propagation::Proceed
                                            },
                                            #[track = "model.network_configuration.changed_faster_networking()"]
                                            set_active: model.network_configuration.faster_networking,
                                            set_tooltip: &T.configuration_advanced_network_faster_networking_tooltip(),
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
                                    set_label: &T.configuration_button_add_farm(),
                                    set_margin_all: 10,
                                },
                            },
                        },

                        if model.reconfiguration {
                            gtk::Box {
                                set_halign: gtk::Align::End,
                                set_spacing: 10,

                                gtk::Button {
                                    connect_clicked => ConfigurationInput::Help,

                                    gtk::Label {
                                        set_label: &T.configuration_button_help(),
                                        set_margin_all: 10,
                                    },
                                },

                                gtk::Button {
                                    connect_clicked => ConfigurationInput::Cancel,

                                    gtk::Label {
                                        set_label: &T.configuration_button_cancel(),
                                        set_margin_all: 10,
                                    },
                                },

                                gtk::Button {
                                    add_css_class: "suggested-action",
                                    connect_clicked => ConfigurationInput::Save,
                                    #[track = "model.reward_address.changed_is_valid() || model.node_path.changed_is_valid() || model.changed_farms()"]
                                    set_sensitive:
                                        model.reward_address.is_valid
                                            && model.node_path.is_valid
                                            && !model.farms.is_empty()
                                            && model.farms.iter().all(|maybe_farm| maybe_farm.map(FarmWidget::valid).unwrap_or_default()),

                                    gtk::Label {
                                        set_label: &T.configuration_button_save(),
                                        set_margin_all: 10,
                                    },
                                },
                            }
                        } else {
                            gtk::Box {
                                set_halign: gtk::Align::End,
                                set_spacing: 10,

                                gtk::Button {
                                    connect_clicked => ConfigurationInput::Help,

                                    gtk::Label {
                                        set_label: &T.configuration_button_help(),
                                        set_margin_all: 10,
                                    },
                                },

                                gtk::Button {
                                    connect_clicked => ConfigurationInput::Back,

                                    gtk::Label {
                                        set_label: &T.configuration_button_back(),
                                        set_margin_all: 10,
                                    },
                                },

                                gtk::Button {
                                    add_css_class: "suggested-action",
                                    connect_clicked => ConfigurationInput::Start,
                                    #[track = "model.reward_address.changed_is_valid() || model.node_path.changed_is_valid() || model.changed_farms()"]
                                    set_sensitive:
                                        model.reward_address.is_valid
                                            && model.node_path.is_valid
                                            && !model.farms.is_empty()
                                            && model.farms.iter().all(|maybe_farm| maybe_farm.map(FarmWidget::valid).unwrap_or_default()),

                                    gtk::Label {
                                        set_label: &T.configuration_button_start(),
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

    async fn init(
        parent_root: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let open_dialog = OpenDialog::builder()
            .transient_for_native(&parent_root)
            .launch(OpenDialogSettings {
                folder_mode: true,
                accept_label: T.configuration_dialog_button_select().to_string(),
                cancel_label: T.configuration_dialog_button_cancel().to_string(),
                ..OpenDialogSettings::default()
            })
            .forward(sender.input_sender(), |response| match response {
                OpenDialogResponse::Accept(path) => ConfigurationInput::DirectorySelected(path),
                OpenDialogResponse::Cancel => ConfigurationInput::Ignore,
            });

        let mut farms = AsyncFactoryVecDeque::builder()
            .launch(gtk::ListBox::new())
            .forward(sender.input_sender(), |output| match output {
                FarmWidgetOutput::OpenDirectory(index) => {
                    ConfigurationInput::OpenDirectory(DirectoryKind::FarmPath(index))
                }
                FarmWidgetOutput::ValidityUpdate => ConfigurationInput::UpdateFarms,
                FarmWidgetOutput::Delete(index) => ConfigurationInput::Delete(index),
            });

        farms.guard().push_back(FarmWidgetInit::default());

        let model = Self {
            reward_address: MaybeValid::no(String::new()),
            node_path: MaybeValid::no(PathBuf::new()),
            farms,
            network_configuration: Default::default(),
            pending_directory_selection: Default::default(),
            open_dialog,
            reconfiguration: false,
            tracker: u8::MAX,
        };

        let configuration_list_box = model.farms.widget();
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        input: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        // Reset changes
        self.reset();
        self.reward_address.reset();
        self.node_path.reset();
        self.network_configuration.reset();

        self.process_input(input, sender).await;
    }
}

impl ConfigurationView {
    async fn process_input(
        &mut self,
        input: ConfigurationInput,
        sender: AsyncComponentSender<Self>,
    ) {
        match input {
            ConfigurationInput::AddFarm => {
                self.get_mut_farms()
                    .guard()
                    .push_back(FarmWidgetInit::default());
            }
            ConfigurationInput::OpenDirectory(directory_kind) => {
                self.pending_directory_selection.replace(directory_kind);
                self.open_dialog.emit(OpenDialogMsg::Open);
            }
            ConfigurationInput::DirectorySelected(path) => {
                match self.pending_directory_selection.take() {
                    Some(DirectoryKind::NodePath) => {
                        self.node_path = if is_directory_writable(path.clone()).await {
                            MaybeValid::yes(path)
                        } else {
                            MaybeValid::no(path)
                        };
                    }
                    Some(DirectoryKind::FarmPath(index)) => {
                        self.get_mut_farms().send(
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
                self.network_configuration.substrate_port = port;
            }
            ConfigurationInput::SubspacePortChanged(port) => {
                self.network_configuration.subspace_port = port;
            }
            ConfigurationInput::FasterNetworkingChanged(faster_networking) => {
                self.network_configuration.faster_networking = faster_networking;
            }
            ConfigurationInput::Delete(index) => {
                let mut farms = self.get_mut_farms().guard();
                farms.remove(index.current_index());
                // Force re-rendering of all farms
                farms.iter_mut().for_each(|_| {
                    // Nothing
                });
            }
            ConfigurationInput::CreateWallet => {
                if let Err(error) =
                    open::that_detached("https://docs.subspace.network/docs/category/wallets")
                {
                    error!(%error, "Failed to open create wallet page in default browser");
                }
            }
            ConfigurationInput::RewardAddressChanged(new_reward_address) => {
                let new_reward_address = new_reward_address.trim();
                // Force change so it is possible to render updates as the input changes
                *self.reward_address.get_mut_is_valid() =
                    parse_ss58_reward_address(new_reward_address).is_ok();
                self.reward_address.value = new_reward_address.to_string();
            }
            ConfigurationInput::Reinitialize {
                raw_config,
                reconfiguration,
            } => {
                let new_reward_address = raw_config.reward_address().trim();
                self.reward_address
                    .set_is_valid(parse_ss58_reward_address(new_reward_address).is_ok());
                self.reward_address
                    .set_value(new_reward_address.to_string());

                self.node_path = if is_directory_writable(raw_config.node_path().clone()).await {
                    MaybeValid::yes(raw_config.node_path().clone())
                } else {
                    MaybeValid::no(raw_config.node_path().clone())
                };
                {
                    let mut farms = self.get_mut_farms().guard();
                    farms.clear();
                    for farm in raw_config.farms() {
                        farms.push_back(FarmWidgetInit {
                            path: farm.path.clone(),
                            size: farm.size.clone(),
                        });
                    }
                }
                self.network_configuration =
                    NetworkConfigurationWrapper::from(raw_config.network());
                self.reconfiguration = reconfiguration;
            }
            ConfigurationInput::Help => {
                if let Err(error) = open::that_detached(
                    "https://docs.subspace.network/docs/category/space-acres-recommended/",
                ) {
                    error!(%error, "Failed to open help page in default browser");
                }
            }
            ConfigurationInput::Start => {
                if let Some(raw_config) = self.create_raw_config()
                    && sender
                        .output(ConfigurationOutput::StartWithNewConfig(raw_config))
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
                if let Some(raw_config) = self.create_raw_config()
                    && sender
                        .output(ConfigurationOutput::ConfigUpdate(raw_config))
                        .is_err()
                {
                    debug!("Failed to send ConfigurationOutput::ConfigUpdate");
                }
            }
            ConfigurationInput::UpdateFarms => {
                // Mark as changed
                let _ = self.get_mut_farms();
            }
            ConfigurationInput::Ignore => {
                // Ignore
            }
        }
    }

    /// Create raw config from own state
    fn create_raw_config(&self) -> Option<RawConfig> {
        Some(RawConfig::V0 {
            reward_address: String::clone(&self.reward_address),
            node_path: PathBuf::clone(&self.node_path),
            farms: self
                .farms
                .iter()
                .map(|maybe_farm_widget| Some(maybe_farm_widget?.farm()))
                .collect::<Option<Vec<_>>>()?,
            network: NetworkConfiguration {
                substrate_port: self.network_configuration.substrate_port,
                subspace_port: self.network_configuration.subspace_port,
                faster_networking: self.network_configuration.faster_networking,
            },
        })
    }
}
