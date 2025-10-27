// Allow clippy lints for the CLI crate
#![allow(clippy::too_many_arguments)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::unnecessary_map_or)]
#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::single_match)]
#![allow(clippy::items_after_test_module)]
#![allow(clippy::len_zero)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_comparisons)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

mod commands;
mod output;

use commands::{
    providers::{
        handle_add_instance, handle_get_instance, handle_list_instances, handle_providers,
        handle_remove_instance, handle_update_instance, handle_validate_instances,
    },
    scan::handle_scan,
};

/// GenAI Key Finder - Discover GenAI API keys and configurations
#[derive(Parser)]
#[command(name = "keyfinder")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Home directory to use (defaults to current user's home)
    #[arg(long, global = true)]
    home: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan for GenAI credentials and configurations
    Scan {
        /// Home directory to scan (defaults to current user's home)
        #[arg(long)]
        home: Option<String>,

        /// Output format (json, ndjson, table, summary)
        #[arg(long, short = 'f', default_value = "table")]
        format: String,

        /// Include full secret values (DANGEROUS - use with caution)
        #[arg(long)]
        include_values: bool,

        /// Only scan specific providers (comma-separated)
        #[arg(long)]
        only: Option<String>,

        /// Exclude specific providers (comma-separated)
        #[arg(long)]
        exclude: Option<String>,

        /// Maximum file size to read (in bytes)
        #[arg(long, default_value = "1048576")]
        max_bytes_per_file: usize,

        /// Dry run - show files that would be scanned without reading them
        #[arg(long)]
        dry_run: bool,

        /// Write audit log to file
        #[arg(long)]
        audit_log: Option<String>,

        /// Verbose output - show actual discovered keys
        #[arg(long, short = 'v')]
        verbose: bool,

        /// Update/create YAML configuration file with discovered providers and keys
        #[arg(long)]
        update: bool,
    },

    /// Show available providers and scanners
    Providers {
        /// Show detailed information
        #[arg(long, short = 'v')]
        verbose: bool,
    },

    /// Provider instance management commands
    Instances {
        /// Instance ID to get (positional argument - shorthand for 'instances get <id>')
        #[arg(value_name = "ID")]
        id: Option<String>,

        /// Include full secret values when using direct ID lookup (DANGEROUS - use with caution)
        #[arg(long)]
        include_values: bool,

        #[command(subcommand)]
        command: Option<InstanceCommands>,
    },

    /// Show version information
    Version,
}

#[derive(Subcommand)]
enum InstanceCommands {
    /// List all provider instances with their configurations
    List {
        /// Show detailed information including keys and models
        #[arg(long, short = 'v')]
        verbose: bool,

        /// Filter by provider type (e.g., openai, anthropic)
        #[arg(long)]
        provider_type: Option<String>,

        /// Show only active instances
        #[arg(long)]
        active_only: bool,
    },

    /// Add a new provider instance
    Add {
        /// Unique identifier for the instance
        #[arg(short = 'i', long)]
        id: String,

        /// Human-readable display name
        #[arg(short = 'n', long)]
        name: String,

        /// Provider type (e.g., openai, anthropic, groq)
        #[arg(short = 't', long)]
        provider_type: String,

        /// Base URL for API requests
        #[arg(short = 'u', long)]
        base_url: String,

        /// API key value (optional, can be added later)
        #[arg(long)]
        api_key: Option<String>,

        /// Models to configure (comma-separated)
        #[arg(long)]
        models: Option<String>,

        /// Set instance as active
        #[arg(long, default_value = "true")]
        active: bool,
    },

    /// Remove a provider instance by ID
    Remove {
        /// Instance ID to remove
        #[arg(short = 'i', long)]
        id: String,

        /// Force removal without confirmation
        #[arg(long)]
        force: bool,
    },

    /// Update an existing provider instance
    Update {
        /// Instance ID to update
        #[arg(short = 'i', long)]
        id: String,

        /// New display name
        #[arg(long)]
        name: Option<String>,

        /// New base URL
        #[arg(long)]
        base_url: Option<String>,

        /// Add or update API key
        #[arg(long)]
        api_key: Option<String>,

        /// Models to set (comma-separated, replaces existing)
        #[arg(long)]
        models: Option<String>,

        /// Set active status
        #[arg(long)]
        active: Option<bool>,
    },

    /// Show detailed information for a specific instance
    Get {
        /// Instance ID to show (positional argument)
        id: String,

        /// Include full secret values (DANGEROUS - use with caution)
        #[arg(long)]
        include_values: bool,
    },

    /// Validate provider instance configurations
    Validate {
        /// Validate specific instance by ID
        #[arg(short = 'i', long)]
        id: Option<String>,

        /// Show all validation errors, not just the first
        #[arg(long)]
        all_errors: bool,
    },
}

fn main() -> Result<()> {
    // Initialize tracing with environment filter
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Scan {
            home: scan_home,
            format,
            include_values,
            only,
            exclude,
            max_bytes_per_file,
            dry_run,
            audit_log,
            verbose,
            update,
        } => handle_scan(
            scan_home.or(cli.home),
            format,
            include_values,
            only,
            exclude,
            max_bytes_per_file,
            dry_run,
            audit_log,
            verbose,
            update,
        ),
        Commands::Providers { verbose } => {
            // Set home directory if provided
            if let Some(home) = &cli.home {
                std::env::set_var("HOME", home);
            }
            handle_providers(verbose)
        }
        Commands::Instances {
            id,
            include_values,
            command,
        } => match (id.as_ref(), command) {
            (Some(instance_id), None) => {
                // If an ID is provided without a subcommand, treat it as a get operation
                handle_get_instance(
                    cli.home.map(PathBuf::from),
                    instance_id.clone(),
                    include_values,
                )
            }
            (None, None) => {
                // Default to list when no subcommand and no ID is provided
                handle_list_instances(cli.home.map(PathBuf::from), false, None, false)
            }
            (
                _,
                Some(InstanceCommands::List {
                    verbose,
                    provider_type,
                    active_only,
                }),
            ) => handle_list_instances(
                cli.home.map(PathBuf::from),
                verbose,
                provider_type,
                active_only,
            ),
            (
                _,
                Some(InstanceCommands::Add {
                    id,
                    name,
                    provider_type,
                    base_url,
                    api_key,
                    models,
                    active,
                }),
            ) => handle_add_instance(id, name, provider_type, base_url, api_key, models, active),
            (_, Some(InstanceCommands::Remove { id, force })) => handle_remove_instance(id, force),
            (
                _,
                Some(InstanceCommands::Update {
                    id,
                    name,
                    base_url,
                    api_key,
                    models,
                    active,
                }),
            ) => handle_update_instance(id, name, base_url, api_key, models, active),
            (_, Some(InstanceCommands::Get { id, include_values })) => {
                handle_get_instance(cli.home.map(PathBuf::from), id, include_values)
            }
            (_, Some(InstanceCommands::Validate { id, all_errors })) => {
                handle_validate_instances(id, all_errors)
            }
        },
        Commands::Version => handle_version(),
    }
}

fn handle_version() -> Result<()> {
    println!(
        "{} {}",
        env!("CARGO_PKG_NAME").green().bold(),
        env!("CARGO_PKG_VERSION").cyan()
    );
    println!("Core library version: {}", env!("CARGO_PKG_VERSION").cyan());
    Ok(())
}
