use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

mod commands;
mod output;

use commands::{list::handle_list, providers::handle_providers, scan::handle_scan};

/// GenAI Key Finder - Discover GenAI API keys and configurations
#[derive(Parser)]
#[command(name = "keyfinder")]
#[command(author, version, about, long_about = None)]
struct Cli {
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

    /// List available providers and scanners
    List {
        /// Show detailed information
        #[arg(long, short = 'v')]
        verbose: bool,
    },

    /// Show available providers and scanners (alias for list)
    Providers {
        /// Show detailed information
        #[arg(long, short = 'v')]
        verbose: bool,
    },

    /// Show version information
    Version,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan {
            home,
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
            home,
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
        Commands::List { verbose } => handle_list(verbose),
        Commands::Providers { verbose } => handle_providers(verbose),
        Commands::Version => handle_version(),
    }
}

fn handle_version() -> Result<()> {
    println!(
        "{} {}",
        env!("CARGO_PKG_NAME").green().bold(),
        env!("CARGO_PKG_VERSION").cyan()
    );
    println!("Core library version: {}", "0.1.0".cyan());
    Ok(())
}
