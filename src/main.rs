mod args;
mod utils;

use crate::args::Args;
use anyhow::{Context, Result};
use async_std::{fs, path::PathBuf, prelude::*};
use clap::Parser;
use colored::Colorize;
use std::io::{self, Write};

/// Ask the user for confirmation to continue
fn ask_continue(message: &str, default_value: bool) -> Result<bool> {
    let prompt = if default_value {
        format!("{} [Y/n]: ", message)
    } else {
        format!("{} [y/N]: ", message)
    };

    print!("{}", prompt);
    io::stdout().flush()?;

    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;

    let answer = answer.trim();
    if answer.is_empty() {
        return Ok(default_value);
    }

    let lower_answer = answer.to_lowercase();
    Ok(lower_answer == "y" || lower_answer == "yes")
}

#[async_std::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Check if the output file exists and confirm overwrite
    let output_path = PathBuf::from(&args.output);
    if output_path.exists().await && !args.skip_confirm {
        let message = format!(
            "Output file {} already exists.\n{}?",
            args.output.yellow(),
            "Overwrite".red(),
        );

        if !ask_continue(&message, true)? {
            println!("{}", "Aborted.".red());
            return Ok(());
        }

        fs::remove_file(&output_path)
            .await
            .context("Failed to delete existing output file")?;
    }

    // Find all files according to criteria
    let files = args.find_files().await?;
    if files.is_empty() {
        println!("No files found matching the criteria.");
        return Ok(());
    } else if args.show_matched {
        println!("\nMatched files:");
    }

    // Process and write files
    let mut output = fs::File::create(&output_path).await?;

    for file_path in &files {
        let content = fs::read_to_string(&file_path)
            .await
            .with_context(|| format!("Failed to read file {:?}", file_path))?;

        let buf = args.write_buffer(&file_path, &content).await?;
        output.write_all(buf.as_bytes()).await?;

        if args.show_matched {
            let size = fs::metadata(file_path).await.map(|e| e.len() as f64);
            // Show matched file path
            println!(
                "{}: {}",
                file_path.display(),
                size.map_or_else(|_| "N/A".to_string(), |s| utils::format_file_size(s))
            );
        }
    }

    // Show the number of files found
    println!(
        "\nFound {} files matching the criteria.",
        files.len().to_string().green()
    );

    // Show summary with improved size formatting
    let output_size = fs::metadata(&output_path).await?.len() as f64;
    let output_size = utils::format_file_size(output_size);
    println!("==> {} ({})", args.output.underline(), output_size.cyan(),);

    Ok(())
}
