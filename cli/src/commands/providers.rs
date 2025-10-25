use anyhow::Result;
use colored::*;

pub fn handle_providers(verbose: bool) -> Result<()> {
    println!("\n{}", "Available Providers:".green().bold());

    let providers = vec![
        ("openai", "OpenAI API keys"),
        ("anthropic", "Anthropic (Claude) API keys"),
        ("huggingface", "Hugging Face tokens"),
        ("ollama", "Ollama local configurations"),
        ("litellm", "LiteLLM configurations"),
    ];

    for (name, desc) in providers {
        if verbose {
            println!("  {} - {}", name.cyan(), desc);
        } else {
            println!("  {}", name.cyan());
        }
    }

    println!("\n{}", "Available Application Scanners:".green().bold());

    let scanners = vec![
        ("roo-code", "Roo Code VSCode extension"),
        ("claude-desktop", "Claude Desktop application"),
        ("ragit", "Ragit configurations"),
        ("langchain", "LangChain application configs"),
    ];

    for (name, desc) in scanners {
        if verbose {
            println!("  {} - {}", name.cyan(), desc);
        } else {
            println!("  {}", name.cyan());
        }
    }

    Ok(())
}