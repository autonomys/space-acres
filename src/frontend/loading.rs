use crate::backend::LoadingStep;
use gtk::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub enum LoadingInput {
    BackendLoading(LoadingStep),
}

#[derive(Debug)]
pub struct LoadingView {
    message: String,
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

            gtk::Spinner {
                start: (),
                set_size_request: (50, 50),
            },

            gtk::Label {
                #[watch]
                set_label: &model.message,
            },
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            message: String::new(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        self.process_input(input);
    }
}

impl LoadingView {
    fn process_input(&mut self, input: LoadingInput) {
        match input {
            LoadingInput::BackendLoading(step) => {
                self.message = match step {
                    LoadingStep::LoadingConfiguration => "Loading configuration...".to_string(),
                    LoadingStep::ReadingConfiguration => "Reading configuration...".to_string(),
                    LoadingStep::ConfigurationReadSuccessfully {
                        configuration_exists,
                    } => {
                        format!(
                            "Configuration {}",
                            if configuration_exists {
                                "found"
                            } else {
                                "not found"
                            }
                        )
                    }
                    LoadingStep::CheckingConfiguration => "Checking configuration...".to_string(),
                    LoadingStep::ConfigurationIsValid => "Configuration is valid".to_string(),
                    LoadingStep::DecodingChainSpecification => {
                        "Decoding chain specification...".to_string()
                    }
                    LoadingStep::DecodedChainSpecificationSuccessfully => {
                        "Decoded chain specification successfully".to_string()
                    }
                    LoadingStep::CheckingNodePath => "Checking node path...".to_string(),
                    LoadingStep::CreatingNodePath => "Creating node path...".to_string(),
                    LoadingStep::NodePathReady => "Node path ready".to_string(),
                    LoadingStep::PreparingNetworkingStack => {
                        "Preparing networking stack...".to_string()
                    }
                    LoadingStep::ReadingNetworkKeypair => "Reading network keypair...".to_string(),
                    LoadingStep::GeneratingNetworkKeypair => {
                        "Generating network keypair...".to_string()
                    }
                    LoadingStep::WritingNetworkKeypair => {
                        "Writing network keypair to disk...".to_string()
                    }
                    LoadingStep::InstantiatingNetworkingStack => {
                        "Instantiating networking stack...".to_string()
                    }
                    LoadingStep::NetworkingStackCreatedSuccessfully => {
                        "Networking stack created successfully".to_string()
                    }
                    LoadingStep::CreatingConsensusNode => "Creating consensus node...".to_string(),
                    LoadingStep::ConsensusNodeCreatedSuccessfully => {
                        "Consensus node created successfully".to_string()
                    }
                    LoadingStep::CreatingFarmer => "Creating farmer...".to_string(),
                    LoadingStep::FarmerCreatedSuccessfully => {
                        "Farmer created successfully".to_string()
                    }
                    LoadingStep::WipingFarm { farm_index, path } => {
                        format!("Wiping farm {farm_index} at {}...", path.display())
                    }
                    LoadingStep::WipingNode { path } => {
                        format!("Wiping node at {}...", path.display())
                    }
                };
            }
        }
    }
}
