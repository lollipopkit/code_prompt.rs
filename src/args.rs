use anyhow::Result;
use async_std::path::{Path, PathBuf};
use clap::Parser;
use ignore::{
    overrides::{Override, OverrideBuilder},
    WalkBuilder,
};

use crate::utils;

const DEFAULT_OUTPUT_FILE: &str = "code_prompt.md";

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    /// Output file name
    #[arg(short = 'o', long, default_value = DEFAULT_OUTPUT_FILE)]
    pub output: String,

    /// Directory to search for files
    #[arg(short = 'd', long, default_value = ".")]
    pub dir: PathBuf,

    /// Glob patterns to exclude files (comma separated)
    #[arg(short = 'e', long)]
    pub exclude: Option<String>,

    /// Glob patterns to include files (comma separated)
    #[arg(short = 'i', long)]
    pub include: Option<String>,

    /// Enable line numbers in output
    #[arg(short = 'l', long, default_value_t = false)]
    pub line_number: bool,

    /// Respect standard filters like .gitignore
    #[arg(short = 'f', long, default_value_t = true)]
    pub standard_filter: bool,

    /// Show matched files
    #[arg(long, default_value_t = false)]
    pub show_matched: bool,

    /// Ignore empty lines
    #[arg(long, default_value_t = false)]
    pub ignore_empty_lines: bool,

    /// Skip confirmation for overwriting output file
    #[arg(long, default_value_t = false)]
    pub skip_confirm: bool,
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
            ignore_empty_lines: false,
            skip_confirm: false,
        }
    }
}

impl Args {
    /// Build overrides based on include and exclude patterns
    fn build_overrides(&self) -> Result<Option<Override>> {
        let mut builder = OverrideBuilder::new(&self.dir);

        let mut has_patterns = false;

        if let Some(include) = &self.include {
            for pattern in utils::smart_pattern_split(include)
                .into_iter()
                .filter(|p| !p.is_empty())
            {
                builder.add(&pattern)?;
                has_patterns = true;
            }
        }

        if let Some(exclude) = &self.exclude {
            for pattern in utils::smart_pattern_split(exclude)
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
    pub async fn find_files(&self) -> Result<Vec<PathBuf>> {
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

    /// Write the content of a file to a buffer with optional line numbers
    /// and syntax highlighting
    /// 
    /// - `path`: The path of the file
    /// - `content`: The content of the file
    /// - `size`: The size of the file in bytes
    /// 
    /// Returns the buffer as a String
    pub async fn write_buffer(&self, path: &Path, content: &String, size: usize) -> Result<String> {
        // Use a buffer to reduce write operations
        let mut buffer = String::with_capacity(size);
        buffer.push_str(&format!("## {}\n\n", path.display()));

        // Detect language for syntax highlighting
        let language = utils::detect_language_from_path(&path);
        if let Some(lang) = language {
            buffer.push_str("```");
            buffer.push_str(&lang);
            buffer.push_str("\n");
        } else {
            buffer.push_str("```\n");
        }

        let lines = content.lines().enumerate();
        for (i, line) in lines {
            // Skip lines by rules
            if line.is_empty() && self.ignore_empty_lines {
                continue;
            }

            // Write line to buffer
            if self.line_number {
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

        Ok(buffer)
    }
}
