use std::path::Path;

/// Detects programming language from file extension for syntax highlighting
pub fn detect_language_from_path(path_str: &str) -> String {
    let path = Path::new(path_str);

    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => match ext.to_lowercase().as_str() {
            "rs" => "rust",
            "js" => "javascript",
            "ts" => "typescript",
            "py" => "python",
            "java" => "java",
            "c" | "h" => "c",
            "cpp" | "hpp" | "cc" | "cxx" => "cpp",
            "go" => "go",
            "rb" => "ruby",
            "php" => "php",
            "sh" => "bash",
            "html" | "htm" => "html",
            "css" => "css",
            "json" => "json",
            "md" => "markdown",
            "sql" => "sql",
            "yaml" | "yml" => "yaml",
            "xml" => "xml",
            "toml" => "toml",
            "kt" | "kts" => "kotlin",
            "swift" => "swift",
            "dart" => "dart",
            "r" => "r",
            "ex" | "exs" => "elixir",
            "hs" => "haskell",
            "pl" => "perl",
            "cs" => "csharp",
            "fs" | "fsx" => "fsharp",
            "scala" => "scala",
            _ => "",
        },
        None => "",
    }
    .to_string()
}

/// Formats file size to human-readable format with appropriate units (B, KB, MB, GB)
pub fn format_file_size(size_in_bytes: f64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    if size_in_bytes < KB {
        format!("{:.0} B", size_in_bytes)
    } else if size_in_bytes < MB {
        format!("{:.1} KB", size_in_bytes / KB)
    } else if size_in_bytes < GB {
        format!("{:.1} MB", size_in_bytes / MB)
    } else {
        format!("{:.2} GB", size_in_bytes / GB)
    }
}
