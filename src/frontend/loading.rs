use crate::backend::LoadingStep;
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
                self.set_title(match &step {
                    LoadingStep::LoadingConfiguration
                    | LoadingStep::ReadingConfiguration
                    | LoadingStep::ConfigurationReadSuccessfully { .. }
                    | LoadingStep::CheckingConfiguration
                    | LoadingStep::ConfigurationIsValid
                    | LoadingStep::DecodingChainSpecification
                    | LoadingStep::DecodedChainSpecificationSuccessfully => {
                        "Loading configuration".to_string()
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
                        "Initializing networking stack".to_string()
                    }
                    LoadingStep::CreatingConsensusNode
                    | LoadingStep::ConsensusNodeCreatedSuccessfully => {
                        "Initializing consensus node".to_string()
                    }
                    LoadingStep::InitializingFarms { .. }
                    | LoadingStep::FarmInitialized { .. }
                    | LoadingStep::FarmerCreatedSuccessfully => "Instantiating farmer".to_string(),
                    LoadingStep::WipingFarm { .. } | LoadingStep::WipedFarmsSuccessfully => {
                        "Wiping farmer data".to_string()
                    }
                    LoadingStep::WipingNode { .. } | LoadingStep::WipedNodeSuccessfully => {
                        "Wiping node data".to_string()
                    }
                });
                self.set_progress(step.progress());
                self.set_message(match step {
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
                    LoadingStep::InitializingFarms { farms_total } => {
                        format!("Initializing farms 0/{farms_total}...")
                    }
                    LoadingStep::FarmInitialized {
                        farm_index,
                        farms_total,
                    } => format!(
                        "Initializing farms {}/{farms_total}...",
                        u16::from(farm_index) + 1
                    ),
                    LoadingStep::FarmerCreatedSuccessfully => {
                        "Farmer created successfully".to_string()
                    }
                    LoadingStep::WipingFarm {
                        farm_index,
                        farms_total,
                        path,
                    } => {
                        format!(
                            "Wiping farm {}/{} at {}...",
                            u16::from(farm_index) + 1,
                            farms_total,
                            path.display()
                        )
                    }
                    LoadingStep::WipedFarmsSuccessfully => {
                        "All farms wiped successfully".to_string()
                    }
                    LoadingStep::WipingNode { path } => {
                        format!("Wiping node at {}...", path.display())
                    }
                    LoadingStep::WipedNodeSuccessfully => {
                        "Node data wiped successfully".to_string()
                    }
                });
            }
        }
    }
}
