use crate::backend::farmer::DiskFarm;
use bytesize::ByteSize;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use subspace_core_primitives::PublicKey;
use subspace_farmer::utils::ss58::{parse_ss58_reward_address, Ss58ParsingError};
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

const DEFAULT_SUBSTRATE_PORT: u16 = 30333;
const DEFAULT_SUBSPACE_PORT: u16 = 30433;

// TODO: Replace with `DiskFarm`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Farm {
    pub path: PathBuf,
    pub size: String,
}

/// Configuration error
#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum RawConfigError {
    /// Failed to determine config directory
    #[error("Failed to determine config directory")]
    FailedToDetermineConfigDirectory,
    /// Failed to create config directory
    #[error("Failed to create config directory: {0}")]
    FailedToCreateConfigDirectory(io::Error),
    /// Failed to open configuration file
    #[error("Failed to open configuration file: {0}")]
    FailedToOpen(io::Error),
    /// Failed to deserialize configuration file
    #[error("Failed to deserialize configuration file: {0}")]
    FailedToDeserialize(serde_json::Error),
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct NetworkConfiguration {
    pub substrate_port: u16,
    pub subspace_port: u16,
}

impl Default for NetworkConfiguration {
    fn default() -> Self {
        Self {
            substrate_port: DEFAULT_SUBSTRATE_PORT,
            subspace_port: DEFAULT_SUBSPACE_PORT,
        }
    }
}

// TODO: This config is not necessarily valid, probably combine with valid config
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum RawConfig {
    #[serde(rename = "0", rename_all = "camelCase")]
    V0 {
        reward_address: String,
        node_path: PathBuf,
        // TODO: Use disk farm once it supports serde
        farms: Vec<Farm>,
        #[serde(default)]
        network: NetworkConfiguration,
    },
}

impl Default for RawConfig {
    fn default() -> Self {
        Self::V0 {
            reward_address: String::new(),
            node_path: PathBuf::new(),
            farms: Vec::new(),
            network: NetworkConfiguration::default(),
        }
    }
}

impl RawConfig {
    pub async fn default_path() -> Result<PathBuf, RawConfigError> {
        let Some(config_local_dir) = dirs::config_local_dir() else {
            return Err(RawConfigError::FailedToDetermineConfigDirectory);
        };

        let app_config_dir = config_local_dir.join(env!("CARGO_PKG_NAME"));
        let config_file_path = match fs::create_dir(&app_config_dir).await {
            Ok(()) => app_config_dir.join("config.json"),
            Err(error) => {
                if error.kind() == io::ErrorKind::AlreadyExists {
                    app_config_dir.join("config.json")
                } else {
                    return Err(RawConfigError::FailedToCreateConfigDirectory(error));
                }
            }
        };

        Ok(config_file_path)
    }

    pub async fn read_from_path(config_file_path: &Path) -> Result<Option<Self>, RawConfigError> {
        match fs::read_to_string(config_file_path).await {
            Ok(config) => serde_json::from_str::<Self>(&config)
                .map(Some)
                .map_err(RawConfigError::FailedToDeserialize),
            Err(error) => {
                if error.kind() == io::ErrorKind::NotFound {
                    Ok(None)
                } else {
                    Err(RawConfigError::FailedToOpen(error))
                }
            }
        }
    }

    pub async fn write_to_path(&self, config_file_path: &Path) -> io::Result<()> {
        let mut options = OpenOptions::new();
        options.write(true).truncate(true).create(true);
        #[cfg(unix)]
        options.mode(0o600);
        options
            .open(config_file_path)
            .await?
            .write_all(
                serde_json::to_string_pretty(self)
                    .expect("Config serialization is infallible; qed")
                    .as_bytes(),
            )
            .await
    }

    pub fn reward_address(&self) -> &str {
        let Self::V0 { reward_address, .. } = self;
        reward_address
    }

    pub fn node_path(&self) -> &PathBuf {
        let Self::V0 { node_path, .. } = self;
        node_path
    }

    pub fn farms(&self) -> &[Farm] {
        let Self::V0 { farms, .. } = self;
        farms
    }

    pub fn network(&self) -> NetworkConfiguration {
        let Self::V0 { network, .. } = self;
        *network
    }
}

/// Valid configuration error
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Invalid SS58 reward address
    #[error("Invalid SS58 reward address \"{reward_address}\": {error}")]
    InvalidSs58RewardAddress {
        reward_address: String,
        error: Ss58ParsingError,
    },
    /// Invalid path
    #[error("Path \"{path}\" is invalid")]
    InvalidPath { path: String },
    /// Path error
    #[error("Path \"{path}\" error: {error}")]
    PathError { path: String, error: io::Error },
    /// Invalid size format
    #[error("Invalid size format \"{size}\": {error}")]
    InvalidSizeFormat { size: String, error: String },
}

#[derive(Debug, Clone)]
pub struct Config {
    pub reward_address: PublicKey,
    pub node_path: PathBuf,
    pub farms: Vec<DiskFarm>,
    pub network: NetworkConfiguration,
}

impl Config {
    /// Tries to construct config from given raw config.
    ///
    /// It will check that path exists or parent directory can be accesses.
    pub async fn try_from_raw_config(raw_config: &RawConfig) -> Result<Self, ConfigError> {
        let reward_address = raw_config.reward_address();
        let reward_address = parse_ss58_reward_address(reward_address).map_err(|error| {
            ConfigError::InvalidSs58RewardAddress {
                reward_address: reward_address.to_string(),
                error,
            }
        })?;

        let node_path = raw_config.node_path().clone();
        check_path(&node_path).await?;

        let mut farms = Vec::with_capacity(raw_config.farms().len());

        for farm in raw_config.farms() {
            let path = PathBuf::from(&farm.path);

            check_path(&path).await?;

            let size = ByteSize::from_str(&farm.size)
                .map_err(|error| ConfigError::InvalidSizeFormat {
                    size: farm.size.clone(),
                    error,
                })?
                .as_u64();

            farms.push(DiskFarm {
                directory: path,
                allocated_plotting_space: size,
            });
        }

        Ok(Self {
            reward_address,
            node_path,
            farms,
            network: raw_config.network(),
        })
    }
}

async fn check_path(path: &Path) -> Result<(), ConfigError> {
    let exists = fs::try_exists(&path)
        .await
        .map_err(|error| ConfigError::PathError {
            path: path.display().to_string(),
            error,
        })?;

    if !exists {
        let Some(parent) = path.parent() else {
            return Err(ConfigError::InvalidPath {
                path: path.display().to_string(),
            });
        };

        let parent_exists =
            fs::try_exists(parent)
                .await
                .map_err(|error| ConfigError::PathError {
                    path: path.display().to_string(),
                    error,
                })?;

        if !parent_exists {
            return Err(ConfigError::InvalidPath {
                path: path.display().to_string(),
            });
        }
    }

    Ok(())
}
