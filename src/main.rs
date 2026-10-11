use clap::{Parser, Subcommand};
use colored::*;
use openlight::agent::OpenLightAgent;
use openlight::config::OpenLightConfig;
use std::io::{self, Write};

#[derive(Parser)]
#[command(name = "openlight")]
#[command(version = "0.1.0")]
#[command(about = "OpenLight: High-performance Autonomous AI Agent in Rust with Screen Control (CUA) and Hermes Loop", long_about = None)]
struct Cli {
    /// Task prompt to execute in single-shot mode
    #[arg(short, long)]
    prompt: Option<String>,

    /// LLM provider override: gemini, openai, anthropic, openrouter, ollama
    #[arg(long)]
    provider: Option<String>,

    /// Model name override
    #[arg(long)]
    model: Option<String>,

    /// Monitor index to capture/control (default: 1)
    #[arg(long, default_value_t = 1)]
    monitor: usize,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage OpenLight configuration and API keys
    Config {
        #[arg(long)]
        set_key: Option<String>,
        #[arg(long)]
        value: Option<String>,
    },
}

fn print_banner(provider: &str, model: &str) {
    println!("{}", "============================================================".cyan());
    println!(
        "{} {}",
        "🌟 OpenLight v0.1.0 (Rust Edition)".bright_cyan().bold(),
        "- Screen Control & Hermes Agent".dimmed()
    );
    println!(
        "Provider: {} | Model: {}",
        provider.yellow().bold(),
        model.green().bold()
    );
    println!("{}", "============================================================".cyan());
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();
    let mut config = OpenLightConfig::load();

    if let Some(Commands::Config { set_key, value }) = cli.command {
        if let (Some(k), Some(v)) = (set_key, value) {
            match k.as_str() {
                "default_provider" => config.default_provider = v,
                "gemini_api_key" => config.gemini_api_key = Some(v),
                "openai_api_key" => config.openai_api_key = Some(v),
                "anthropic_api_key" => config.anthropic_api_key = Some(v),
                "openrouter_api_key" => config.openrouter_api_key = Some(v),
                other => eprintln!("Unknown config key: {other}"),
            }
            if let Err(e) = config.save() {
                eprintln!("Failed to save config: {e}");
            } else {
                println!("{}", "Configuration saved successfully.".green());
            }
        } else {
            println!("{}", "Current OpenLight Configuration:".bright_cyan().bold());
            println!("  default_provider: {}", config.default_provider);
            println!("  gemini_model: {}", config.gemini_model);
            println!("  openai_model: {}", config.openai_model);
            println!("  anthropic_model: {}", config.anthropic_model);
            println!("  gemini_api_key: {}", if config.gemini_api_key.is_some() { "********" } else { "None" });
            println!("  openai_api_key: {}", if config.openai_api_key.is_some() { "********" } else { "None" });
            println!("  anthropic_api_key: {}", if config.anthropic_api_key.is_some() { "********" } else { "None" });
        }
        return Ok(());
    }

    if cli.monitor > 0 {
        config.default_monitor = cli.monitor;
    }

    let mut agent = match OpenLightAgent::new(
        config.clone(),
        cli.provider.as_deref(),
        cli.model.as_deref(),
        true,
    ) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{} {e}", "Initialization Error:".red().bold());
            eprintln!("{}", "Set your API key (e.g., $env:GEMINI_API_KEY='...') or run 'openlight config'".dimmed());
            std::process::exit(1);
        }
    };

    print_banner(agent.provider.provider_name(), agent.provider.model_name());

    if let Some(prompt) = cli.prompt {
        println!("{} {}\n", "Goal:".bright_green().bold(), prompt);
        match agent.run_task(&prompt).await {
            Ok(answer) => {
                println!("\n{}\n{}", "OpenLight Response:".bright_cyan().bold(), answer);
            }
            Err(e) => {
                eprintln!("\n{} {e}", "Error:".red().bold());
            }
        }
        return Ok(());
    }

    println!("{}", "Interactive session started. Type 'exit' to quit.\n".dimmed());

    let stdin = io::stdin();
    loop {
        print!("{}", "openlight > ".bright_cyan().bold());
        io::stdout().flush()?;

        let mut input = String::new();
        if stdin.read_line(&mut input)? == 0 {
            break;
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("quit") {
            println!("{}", "Goodbye!".dimmed());
            break;
        }

        match agent.run_task(trimmed).await {
            Ok(answer) => {
                println!("\n{}\n{}", "OpenLight Response:".bright_cyan().bold(), answer);
            }
            Err(e) => {
                eprintln!("\n{} {e}", "Error:".red().bold());
            }
        }
        println!();
    }

    Ok(())
}
