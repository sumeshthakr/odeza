//! # Odeza CLI
//!
//! Command-line interface for the Odeza game engine.
//!
//! ## Commands
//! - `build` - Build the project
//! - `run` - Run the project
//! - `cook` - Cook assets for target platform
//! - `package` - Package for distribution
//! - `deploy` - Deploy to device
//! - `profile` - Capture performance profile

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Odeza Game Engine CLI
#[derive(Parser)]
#[command(name = "odeza")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Project directory
    #[arg(short, long, default_value = ".")]
    pub project: PathBuf,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

/// CLI commands
#[derive(Subcommand)]
pub enum Commands {
    /// Build the project
    Build {
        /// Build configuration
        #[arg(short, long, default_value = "development")]
        config: String,

        /// Target platform
        #[arg(short, long, default_value = "native")]
        platform: String,

        /// Clean build
        #[arg(long)]
        clean: bool,
    },

    /// Run the project
    Run {
        /// Build configuration
        #[arg(short, long, default_value = "development")]
        config: String,

        /// Additional arguments
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Cook assets for target platform
    Cook {
        /// Target platform
        #[arg(short, long)]
        platform: String,

        /// Force recook all assets
        #[arg(long)]
        force: bool,

        /// Specific asset to cook
        #[arg(short, long)]
        asset: Option<String>,
    },

    /// Package for distribution
    Package {
        /// Target platform
        #[arg(short, long)]
        platform: String,

        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Build configuration
        #[arg(short, long, default_value = "shipping")]
        config: String,
    },

    /// Deploy to device
    Deploy {
        /// Target device
        #[arg(short, long)]
        device: Option<String>,

        /// List available devices
        #[arg(long)]
        list: bool,
    },

    /// Capture performance profile
    Profile {
        /// Duration in seconds
        #[arg(short, long, default_value = "10")]
        duration: u32,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Create a new project
    New {
        /// Project name
        name: String,

        /// Project directory
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Template to use
        #[arg(short, long, default_value = "empty")]
        template: String,
    },

    /// Initialize project in current directory
    Init {
        /// Project name
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Open the editor
    Editor,
}

/// Execute the CLI command
pub fn execute(cli: Cli) -> Result<()> {
    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    match cli.command {
        Commands::Build { config, platform, clean } => {
            log::info!("Building project...");
            log::info!("  Config: {}", config);
            log::info!("  Platform: {}", platform);
            if clean {
                log::info!("  Clean build requested");
            }
            // Build logic would go here
            log::info!("Build complete!");
        }

        Commands::Run { config, args } => {
            log::info!("Running project...");
            log::info!("  Config: {}", config);
            if !args.is_empty() {
                log::info!("  Args: {:?}", args);
            }
            // Run logic would go here
        }

        Commands::Cook { platform, force, asset } => {
            log::info!("Cooking assets for {}...", platform);
            if force {
                log::info!("  Force recook enabled");
            }
            if let Some(asset) = asset {
                log::info!("  Specific asset: {}", asset);
            }
            // Cook logic would go here
            log::info!("Cook complete!");
        }

        Commands::Package { platform, output, config } => {
            log::info!("Packaging for {}...", platform);
            log::info!("  Config: {}", config);
            if let Some(output) = output {
                log::info!("  Output: {}", output.display());
            }
            // Package logic would go here
            log::info!("Package complete!");
        }

        Commands::Deploy { device, list } => {
            if list {
                log::info!("Available devices:");
                log::info!("  (No devices found)");
            } else {
                let device = device.unwrap_or_else(|| "default".to_string());
                log::info!("Deploying to {}...", device);
                // Deploy logic would go here
            }
        }

        Commands::Profile { duration, output } => {
            log::info!("Capturing profile for {} seconds...", duration);
            if let Some(output) = output {
                log::info!("  Output: {}", output.display());
            }
            // Profile capture logic would go here
        }

        Commands::New { name, path, template } => {
            let project_path = path.unwrap_or_else(|| PathBuf::from(&name));
            log::info!("Creating new project '{}'...", name);
            log::info!("  Path: {}", project_path.display());
            log::info!("  Template: {}", template);
            // Project creation logic would go here
            log::info!("Project created!");
        }

        Commands::Init { name } => {
            let name = name.unwrap_or_else(|| {
                std::env::current_dir()
                    .ok()
                    .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                    .unwrap_or_else(|| "untitled".to_string())
            });
            log::info!("Initializing project '{}'...", name);
            // Init logic would go here
            log::info!("Project initialized!");
        }

        Commands::Editor => {
            log::info!("Opening editor...");
            // Editor launch logic would go here
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse() {
        let cli = Cli::parse_from(["odeza", "build"]);
        assert!(matches!(cli.command, Commands::Build { .. }));
    }

    #[test]
    fn test_build_command() {
        let cli = Cli::parse_from(["odeza", "build", "-c", "debug", "-p", "android", "--clean"]);
        if let Commands::Build { config, platform, clean } = cli.command {
            assert_eq!(config, "debug");
            assert_eq!(platform, "android");
            assert!(clean);
        } else {
            panic!("Expected Build command");
        }
    }
}
