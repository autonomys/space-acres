use crate::backend::farmer::{CACHE_PERCENTAGE, DiskFarm};
use bytesize::ByteSize;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use subspace_core_primitives::PublicKey;
use subspace_farmer::single_disk_farm::SingleDiskFarm;
use subspace_farmer::utils::ss58::{Ss58ParsingError, parse_ss58_reward_address};
use tokio::io::AsyncWriteExt;
use tokio::task;
use tracing::warn;

const DEFAULT_SUBSTRATE_PORT: u16 = 30333;
const DEFAULT_SUBSPACE_PORT: u16 = 30433;
pub const MIN_FARM_SIZE: u64 = ByteSize::gb(2).as_u64();
/// Marginal difference in farm size that will not trigger resizing
const FARM_SIZE_DIFF_MARGIN: u64 = ByteSize::gib(5).as_u64();
/// Margin for farm size allocation relatively to available space
const FARM_SIZE_ALLOCATION_MARGIN: u64 = ByteSize::gib(2).as_u64();

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Farm {
    pub path: PathBuf,
    /// Could be absolute value or percentage of free disk space (when ends with `%`)
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct NetworkConfiguration {
    pub substrate_port: u16,
    pub subspace_port: u16,
    #[serde(default)]
    pub faster_networking: bool,
}

impl Default for NetworkConfiguration {
    fn default() -> Self {
        Self {
            substrate_port: DEFAULT_SUBSTRATE_PORT,
            subspace_port: DEFAULT_SUBSPACE_PORT,
            faster_networking: false,
        }
    }
}

// TODO: This config is not necessarily valid, probably combine with valid config
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum RawConfig {
    #[serde(rename = "0", rename_all = "camelCase")]
    V0 {
        reward_address: String,
        node_path: PathBuf,
        // TODO: Use disk farm once it supports serde
        farms: Vec<Farm>,
        #[serde(default)]
        reduce_plotting_cpu_load: bool,
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
            reduce_plotting_cpu_load: false,
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
        let config_file_path = match tokio::fs::create_dir(&app_config_dir).await {
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
        match tokio::fs::read_to_string(config_file_path).await {
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
        let mut options = tokio::fs::OpenOptions::new();
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

    pub fn reduce_plotting_cpu_load(&self) -> bool {
        let Self::V0 {
            reduce_plotting_cpu_load,
            ..
        } = self;
        *reduce_plotting_cpu_load
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
    pub reduce_plotting_cpu_load: bool,
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
        check_path(node_path.clone()).await?;

        let mut farms = Vec::with_capacity(raw_config.farms().len());

        for farm in raw_config.farms() {
            check_path(farm.path.clone()).await?;

            let farm_details_fut = task::spawn_blocking({
                let farm = farm.clone();

                move || {
                    let fs_stats = fs4::statvfs(&farm.path)?;
                    let effective_disk_usage =
                        SingleDiskFarm::effective_disk_usage(&farm.path, CACHE_PERCENTAGE.get())
                            .map_err(|error| {
                                io::Error::other(format!(
                                    "Failed to check effective disk usage: {error}"
                                ))
                            })?;

                    Ok((fs_stats, effective_disk_usage))
                }
            });
            let farm_details_result = farm_details_fut
                .await
                .map_err(|error| io::Error::other(format!("Failed to spawn tokio task: {error}")))
                .flatten();

            let (fs_stats, effective_disk_usage) = match farm_details_result {
                Ok(result) => result,
                Err(error) => {
                    return Err(ConfigError::PathError {
                        path: farm.path.display().to_string(),
                        error,
                    });
                }
            };
            // Includes "virtual" free space that corresponds to the space farm already occupies,
            // which simplifies logic below when checking amount of space farm is able to occupy
            let available_space = fs_stats.available_space() + effective_disk_usage;

            let target_size = if farm.size.ends_with("%") {
                let size_percentage =
                    f64::from_str(farm.size.trim_end_matches('%')).map_err(|error| {
                        ConfigError::InvalidSizeFormat {
                            size: farm.size.clone(),
                            error: error.to_string(),
                        }
                    })?;
                if size_percentage <= 0.0 || size_percentage > 100.0 {
                    return Err(ConfigError::InvalidSizeFormat {
                        size: farm.size.clone(),
                        error: "Size percentage should be above 0% and not exceed 100%".to_string(),
                    });
                }

                let target_size = (available_space - FARM_SIZE_ALLOCATION_MARGIN) as f64
                    * size_percentage
                    / 100.0;
                let target_size = MIN_FARM_SIZE.max(target_size.round() as u64);

                if target_size.abs_diff(effective_disk_usage) <= FARM_SIZE_DIFF_MARGIN {
                    effective_disk_usage
                } else {
                    target_size
                }
            } else {
                ByteSize::from_str(&farm.size)
                    .map_err(|error| ConfigError::InvalidSizeFormat {
                        size: farm.size.clone(),
                        error,
                    })?
                    .as_u64()
            };

            let size = if target_size > available_space {
                let new_size = available_space - FARM_SIZE_ALLOCATION_MARGIN;
                warn!(
                    target_size,
                    available_space,
                    new_size,
                    "Overriding farm size due to not enough available space"
                );

                new_size
            } else {
                target_size
            };

            farms.push(DiskFarm {
                directory: farm.path.clone(),
                allocated_space: size,
            });
        }

        Ok(Self {
            reward_address,
            node_path,
            farms,
            reduce_plotting_cpu_load: raw_config.reduce_plotting_cpu_load(),
            network: raw_config.network(),
        })
    }
}

async fn check_path(path: PathBuf) -> Result<(), ConfigError> {
    let path_string = path.display().to_string();
    task::spawn_blocking(move || {
        let exists = path.try_exists().map_err(|error| ConfigError::PathError {
            path: path.display().to_string(),
            error,
        })?;

        if exists {
            // Try to create a temporary file to check if path is writable
            tempfile::tempfile_in(&path).map_err(|error| ConfigError::PathError {
                path: path.display().to_string(),
                error: io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    format!("Path not writable: {error}"),
                ),
            })?;
        } else {
            let Some(parent) = path.parent() else {
                return Err(ConfigError::InvalidPath {
                    path: path.display().to_string(),
                });
            };

            let parent_exists = parent
                .try_exists()
                .map_err(|error| ConfigError::PathError {
                    path: path.display().to_string(),
                    error,
                })?;

            if !parent_exists {
                return Err(ConfigError::InvalidPath {
                    path: path.display().to_string(),
                });
            }

            // Try to create a temporary file in parent directory to check if path is writable, and
            // it would be possible to create a parent directory later
            tempfile::tempfile_in(parent).map_err(|error| ConfigError::PathError {
                path: path.display().to_string(),
                error: io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    format!("Path doesn't exist and can't be created: {error}"),
                ),
            })?;
        }

        Ok(())
    })
    .await
    .map_err(|error| ConfigError::PathError {
        path: path_string,
        error: io::Error::other(format!("Failed to spawn tokio task: {error}")),
    })?
}
