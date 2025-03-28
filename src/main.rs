mod utils;

use anyhow::{Context, Result};
use async_std::{fs, path::PathBuf, prelude::*};
use clap::Parser;
use colored::Colorize;
use futures::stream::StreamExt;
use ignore::WalkBuilder;
use regex::Regex;
use std::io::{self, Write};

const DEFAULT_OUTPUT_FILE: &str = "code_prompt.txt";

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Output file name
    #[arg(short = 'o', long, default_value = DEFAULT_OUTPUT_FILE)]
    output: String,

    /// Directory to search for files
    #[arg(short = 'd', long, default_value = ".")]
    dir: PathBuf,

    /// Regex patterns to exclude files (comma separated)
    #[arg(short = 'e', long)]
    exclude: Option<String>,

    /// Regex patterns to include files (comma separated)
    #[arg(short = 'i', long)]
    include: Option<String>,

    /// Enable line numbers in output
    #[arg(short = 'l', long, default_value_t = false)]
    line_number: bool,

    /// Respect .gitignore rules
    #[arg(short = 'g', long, default_value_t = true)]
    respect_gitignore: bool,

    /// Show matched files
    #[arg(long, default_value_t = false)]
    show_matched: bool,

    /// Relative path prefix to strip (for better Windows compatibility)
    #[arg(skip = std::path::MAIN_SEPARATOR.to_string() + ".")]
    relative_prefix: String,

    /// Cached compiled regex patterns for exclude
    #[clap(skip)]
    exclude_regexs: Option<Vec<Regex>>,

    /// Cached compiled regex patterns for include
    #[clap(skip)]
    include_regexs: Option<Vec<Regex>>,
}

impl Args {
    /// Check if the file should be included or excluded.
    ///
    /// - `path` is the file path to check.
    /// - `default_include` is the default value to return if no patterns match.
    ///
    /// Returns `true` if the file should be included, `false` otherwise.
    fn should_include(&self, path: &str, default_include: Option<bool>) -> bool {
        let (mut exclude_is_empty, mut include_is_empty) = (true, true);
        if let Some(exclude_patterns) = &self.exclude_regexs {
            exclude_is_empty = exclude_patterns.is_empty();
            if !exclude_is_empty && exclude_patterns.iter().any(|re| re.is_match(path)) {
                return false;
            }
        }

        if let Some(include_patterns) = &self.include_regexs {
            include_is_empty = include_patterns.is_empty();
            if !include_is_empty && include_patterns.iter().any(|re| re.is_match(path)) {
                return true;
            }
        }

        default_include.unwrap_or(exclude_is_empty && include_is_empty)
    }

    /// Convert a path to a platform-independent string format for matching
    fn path_to_normalized_string(&self, path: &PathBuf) -> Option<String> {
        path.to_str().map(|p| {
            // Convert Windows backslashes to forward slashes for consistent matching
            let normalized = p.replace('\\', "/");

            // Strip relative prefix if present
            normalized
                .strip_prefix(&self.relative_prefix)
                .unwrap_or(&normalized)
                .to_string()
        })
    }
}

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

async fn find_files(args: &Args) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    // If respect_gitignore is true, use the ignore crate which handles .gitignore rules
    if args.respect_gitignore {
        // Create a walker that respects .gitignore rules
        let walker = WalkBuilder::new(&args.dir)
            .standard_filters(true) // Use standard filters like .gitignore
            .build();

        for result in walker {
            let entry = match result {
                Ok(entry) => entry,
                Err(_) => continue,
            };

            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let path_buf = PathBuf::from(path);

            // Check if we should include this file
            if let Some(normalized_path) = args.path_to_normalized_string(&path_buf) {
                if args.should_include(&normalized_path, Some(true)) {
                    files.push(path_buf);
                }
            }
        }
    } else {
        let mut entries = fs::read_dir(&args.dir).await?;
        while let Some(entry) = entries.next().await {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir().await {
                let mut subdir_files = Box::pin(find_files(args)).await?;
                files.append(&mut subdir_files);
            } else if path.is_file().await {
                // Check if we should include this file
                if let Some(normalized_path) = args.path_to_normalized_string(&path) {
                    if args.should_include(&normalized_path, None) {
                        files.push(path);
                    }
                }
            }
        }
    }

    Ok(files)
}

#[async_std::main]
async fn main() -> Result<()> {
    let mut args = Args::parse();

    // Set the relative prefix based on platform
    args.relative_prefix = format!("{}{}", std::path::MAIN_SEPARATOR, ".");

    // Cached compiled regex patterns
    if let Some(exclude) = &args.exclude {
        args.exclude_regexs = Some(
            exclude
                .split(',')
                .filter(|p| !p.is_empty())
                .filter_map(|p| Regex::new(p).ok())
                .collect(),
        );
    }
    if let Some(include) = &args.include {
        args.include_regexs = Some(
            include
                .split(',')
                .filter(|p| !p.is_empty())
                .filter_map(|p| Regex::new(p).ok())
                .collect(),
        );
    }

    // Check if the output file exists and confirm overwrite
    let output_path = PathBuf::from(&args.output);
    if output_path.exists().await {
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
    let files = find_files(&args).await?;
    if files.is_empty() {
        println!("No files found matching the criteria.");
        return Ok(());
    } else {
        if args.show_matched {
            println!("\nMatched files:");
            for file in &files {
                if let Some(path_str) = file.to_str() {
                    println!("{}", path_str.green());
                }
            }
        }

        // Show the number of files found
        println!(
            "\nFound {} files matching the criteria.",
            files.len().to_string().green()
        );
    }

    // Process and write files
    let mut output = fs::File::create(&output_path).await?;

    for file_path in &files {
        let content = fs::read_to_string(&file_path)
            .await
            .with_context(|| format!("Failed to read file {:?}", file_path))?;

        // Use normalized path display for consistency across platforms
        if let Some(normalized_path) = args.path_to_normalized_string(file_path) {
            output
                .write_all(format!("{}\n```\n", normalized_path).as_bytes())
                .await?;

            if args.line_number {
                for (i, line) in content.lines().enumerate() {
                    output
                        .write_all(format!("{}\t{}\n", i + 1, line).as_bytes())
                        .await?;
                }
            } else {
                output.write_all(content.as_bytes()).await?;
                if !content.ends_with('\n') {
                    output.write_all(b"\n").await?;
                }
            }

            output.write_all(b"```\n\n").await?;
        }
    }

    // Show summary
    let output_size = fs::metadata(&output_path).await?.len() as f64 / 1024.0;
    // Convert to KB/MB and format
    let output_size = if output_size < 1024.0 {
        format!("{:.1} KB", output_size)
    } else {
        format!("{:.1} MB", output_size / 1024.0)
    };
    println!(
        "==> {} ({})",
        args.output.underline(),
        output_size.to_string().cyan(),
    );

    Ok(())
}
