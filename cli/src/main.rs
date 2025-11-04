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

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

mod commands;
mod output;

use commands::{
    labels::{handle_list_labels, handle_set_label, handle_unset_label},
    providers::{
        handle_add_instance, handle_get_instance, handle_list_instances, handle_list_models,
        handle_providers, handle_remove_instance, handle_update_instance,
        handle_validate_instances,
    },
    scan::handle_scan,
    tags::{
        handle_add_tag, handle_assign_tag, handle_list_tags, handle_remove_tag,
        handle_unassign_tag, handle_update_tag,
    },
};

/// AICred - Discover AI API keys and configurations
#[derive(Parser)]
#[command(name = "aicred")]
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

    /// Tag management commands
    Tags {
        #[command(subcommand)]
        command: Option<TagCommands>,
    },

    /// Label management commands
    Labels {
        #[command(subcommand)]
        command: Option<LabelCommands>,
    },

    /// Model management commands
    Models {
        #[command(subcommand)]
        command: Option<ModelCommands>,
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

        /// Filter by tag name
        #[arg(long)]
        tag: Option<String>,

        /// Filter by label name
        #[arg(long)]
        label: Option<String>,
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

#[derive(Subcommand)]
enum TagCommands {
    /// List all tags
    List,

    /// Add a new tag
    Add {
        /// Tag name
        #[arg(short = 'n', long)]
        name: String,

        /// Tag color (hex code)
        #[arg(short = 'c', long)]
        color: Option<String>,

        /// Tag description
        #[arg(short = 'd', long)]
        description: Option<String>,
    },

    /// Remove a tag
    Remove {
        /// Tag name to remove
        #[arg(short = 'n', long)]
        name: String,

        /// Force removal without confirmation
        #[arg(long)]
        force: bool,
    },

    /// Update a tag
    Update {
        /// Tag name to update
        #[arg(short = 'n', long)]
        name: String,

        /// New tag color
        #[arg(short = 'c', long)]
        color: Option<String>,

        /// New tag description
        #[arg(short = 'd', long)]
        description: Option<String>,
    },

    /// Assign a tag to an instance or model
    Assign {
        /// Tag name to assign
        #[arg(short = 'n', long)]
        name: String,

        /// Instance ID
        #[arg(short = 'i', long)]
        instance: Option<String>,

        /// Model ID (requires instance ID)
        #[arg(short = 'm', long)]
        model: Option<String>,
    },

    /// Unassign a tag from an instance or model
    Unassign {
        /// Tag name to unassign
        #[arg(short = 'n', long)]
        name: String,

        /// Instance ID
        #[arg(short = 'i', long)]
        instance: Option<String>,

        /// Model ID (requires instance ID)
        #[arg(short = 'm', long)]
        model: Option<String>,
    },
}

#[derive(Subcommand)]
enum LabelCommands {
    /// List all label assignments
    List,

    /// Set (create or update) a label assignment
    Set {
        /// Label assignment in format: label=provider:model
        #[arg(index = 1, required = true)]
        assignment: String,

        /// Label color (hex code)
        #[arg(short = 'c', long)]
        color: Option<String>,

        /// Label description
        #[arg(short = 'd', long)]
        description: Option<String>,
    },

    /// Unset (remove) a label assignment
    Unset {
        /// Label name to remove
        #[arg(index = 1, required = true)]
        name: String,

        /// Force removal without confirmation
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
enum ModelCommands {
    /// List all models with their configurations
    List {
        /// Show detailed information
        #[arg(long, short = 'v')]
        verbose: bool,

        /// Filter by provider type (e.g., openai, anthropic)
        #[arg(long)]
        provider_type: Option<String>,

        /// Filter by tag name
        #[arg(long)]
        tag: Option<String>,

        /// Filter by label name
        #[arg(long)]
        label: Option<String>,
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
                handle_list_instances(cli.home.map(PathBuf::from), false, None, false, None, None)
            }
            (
                _,
                Some(InstanceCommands::List {
                    verbose,
                    provider_type,
                    active_only,
                    tag,
                    label,
                }),
            ) => handle_list_instances(
                cli.home.map(PathBuf::from),
                verbose,
                provider_type,
                active_only,
                tag,
                label,
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
        Commands::Tags { command } => match command {
            Some(TagCommands::List) => handle_list_tags(),
            Some(TagCommands::Add {
                name,
                color,
                description,
            }) => handle_add_tag(name, color, description),
            Some(TagCommands::Remove { name, force }) => handle_remove_tag(name, force),
            Some(TagCommands::Update {
                name,
                color,
                description,
            }) => handle_update_tag(name, color, description),
            Some(TagCommands::Assign {
                name,
                instance,
                model,
            }) => handle_assign_tag(name, instance, model),
            Some(TagCommands::Unassign {
                name,
                instance,
                model,
            }) => handle_unassign_tag(name, instance, model),
            None => handle_list_tags(),
        },
        Commands::Labels { command } => match command {
            Some(LabelCommands::List) => handle_list_labels(),
            Some(LabelCommands::Set {
                assignment,
                color,
                description,
            }) => {
                // Parse assignment format: label=provider:model
                let parts: Vec<&str> = assignment.split('=').collect();
                if parts.len() != 2 {
                    return Err(anyhow::anyhow!(
                        "Assignment format must be 'label=provider:model', e.g., 'thinking=openrouter:deepseek-v3.2-exp'"
                    ));
                }
                let label_name = parts[0].trim().to_string();
                let tuple_str = parts[1].trim().to_string();
                handle_set_label(label_name, tuple_str, color, description)
            }
            Some(LabelCommands::Unset { name, force }) => handle_unset_label(name, force),
            None => handle_list_labels(),
        },
        Commands::Models { command } => match command {
            Some(ModelCommands::List {
                verbose,
                provider_type,
                tag,
                label,
            }) => handle_list_models(
                cli.home.map(PathBuf::from),
                verbose,
                provider_type,
                tag,
                label,
            ),
            None => handle_list_models(cli.home.map(PathBuf::from), false, None, None, None),
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
