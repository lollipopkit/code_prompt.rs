mod utils;

use anyhow::{Context, Result};
use async_std::{fs, path::PathBuf, prelude::*};
use clap::Parser;
use colored::Colorize;
use ignore::{
    overrides::{Override, OverrideBuilder},
    WalkBuilder,
};
use std::io::{self, Write};
use utils::{get_comment_prefix, smart_pattern_split};

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

    /// Glob patterns to exclude files (comma separated)
    #[arg(short = 'e', long)]
    exclude: Option<String>,

    /// Glob patterns to include files (comma separated)
    #[arg(short = 'i', long)]
    include: Option<String>,

    /// Enable line numbers in output
    #[arg(short = 'l', long, default_value_t = false)]
    line_number: bool,

    /// Respect standard filters like .gitignore
    #[arg(short = 'f', long, default_value_t = true)]
    standard_filter: bool,

    /// Show matched files
    #[arg(long, default_value_t = false)]
    show_matched: bool,

    /// Relative path prefix to strip (for better Windows compatibility)
    #[arg(skip = std::path::MAIN_SEPARATOR.to_string() + ".")]
    relative_prefix: String,

    /// Ignore comments lines
    #[arg(long, default_value_t = false)]
    ignore_comments: bool,

    /// Ignore empty lines
    #[arg(long, default_value_t = false)]
    ignore_empty_lines: bool,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            output: DEFAULT_OUTPUT_FILE.to_string(),
            dir: PathBuf::from("."),
            exclude: None,
            include: None,
            line_number: false,
            standard_filter: true,
            show_matched: false,
            relative_prefix: String::new(),
            ignore_comments: false,
            ignore_empty_lines: false,
        }
    }
}

impl Args {
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

    /// Build overrides based on include and exclude patterns
    fn build_overrides(&self) -> Result<Option<Override>> {
        let mut builder = OverrideBuilder::new(&self.dir);

        let mut has_patterns = false;

        if let Some(include) = &self.include {
            for pattern in smart_pattern_split(include)
                .into_iter()
                .filter(|p| !p.is_empty())
            {
                builder.add(&pattern)?;
                has_patterns = true;
            }
        }

        if let Some(exclude) = &self.exclude {
            for pattern in smart_pattern_split(exclude)
                .into_iter()
                .filter(|p| !p.is_empty())
            {
                builder.add(&format!("!{}", pattern))?;
                has_patterns = true;
            }
        }

        if has_patterns {
            Ok(Some(builder.build()?))
        } else {
            Ok(None)
        }
    }

    /// Find files in the specified directory based on include/exclude patterns
    /// and standard filters
    async fn find_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let overrides = self.build_overrides()?;

        let mut builder = WalkBuilder::new(&self.dir);
        builder.standard_filters(self.standard_filter);

        if let Some(overrides) = overrides {
            builder.overrides(overrides);
        }

        let walked_files = builder.build();

        for result in walked_files {
            let entry = match result {
                Ok(entry) => entry,
                Err(_) => continue,
            };

            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let path_buf = PathBuf::from(path);
            files.push(path_buf);
        }

        Ok(files)
    }
}

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
    let mut args = Args::parse();

    // Set the relative prefix based on platform
    args.relative_prefix = format!("{}{}", std::path::MAIN_SEPARATOR, ".");

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

        // Use normalized path display for consistency across platforms
        if let Some(normalized_path) = args.path_to_normalized_string(file_path) {
            // Detect language for syntax highlighting
            let language = utils::detect_language_from_path(&normalized_path);
            let code_block_start = format!("{}\n```{}\n", normalized_path, language);
            output.write_all(code_block_start.as_bytes()).await?;

            let comment_prefix = get_comment_prefix(&file_path);

            // Use a buffer to reduce write operations
            let mut buffer = String::with_capacity(content.len() + content.lines().count() * 8);
            let lines = content.lines().enumerate();

            for (i, line) in lines {
                // Skip lines by rules
                if line.is_empty() && args.ignore_empty_lines {
                    continue;
                }
                if args.ignore_comments {
                    if let Some(prefix) = &comment_prefix {
                        if line.trim_start().starts_with(prefix) {
                            continue;
                        }
                    }
                }

                // Write line to buffer
                if args.line_number {
                    // DO NOT trim_end(), keep the original line endings
                    buffer.push_str(&format!("{}\t{}\n", i + 1, line));
                } else {
                    buffer.push_str(line);
                    if !line.ends_with('\n') {
                        buffer.push('\n');
                    }
                }
            }

            buffer.push_str("```\n\n");
            output.write_all(buffer.as_bytes()).await?;

            if args.show_matched {
                let size = fs::metadata(file_path).await.map(|e| e.len() as f64);
                // Show matched file path
                println!(
                    "{}: {}",
                    normalized_path.underline(),
                    size.map_or_else(|_| "N/A".to_string(), |s| utils::format_file_size(s))
                );
            }
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
