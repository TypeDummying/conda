
import os
import sys
import json
import logging
import configparser
from typing import Dict, List, Optional, Union
from pathlib import Path
from dataclasses import dataclass, field
from enum import Enum, auto

# Set up logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(name)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

class EnvironmentType(Enum):
    """Enum for different environment types"""
    DEVELOPMENT = auto()
    STAGING = auto()
    PRODUCTION = auto()

@dataclass
class RustPackageConfig:
    """Configuration for a Rust package"""
    name: str
    version: str
    authors: List[str] = field(default_factory=list)
    description: Optional[str] = None
    dependencies: Dict[str, str] = field(default_factory=dict)
    dev_dependencies: Dict[str, str] = field(default_factory=dict)

@dataclass
class CondaEnvironmentConfig:
    """Configuration for a Conda environment"""
    name: str
    channels: List[str] = field(default_factory=list)
    dependencies: List[str] = field(default_factory=list)

class MainConfig:
    """Main configuration class for the Rust library / package manager conda"""

    def __init__(self, config_path: Union[str, Path] = "config.ini"):
        self.config_path = Path(config_path)
        self.config = configparser.ConfigParser()
        self.environment: EnvironmentType = EnvironmentType.DEVELOPMENT
        self.rust_package: RustPackageConfig = RustPackageConfig(name="", version="")
        self.conda_env: CondaEnvironmentConfig = CondaEnvironmentConfig(name="")
        self.cache_dir: Path = Path.home() / ".rust_conda_cache"
        self.max_concurrent_downloads: int = 5
        self.timeout: int = 60
        self.load_config()

    def load_config(self):
        """Load configuration from the config file"""
        if not self.config_path.exists():
            logger.warning(f"Config file not found at {self.config_path}. Using default values.")
            return

        try:
            self.config.read(self.config_path)

            # Load general settings
            general = self.config['General']
            self.environment = EnvironmentType[general.get('Environment', 'DEVELOPMENT').upper()]
            self.cache_dir = Path(general.get('CacheDir', str(self.cache_dir)))
            self.max_concurrent_downloads = general.getint('MaxConcurrentDownloads', self.max_concurrent_downloads)
            self.timeout = general.getint('Timeout', self.timeout)

            # Load Rust package settings
            rust = self.config['RustPackage']
            self.rust_package = RustPackageConfig(
                name=rust['Name'],
                version=rust['Version'],
                authors=json.loads(rust.get('Authors', '[]')),
                description=rust.get('Description'),
                dependencies=json.loads(rust.get('Dependencies', '{}')),
                dev_dependencies=json.loads(rust.get('DevDependencies', '{}'))
            )

            # Load Conda environment settings
            conda = self.config['CondaEnvironment']
            self.conda_env = CondaEnvironmentConfig(
                name=conda['Name'],
                channels=json.loads(conda.get('Channels', '[]')),
                dependencies=json.loads(conda.get('Dependencies', '[]'))
            )

        except Exception as e:
            logger.error(f"Error loading config: {e}")
            raise

    def save_config(self):
        """Save the current configuration to the config file"""
        try:
            # Update config object with current values
            self.config['General'] = {
                'Environment': self.environment.name,
                'CacheDir': str(self.cache_dir),
                'MaxConcurrentDownloads': str(self.max_concurrent_downloads),
                'Timeout': str(self.timeout)
            }

            self.config['RustPackage'] = {
                'Name': self.rust_package.name,
                'Version': self.rust_package.version,
                'Authors': json.dumps(self.rust_package.authors),
                'Description': self.rust_package.description or '',
                'Dependencies': json.dumps(self.rust_package.dependencies),
                'DevDependencies': json.dumps(self.rust_package.dev_dependencies)
            }

            self.config['CondaEnvironment'] = {
                'Name': self.conda_env.name,
                'Channels': json.dumps(self.conda_env.channels),
                'Dependencies': json.dumps(self.conda_env.dependencies)
            }

            # Write to file
            with open(self.config_path, 'w') as configfile:
                self.config.write(configfile)

            logger.info(f"Configuration saved to {self.config_path}")
        except Exception as e:
            logger.error(f"Error saving config: {e}")
            raise

    def update_rust_package(self, **kwargs):
        """Update Rust package configuration"""
        for key, value in kwargs.items():
            if hasattr(self.rust_package, key):
                setattr(self.rust_package, key, value)
            else:
                logger.warning(f"Invalid Rust package attribute: {key}")

    def update_conda_env(self, **kwargs):
        """Update Conda environment configuration"""
        for key, value in kwargs.items():
            if hasattr(self.conda_env, key):
                setattr(self.conda_env, key, value)
            else:
                logger.warning(f"Invalid Conda environment attribute: {key}")

    def validate_config(self) -> bool:
        """Validate the current configuration"""
        try:
            assert self.rust_package.name, "Rust package name is required"
            assert self.rust_package.version, "Rust package version is required"
            assert self.conda_env.name, "Conda environment name is required"
            assert self.cache_dir.is_dir(), f"Cache directory does not exist: {self.cache_dir}"
            assert self.max_concurrent_downloads > 0, "Max concurrent downloads must be positive"
            assert self.timeout > 0, "Timeout must be positive"
            return True
        except AssertionError as e:
            logger.error(f"Configuration validation failed: {e}")
            return False

    def get_environment_variables(self) -> Dict[str, str]:
        """Get environment variables based on the current configuration"""
        return {
            "RUST_CONDA_ENV": self.environment.name,
            "RUST_PACKAGE_NAME": self.rust_package.name,
            "RUST_PACKAGE_VERSION": self.rust_package.version,
            "CONDA_ENV_NAME": self.conda_env.name,
            "RUST_CONDA_CACHE_DIR": str(self.cache_dir),
            "RUST_CONDA_MAX_CONCURRENT_DOWNLOADS": str(self.max_concurrent_downloads),
            "RUST_CONDA_TIMEOUT": str(self.timeout)
        }

    def apply_environment_variables(self):
        """Apply the configuration as environment variables"""
        for key, value in self.get_environment_variables().items():
            os.environ[key] = value
        logger.info("Environment variables applied")

    def generate_rust_toml(self) -> str:
        """Generate Cargo.toml content based on the Rust package configuration"""
        toml_content = f"""
[package]
name = "{self.rust_package.name}"
version = "{self.rust_package.version}"
authors = {json.dumps(self.rust_package.authors)}
description = "{self.rust_package.description or ''}"

[dependencies]
{self._format_dependencies(self.rust_package.dependencies)}

[dev-dependencies]
{self._format_dependencies(self.rust_package.dev_dependencies)}
"""
        return toml_content.strip()

    def generate_conda_yaml(self) -> str:
        """Generate conda environment.yml content based on the Conda environment configuration"""
        yaml_content = f"""
name: {self.conda_env.name}
channels:
{self._format_list(self.conda_env.channels)}
dependencies:
{self._format_list(self.conda_env.dependencies)}
"""
        return yaml_content.strip()

    @staticmethod
    def _format_dependencies(deps: Dict[str, str]) -> str:
        return "\n".join(f"{name} = \"{version}\"" for name, version in deps.items())

    @staticmethod
    def _format_list(items: List[str]) -> str:
        return "\n".join(f"  - {item}" for item in items)

    def __str__(self) -> str:
        """String representation of the configuration"""
        return f"""
MainConfig:
  Environment: {self.environment.name}
  Cache Directory: {self.cache_dir}
  Max Concurrent Downloads: {self.max_concurrent_downloads}
  Timeout: {self.timeout}

  Rust Package:
    Name: {self.rust_package.name}
    Version: {self.rust_package.version}
    Authors: {', '.join(self.rust_package.authors)}
    Description: {self.rust_package.description or 'N/A'}
    Dependencies: {len(self.rust_package.dependencies)}
    Dev Dependencies: {len(self.rust_package.dev_dependencies)}

  Conda Environment:
    Name: {self.conda_env.name}
    Channels: {', '.join(self.conda_env.channels)}
    Dependencies: {len(self.conda_env.dependencies)}
"""

if __name__ == "__main__":
    # Example usage
    config = MainConfig()
    config.update_rust_package(name="my_rust_lib", version="0.1.0", authors=["John Doe"])
    config.update_conda_env(name="rust_conda_env", channels=["conda-forge"])
    
    if config.validate_config():
        config.save_config()
        config.apply_environment_variables()
        print(config)
        print("\nCargo.toml content:")
        print(config.generate_rust_toml())
        print("\nconda environment.yml content:")
        print(config.generate_conda_yaml())
    else:
        print("Configuration is invalid. Please check the logs for details.")
