use crate::frontend::translations::{AsDefaultStr, T};
use gtk::prelude::*;
use relm4::prelude::*;
use relm4::{Sender, ShutdownReceiver};
use reqwest::Client;
use semver::Version;
use serde::Deserialize;
use std::time::Duration;
use tracing::{debug, error, warn};

/// Check new release every hour
const NEW_VERSION_CHECK_INTERVAL: Duration = Duration::from_secs(3600);
/// Retry failed check every 5 minutes
const NEW_VERSION_CHECK_RETRY_INTERVAL: Duration = Duration::from_secs(5 * 60);

#[derive(Debug, Deserialize)]
struct LatestRelease {
    tag_name: String,
}

#[derive(Debug)]
pub enum NewVersionCommandOutput {
    NewVersion(Version),
}

#[tracker::track]
#[derive(Debug)]
pub struct NewVersion {
    new_version: Option<Version>,
}

#[relm4::component(pub)]
impl Component for NewVersion {
    type Init = ();
    type Input = ();
    type Output = ();
    type CommandOutput = NewVersionCommandOutput;

    view! {
        #[root]
        // TODO: Use LinkButton once https://gitlab.gnome.org/GNOME/glib/-/issues/3403 is fixed
        //  for macOS
        gtk::Button {
            add_css_class: "suggested-action",
            // TODO: Use LinkButton once https://gitlab.gnome.org/GNOME/glib/-/issues/3403 is fixed
            //  for macOS
            connect_clicked => move |_| {
                let repository = env!("CARGO_PKG_REPOSITORY");

                let link = if repository.starts_with("https://github.com") {
                    // Turn:
                    // https://github.com/autonomys/space-acres
                    // Into:
                    // https://github.com/autonomys/space-acres/releases
                    format!("{}/releases", env!("CARGO_PKG_REPOSITORY"))
                } else {
                    repository.to_string()
                };

                if let Err(error) = open::that_detached(link) {
                    error!(%error, "Failed to open releases page in default browser");
                }
            },
            remove_css_class: "flat",
            remove_css_class: "link",
            remove_css_class: "text-button",
            #[track = "model.changed_new_version()"]
            set_label: T
                .new_version_available(
                    model.new_version.as_ref().map(Version::to_string).unwrap_or_default()
                )
                .as_str(),
            set_tooltip: &T.new_version_available_button_open(),
            // TODO: Use LinkButton once https://gitlab.gnome.org/GNOME/glib/-/issues/3403 is fixed
            //  for macOS
            // set_uri: &{
            //     let repository = env!("CARGO_PKG_REPOSITORY");
            //
            //     if repository.starts_with("https://github.com") {
            //         // Turn:
            //         // https://github.com/autonomys/space-acres
            //         // Into:
            //         // https://github.com/autonomys/space-acres/releases
            //         format!("{}/releases", env!("CARGO_PKG_REPOSITORY"))
            //     } else {
            //         repository.to_string()
            //     }
            // },
            set_use_underline: false,
            #[track = "model.changed_new_version()"]
            set_visible: model.new_version.is_some(),
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            new_version: None,
            tracker: u8::MAX,
        };

        let widgets = view_output!();

        sender.command(Self::check_new_version);

        ComponentParts { model, widgets }
    }

    fn update_cmd(
        &mut self,
        input: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        // Reset changes
        self.reset();

        self.process_command(input);
    }
}

impl NewVersion {
    async fn check_new_version(
        sender: Sender<NewVersionCommandOutput>,
        shutdown_receiver: ShutdownReceiver,
    ) {
        let url = env!("CARGO_PKG_REPOSITORY");

        if !url.starts_with("https://github.com") {
            warn!(%url, "Unexpected repository URL, not checking for new version");
            return;
        }
        // Turn:
        // https://github.com/autonomys/space-acres
        // Into:
        // https://api.github.com/repos/autonomys/space-acres/releases/latest
        let mut url = url.replace("https://github.com", "https://api.github.com/repos");
        url.push_str("/releases/latest");

        let current_version = env!("CARGO_PKG_VERSION");
        let current_version = match Version::parse(current_version) {
            Ok(current_version) => current_version,
            Err(error) => {
                warn!(%error, %current_version, "Invalid version in Cargo.toml");
                return;
            }
        };
        let user_agent = format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

        shutdown_receiver
            .register(async move {
                let client = Client::new();
                loop {
                    let response: reqwest::Result<LatestRelease> = try {
                        client
                            .get(&url)
                            .header("User-Agent", &user_agent)
                            .send()
                            .await?
                            .json()
                            .await?
                    };

                    let tag_name = match response {
                        Ok(latest_release) => latest_release.tag_name,
                        Err(error) => {
                            warn!(%error, %url, "Failed to check new release");
                            tokio::time::sleep(NEW_VERSION_CHECK_RETRY_INTERVAL).await;
                            continue;
                        }
                    };

                    let new_version = match Version::parse(&tag_name) {
                        Ok(new_version) => new_version,
                        Err(error) => {
                            debug!(%error, %tag_name, "Failed to parse new version");
                            tokio::time::sleep(NEW_VERSION_CHECK_RETRY_INTERVAL).await;
                            continue;
                        }
                    };

                    if new_version > current_version
                        && sender
                            .send(NewVersionCommandOutput::NewVersion(new_version))
                            .is_err()
                    {
                        break;
                    }

                    tokio::time::sleep(NEW_VERSION_CHECK_INTERVAL).await;
                }
            })
            .drop_on_shutdown()
            .await
    }

    fn process_command(&mut self, command_output: NewVersionCommandOutput) {
        match command_output {
            NewVersionCommandOutput::NewVersion(version) => {
                self.get_mut_new_version().replace(version);
            }
        }
    }
}
