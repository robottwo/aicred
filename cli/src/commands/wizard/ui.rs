//! UI components and helpers for the wizard

use anyhow::Result;
use console::{style, Term};
use std::io::Write;
use std::path::Path;

/// Show the welcome screen
pub fn show_welcome() -> Result<()> {
    let term = Term::stdout();
    term.clear_screen()?;
    
    println!("{}", style("╭─────────────────────────────────────────────────────────╮").dim());
    println!("{}", style("│                                                         │").dim());
    println!("{}  {} {}", 
        style("│").dim(),
        style("🚀 Welcome to AICred Setup Wizard").cyan().bold(),
        style("│").dim()
    );
    println!("{}", style("│                                                         │").dim());
    println!("{}", style("╰─────────────────────────────────────────────────────────╯").dim());
    println!();
    
    println!("This wizard will help you:");
    println!("  {} Discover existing AI credentials on your system", style("•").cyan());
    println!("  {} Import provider configurations", style("•").cyan());
    println!("  {} Set up default models for common tasks", style("•").cyan());
    println!();
    
    println!("We'll scan these locations:");
    println!("  {} ~/.config/roo-code/", style("•").dim());
    println!("  {} ~/Library/Application Support/Claude/", style("•").dim());
    println!("  {} Environment variables (OPENAI_API_KEY, etc.)", style("•").dim());
    println!("  {} ~/.env files", style("•").dim());
    println!();
    
    println!("{}", style("No API keys will be displayed in plain text.").yellow());
    println!();
    
    use inquire::Confirm;
    let proceed = Confirm::new("Ready to start?")
        .with_default(true)
        .prompt()?;
    
    if !proceed {
        println!("{}", style("Setup cancelled.").yellow());
        std::process::exit(0);
    }
    
    println!();
    Ok(())
}

/// Show a section header
pub fn section_header(title: &str) {
    println!();
    println!("{}", style("╭─────────────────────────────────────────────────────────────────────╮").dim());
    println!("{}  {}", 
        style("│").dim(),
        style(title).cyan().bold()
    );
    println!("{}", style("╰─────────────────────────────────────────────────────────────────────╯").dim());
    println!();
}

/// Show success screen
pub fn show_success(config_path: &Path, instances: usize, labels: usize) -> Result<()> {
    println!();
    println!("{}", style("╭─────────────────────────────────────────────────────────╮").green());
    println!("{}  {} {}", 
        style("│").green(),
        style("✓ Configuration Complete!").green().bold(),
        style("                              │").green()
    );
    println!("{}", style("╰─────────────────────────────────────────────────────────╯").green());
    println!();
    
    println!("Your AICred configuration has been saved to:");
    println!("  {}", style(config_path.display()).cyan());
    println!();
    
    println!("Summary:");
    println!("  {} {} provider instances configured", style("•").green(), instances);
    println!("  {} {} labels created", style("•").green(), labels);
    println!();
    
    println!("{}", style("What's next?").bold());
    println!();
    println!("Try these commands:");
    println!();
    println!("  {} View your providers", style("aicred instances list").cyan());
    println!("  {} Check your labels", style("aicred labels list").cyan());
    println!("  {} Use the wrap command", style("aicred wrap --labels fast -- python script.py").cyan());
    println!();
    println!("For more help, run: {}", style("aicred --help").cyan());
    println!();
    println!("{}", style("Happy building! 🚀").green().bold());
    println!();
    
    Ok(())
}

/// Show a progress spinner with message
pub fn show_progress(message: &str) {
    print!("{} {}... ", style("⠋").cyan(), message);
    std::io::stdout().flush().unwrap_or(());
}

/// Complete a progress message
pub fn complete_progress(message: &str) {
    println!("{} {}", style("✓").green(), message);
}

/// Show an error message
pub fn show_error(message: &str) {
    eprintln!("{} {}", style("✗").red(), style(message).red());
}

/// Show a warning message
pub fn show_warning(message: &str) {
    println!("{} {}", style("⚠").yellow(), style(message).yellow());
}

/// Show an info message
pub fn show_info(message: &str) {
    println!("{} {}", style("ℹ").blue(), message);
}
