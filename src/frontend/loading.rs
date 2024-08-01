use crate::backend::LoadingStep;
use crate::frontend::translations::{AsDefaultStr, T};
use gtk::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub enum LoadingInput {
    BackendLoading(LoadingStep),
}

#[tracker::track]
#[derive(Debug)]
pub struct LoadingView {
    title: String,
    message: String,
    /// Progress in %: 0.0..=100.0
    progress: f32,
}

#[relm4::component(pub)]
impl Component for LoadingView {
    type Init = ();
    type Input = LoadingInput;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,
            set_vexpand: true,
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,

            gtk::Label {
               #[track = "model.changed_title()"]
               set_markup: &format!("<span size=\"medium\" weight=\"medium\">{}</span>", &model.title),
            },

            gtk::ProgressBar {
                #[track = "model.changed_progress()"]
                set_fraction: f64::from(model.progress / 100.0),
            },

            gtk::Label {
                #[track = "model.changed_message()"]
                set_markup: &format!("<span color=\"grey\">{}</span>", &model.message),
            },
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            title: String::new(),
            message: String::new(),
            progress: 0.0,
            tracker: u8::MAX,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        // Reset changes
        self.reset();

        self.process_input(input);
    }
}

impl LoadingView {
    fn process_input(&mut self, input: LoadingInput) {
        match input {
            LoadingInput::BackendLoading(step) => {
                let title = match &step {
                    LoadingStep::LoadingConfiguration
                    | LoadingStep::ReadingConfiguration
                    | LoadingStep::ConfigurationReadSuccessfully { .. }
                    | LoadingStep::CheckingConfiguration
                    | LoadingStep::ConfigurationIsValid
                    | LoadingStep::DecodingChainSpecification
                    | LoadingStep::DecodedChainSpecificationSuccessfully => {
                        T.loading_configuration_title()
                    }
                    LoadingStep::CheckingNodePath
                    | LoadingStep::CreatingNodePath
                    | LoadingStep::NodePathReady
                    | LoadingStep::PreparingNetworkingStack
                    | LoadingStep::ReadingNetworkKeypair
                    | LoadingStep::GeneratingNetworkKeypair
                    | LoadingStep::WritingNetworkKeypair
                    | LoadingStep::InstantiatingNetworkingStack
                    | LoadingStep::NetworkingStackCreatedSuccessfully => {
                        T.loading_networking_stack_title()
                    }
                    LoadingStep::CreatingConsensusNode
                    | LoadingStep::ConsensusNodeCreatedSuccessfully => {
                        T.loading_consensus_node_title()
                    }
                    LoadingStep::InitializingFarms { .. }
                    | LoadingStep::FarmInitialized { .. }
                    | LoadingStep::FarmerCreatedSuccessfully => T.loading_farmer_title(),
                    LoadingStep::WipingFarm { .. } | LoadingStep::WipedFarmsSuccessfully => {
                        T.loading_wiping_farmer_data_title()
                    }
                    LoadingStep::WipingNode { .. } | LoadingStep::WipedNodeSuccessfully => {
                        T.loading_wiping_node_data_title()
                    }
                };
                self.set_title(title.to_string());
                self.set_progress(step.progress());
                let message = match step {
                    LoadingStep::LoadingConfiguration => {
                        T.loading_configuration_step_loading().to_string()
                    }
                    LoadingStep::ReadingConfiguration => {
                        T.loading_configuration_step_reading().to_string()
                    }
                    LoadingStep::ConfigurationReadSuccessfully {
                        configuration_exists,
                    } => {
                        if configuration_exists {
                            T.loading_configuration_step_configuration_exists()
                                .to_string()
                        } else {
                            T.loading_configuration_step_configuration_not_found()
                                .to_string()
                        }
                    }
                    LoadingStep::CheckingConfiguration => T
                        .loading_configuration_step_configuration_checking()
                        .to_string(),
                    LoadingStep::ConfigurationIsValid => T
                        .loading_configuration_step_configuration_valid()
                        .to_string(),
                    LoadingStep::DecodingChainSpecification => T
                        .loading_configuration_step_decoding_chain_spec()
                        .to_string(),
                    LoadingStep::DecodedChainSpecificationSuccessfully => T
                        .loading_configuration_step_decoded_chain_spec()
                        .to_string(),
                    LoadingStep::CheckingNodePath => T
                        .loading_networking_stack_step_checking_node_path()
                        .to_string(),
                    LoadingStep::CreatingNodePath => T
                        .loading_networking_stack_step_creating_node_path()
                        .to_string(),
                    LoadingStep::NodePathReady => T
                        .loading_networking_stack_step_node_path_ready()
                        .to_string(),
                    LoadingStep::PreparingNetworkingStack => {
                        T.loading_networking_stack_step_preparing().to_string()
                    }
                    LoadingStep::ReadingNetworkKeypair => T
                        .loading_networking_stack_step_reading_keypair()
                        .to_string(),
                    LoadingStep::GeneratingNetworkKeypair => T
                        .loading_networking_stack_step_generating_keypair()
                        .to_string(),
                    LoadingStep::WritingNetworkKeypair => T
                        .loading_networking_stack_step_writing_keypair_to_disk()
                        .to_string(),
                    LoadingStep::InstantiatingNetworkingStack => {
                        T.loading_networking_stack_step_instantiating().to_string()
                    }
                    LoadingStep::NetworkingStackCreatedSuccessfully => T
                        .loading_networking_stack_step_created_successfully()
                        .to_string(),
                    LoadingStep::CreatingConsensusNode => {
                        T.loading_consensus_node_step_creating().to_string()
                    }
                    LoadingStep::ConsensusNodeCreatedSuccessfully => T
                        .loading_consensus_node_step_created_successfully()
                        .to_string(),
                    LoadingStep::InitializingFarms { farms_total } => T
                        .loading_farmer_step_initializing(0, farms_total)
                        .to_string(),
                    LoadingStep::FarmInitialized {
                        farm_index,
                        farms_total,
                    } => T
                        .loading_farmer_step_initializing(u16::from(farm_index) + 1, farms_total)
                        .to_string(),
                    LoadingStep::FarmerCreatedSuccessfully => {
                        T.loading_farmer_step_created_successfully().to_string()
                    }
                    LoadingStep::WipingFarm {
                        farm_index,
                        farms_total,
                        path,
                    } => T
                        .loading_wiping_farmer_data_step_wiping_farm(
                            farm_index,
                            farms_total,
                            path.display().to_string(),
                        )
                        .to_string(),
                    LoadingStep::WipedFarmsSuccessfully => {
                        T.loading_wiping_farmer_data_step_success().to_string()
                    }
                    LoadingStep::WipingNode { path } => T
                        .loading_wiping_node_data_step_wiping_node(path.display().to_string())
                        .to_string(),
                    LoadingStep::WipedNodeSuccessfully => {
                        T.loading_wiping_node_data_step_success().to_string()
                    }
                };
                self.set_message(message);
            }
        }
    }
}
