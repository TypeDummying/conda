
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;
use toml;

/// Represents the configuration settings for the Conda package manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CondaSettings {
    /// The base directory for Conda environments.
    pub environments_dir: PathBuf,
    
    /// The default Python version to use when creating new environments.
    pub default_python_version: String,
    
    /// Whether to use strict channel priority when resolving dependencies.
    pub channel_priority_strict: bool,
    
    /// A list of default channels to use for package installation.
    pub default_channels: Vec<String>,
    
    /// Custom environment variables to set for all Conda operations.
    pub env_vars: HashMap<String, String>,
    
    /// The maximum number of retries for network operations.
    pub max_retries: u32,
    
    /// The timeout in seconds for network operations.
    pub network_timeout: u64,
    
    /// Whether to automatically update Conda on startup.
    pub auto_update: bool,
    
    /// The default shell to use for Conda operations.
    pub default_shell: String,
    
    /// Whether to show channel URLs when displaying package information.
    pub show_channel_urls: bool,
    
    /// The default compression type for creating packages.
    pub compression_type: CompressionType,
    
    /// Whether to use the 'conda-forge' channel by default.
    pub use_conda_forge: bool,
    
    /// The maximum number of environments to keep in the environments list.
    pub max_environments: usize,
    
    /// Whether to create .condarc file in the user's home directory.
    pub create_condarc: bool,
    
    /// The default architecture to use when installing packages.
    pub default_architecture: Architecture,
    
    /// Whether to verify SSL certificates for HTTPS connections.
    pub ssl_verify: bool,
    
    /// The proxy settings for network connections.
    pub proxy_settings: Option<ProxySettings>,
    
    /// Whether to use the local package cache.
    pub offline_mode: bool,
    
    /// The maximum size of the package cache in bytes.
    pub package_cache_size_limit: u64,
    
    /// Whether to add pip as a dependency to new environments by default.
    pub add_pip_as_python_dependency: bool,
}

/// Represents the compression type for creating packages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionType {
    Bzip2,
    Gzip,
    Lzma,
    Zstd,
}

/// Represents the default architecture for package installation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Architecture {
    X86_64,
    Arm64,
    Ppc64le,
    S390x,
}

/// Represents the proxy settings for network connections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySettings {
    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub no_proxy: Option<Vec<String>>,
}

/// Represents possible errors that can occur when working with Conda settings.
#[derive(Error, Debug)]
pub enum CondaSettingsError {
    #[error("Failed to read settings file: {0}")]
    FileReadError(#[from] std::io::Error),
    
    #[error("Failed to parse settings: {0}")]
    ParseError(#[from] toml::de::Error),
    
    #[error("Failed to serialize settings: {0}")]
    SerializeError(#[from] toml::ser::Error),
}

impl CondaSettings {
    /// Creates a new `CondaSettings` instance with default values.
    pub fn new() -> Self {
        CondaSettings {
            environments_dir: PathBuf::from("~/.conda/envs"),
            default_python_version: String::from("3.9"),
            channel_priority_strict: true,
            default_channels: vec![
                String::from("https://repo.anaconda.com/pkgs/main"),
                String::from("https://repo.anaconda.com/pkgs/r"),
            ],
            env_vars: HashMap::new(),
            max_retries: 3,
            network_timeout: 60,
            auto_update: false,
            default_shell: String::from("bash"),
            show_channel_urls: false,
            compression_type: CompressionType::Zstd,
            use_conda_forge: false,
            max_environments: 20,
            create_condarc: true,
            default_architecture: Architecture::X86_64,
            ssl_verify: true,
            proxy_settings: None,
            offline_mode: false,
            package_cache_size_limit: 10 * 1024 * 1024 * 1024, // 10 GB
            add_pip_as_python_dependency: true,
        }
    }

    /// Loads the Conda settings from a TOML file.
    pub fn load(path: &PathBuf) -> Result<Self, CondaSettingsError> {
        let contents = fs::read_to_string(path)?;
        let settings: CondaSettings = toml::from_str(&contents)?;
        Ok(settings)
    }

    /// Saves the Conda settings to a TOML file.
    pub fn save(&self, path: &PathBuf) -> Result<(), CondaSettingsError> {
        let toml_string = toml::to_string_pretty(self)?;
        fs::write(path, toml_string)?;
        Ok(())
    }

    /// Updates the settings with values from another `CondaSettings` instance.
    pub fn update(&mut self, other: &CondaSettings) {
        self.environments_dir = other.environments_dir.clone();
        self.default_python_version = other.default_python_version.clone();
        self.channel_priority_strict = other.channel_priority_strict;
        self.default_channels = other.default_channels.clone();
        self.env_vars = other.env_vars.clone();
        self.max_retries = other.max_retries;
        self.network_timeout = other.network_timeout;
        self.auto_update = other.auto_update;
        self.default_shell = other.default_shell.clone();
        self.show_channel_urls = other.show_channel_urls;
        self.compression_type = other.compression_type.clone();
        self.use_conda_forge = other.use_conda_forge;
        self.max_environments = other.max_environments;
        self.create_condarc = other.create_condarc;
        self.default_architecture = other.default_architecture.clone();
        self.ssl_verify = other.ssl_verify;
        self.proxy_settings = other.proxy_settings.clone();
        self.offline_mode = other.offline_mode;
        self.package_cache_size_limit = other.package_cache_size_limit;
        self.add_pip_as_python_dependency = other.add_pip_as_python_dependency;
    }

    /// Adds a custom environment variable to the settings.
    pub fn add_env_var(&mut self, key: String, value: String) {
        self.env_vars.insert(key, value);
    }

    /// Removes a custom environment variable from the settings.
    pub fn remove_env_var(&mut self, key: &str) -> Option<String> {
        self.env_vars.remove(key)
    }

    /// Adds a default channel to the settings.
    pub fn add_default_channel(&mut self, channel: String) {
        if !self.default_channels.contains(&channel) {
            self.default_channels.push(channel);
        }
    }

    /// Removes a default channel from the settings.
    pub fn remove_default_channel(&mut self, channel: &str) {
        self.default_channels.retain(|c| c != channel);
    }

    /// Sets the proxy settings for network connections.
    pub fn set_proxy_settings(&mut self, settings: ProxySettings) {
        self.proxy_settings = Some(settings);
    }

    /// Clears the proxy settings.
    pub fn clear_proxy_settings(&mut self) {
        self.proxy_settings = None;
    }

    /// Validates the current settings and returns a Result indicating whether they are valid.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.max_retries == 0 {
            errors.push("max_retries must be greater than 0".to_string());
        }

        if self.network_timeout == 0 {
            errors.push("network_timeout must be greater than 0".to_string());
        }

        if self.max_environments == 0 {
            errors.push("max_environments must be greater than 0".to_string());
        }

        if self.package_cache_size_limit == 0 {
            errors.push("package_cache_size_limit must be greater than 0".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Returns a formatted string representation of the settings for display purposes.
    pub fn to_display_string(&self) -> String {
        format!(
            "Conda Settings:
            - Environments Directory: {}
            - Default Python Version: {}
            - Channel Priority Strict: {}
            - Default Channels: {}
            - Max Retries: {}
            - Network Timeout: {} seconds
            - Auto Update: {}
            - Default Shell: {}
            - Show Channel URLs: {}
            - Compression Type: {:?}
            - Use conda-forge: {}
            - Max Environments: {}
            - Create .condarc: {}
            - Default Architecture: {:?}
            - SSL Verify: {}
            - Offline Mode: {}
            - Package Cache Size Limit: {} bytes
            - Add pip as Python Dependency: {}",
            self.environments_dir.display(),
            self.default_python_version,
            self.channel_priority_strict,
            self.default_channels.join(", "),
            self.max_retries,
            self.network_timeout,
            self.auto_update,
            self.default_shell,
            self.show_channel_urls,
            self.compression_type,
            self.use_conda_forge,
            self.max_environments,
            self.create_condarc,
            self.default_architecture,
            self.ssl_verify,
            self.offline_mode,
            self.package_cache_size_limit,
            self.add_pip_as_python_dependency
        )
    }

    /// Resets all settings to their default values.
    pub fn reset_to_defaults(&mut self) {
        *self = CondaSettings::new();
    }

    /// Merges the current settings with another `CondaSettings` instance, preferring non-default values.
    pub fn merge(&mut self, other: &CondaSettings) {
        let default = CondaSettings::new();

        if other.environments_dir != default.environments_dir {
            self.environments_dir = other.environments_dir.clone();
        }
        if other.default_python_version != default.default_python_version {
            self.default_python_version = other.default_python_version.clone();
        }
        if other.channel_priority_strict != default.channel_priority_strict {
            self.channel_priority_strict = other.channel_priority_strict;
        }
        if other.default_channels != default.default_channels {
            self.default_channels = other.default_channels.clone();
        }
        if !other.env_vars.is_empty() {
            self.env_vars.extend(other.env_vars.clone());
        }
        if other.max_retries != default.max_retries {
            self.max_retries = other.max_retries;
        }
        if other.network_timeout != default.network_timeout {
            self.network_timeout = other.network_timeout;
        }
        if other.auto_update != default.auto_update {
            self.auto_update = other.auto_update;
        }
        if other.default_shell != default.default_shell {
            self.default_shell = other.default_shell.clone();
        }
        if other.show_channel_urls != default.show_channel_urls {
            self.show_channel_urls = other.show_channel_urls;
        }
        if other.compression_type != default.compression_type {
            self.compression_type = other.compression_type.clone();
        }
        if other.use_conda_forge != default.use_conda_forge {
            self.use_conda_forge = other.use_conda_forge;
        }
        if other.max_environments != default.max_environments {
            self.max_environments = other.max_environments;
        }
        if other.create_condarc != default.create_condarc {
            self.create_condarc = other.create_condarc;
        }
        if other.default_architecture != default.default_architecture {
            self.default_architecture = other.default_architecture.clone();
        }
        if other.ssl_verify != default.ssl_verify {
            self.ssl_verify = other.ssl_verify;
        }
        if other.proxy_settings.is_some() {
            self.proxy_settings = other.proxy_settings.clone();
        }
        if other.offline_mode != default.offline_mode {
            self.offline_mode = other.offline_mode;
        }
        if other.package_cache_size_limit != default.package_cache_size_limit {
            self.package_cache_size_limit = other.package_cache_size_limit;
        }
        if other.add_pip_as_python_dependency != default.add_pip_as_python_dependency {
            self.add_pip_as_python_dependency = other.add_pip_as_python_dependency;
        }
    }

    /// Exports the current settings to a JSON string.
    pub fn export_to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Imports settings from a JSON string.
    pub fn import_from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Returns the total number of configured settings.
    pub fn count_settings(&self) -> usize {
        // Count the number of fields in the struct
        std::mem::size_of::<CondaSettings>() / std::mem::size_of::<usize>()
    }

    /// Checks if the current settings are using all default values.
    pub fn is_default(&self) -> bool {
        let default = CondaSettings::new();
        self == &default
    }

    /// Applies a custom function to modify a specific setting.
    pub fn modify_setting<F, T>(&mut self, field: &str, modifier: F) -> Result<(), String>
    where
        F: FnOnce(&mut T),
        T: 'static,
    {
        match field {
            "environments_dir" => modifier(unsafe { &mut *(&mut self.environments_dir as *mut _ as *mut T) }),
            "default_python_version" => modifier(unsafe { &mut *(&mut self.default_python_version as *mut _ as *mut T) }),
            "channel_priority_strict" => modifier(unsafe { &mut *(&mut self.channel_priority_strict as *mut _ as *mut T) }),
            "default_channels" => modifier(unsafe { &mut
            })}}}