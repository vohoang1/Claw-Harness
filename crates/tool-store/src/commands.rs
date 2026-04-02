//! Tool Store CLI Commands

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::ToolRegistry;

/// Claw-Harness Tool Store CLI
#[derive(Parser)]
#[command(name = "claw-tool")]
#[command(about = "Tool marketplace for Claw-Harness", long_about = None)]
pub struct Cli {
    /// Registry URL
    #[arg(long, env = "CLAW_REGISTRY_URL", default_value = "https://registry.claw-harness.io")]
    pub registry: String,

    /// Cache directory
    #[arg(long, env = "CLAW_TOOL_CACHE", default_value = "~/.claw/tools")]
    pub cache: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Search for tools in the registry
    Search {
        /// Search query
        query: String,
    },

    /// Install a tool
    Install {
        /// Tool name
        name: String,
        /// Tool version (latest if not specified)
        #[arg(short, long)]
        version: Option<String>,
    },

    /// Uninstall a tool
    Uninstall {
        /// Tool name
        name: String,
    },

    /// List installed tools
    List,

    /// Show tool information
    Info {
        /// Tool name
        name: String,
    },

    /// Publish a tool to the registry
    Publish {
        /// Path to .clawpkg file
        package: PathBuf,
    },

    /// Update registry index
    Update,
}

/// Run the CLI
pub async fn run() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();
    let registry = ToolRegistry::new(&cli.registry, cli.cache.clone());

    match cli.command {
        Commands::Search { query } => {
            let results = registry.search(&query).await?;
            
            if results.is_empty() {
                println!("No tools found matching '{}'", query);
            } else {
                println!("Found {} tool(s):\n", results.len());
                for tool in results {
                    println!("  {} v{}", tool.name, tool.version);
                    println!("    {}", tool.description);
                    if let Some(author) = &tool.author {
                        println!("    by @{}", author);
                    }
                    println!();
                }
            }
        }

        Commands::Install { name, version } => {
            registry.install(&name, version.as_deref()).await?;
            println!("✓ Installed {} {}", name, version.unwrap_or_else(|| "latest".to_string()));
        }

        Commands::Uninstall { name } => {
            registry.uninstall(&name)?;
            println!("✓ Uninstalled {}", name);
        }

        Commands::List => {
            let installed = registry.list_installed()?;
            
            if installed.is_empty() {
                println!("No tools installed.");
            } else {
                println!("Installed tools ({}):\n", installed.len());
                for tool in installed {
                    println!("  {} v{}", tool.name, tool.version);
                    println!("    {}", tool.description);
                    println!();
                }
            }
        }

        Commands::Info { name } => {
            let installed = registry.list_installed()?;
            
            if let Some(tool) = installed.iter().find(|t| t.name == name) {
                println!("{} v{}", tool.name, tool.version);
                println!();
                println!("Description: {}", tool.description);
                if let Some(author) = &tool.author {
                    println!("Author: {}", author);
                }
                if let Some(license) = &tool.license {
                    println!("License: {}", license);
                }
                println!();
                println!("Inputs:");
                for input in &tool.inputs {
                    println!("  - {} ({}) {:?}", input.name, input.input_type, input.required);
                }
                println!();
                println!("Outputs:");
                for output in &tool.outputs {
                    println!("  - {} ({})", output.name, output.output_type);
                }
            } else {
                println!("Tool '{}' not found. Run 'claw-tool list' to see installed tools.", name);
            }
        }

        Commands::Publish { package } => {
            // TODO: Implement publish to S3/MinIO
            println!("Publishing {}...", package.display());
            println!("Note: Publishing requires AWS credentials configured.");
            println!("This feature will be implemented in the next iteration.");
        }

        Commands::Update => {
            let index = registry.fetch_index().await?;
            println!("Registry updated! {} tools available.", index.tools.len());
        }
    }

    Ok(())
}
