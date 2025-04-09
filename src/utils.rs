use async_std::path::Path;
use phf::phf_map;

/// Detects programming language from file extension for syntax highlighting
pub fn detect_language_from_path(path: &Path) -> Option<&'static str> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|s| {
            let ext = s.to_lowercase();
            LANG_EXT_MD_MAP.get(ext.as_str()).map(|lang| *lang)
        })
        .flatten()
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

/// Split patterns by commas while respecting glob brace expressions
///
/// eg:
/// code_prompt --show-matched -e '*.png,*.ico,lib/{generated,l10n}*'
pub fn smart_pattern_split(pattern_str: &str) -> Vec<String> {
    let mut patterns = Vec::new();
    let mut current_pattern = String::new();
    let mut brace_depth = 0;

    for ch in pattern_str.chars() {
        match ch {
            '{' => {
                brace_depth += 1;
                current_pattern.push(ch);
            }
            '}' => {
                if brace_depth > 0 {
                    brace_depth -= 1;
                }
                current_pattern.push(ch);
            }
            ',' if brace_depth == 0 => {
                if !current_pattern.is_empty() {
                    patterns.push(current_pattern);
                    current_pattern = String::new();
                }
            }
            _ => {
                current_pattern.push(ch);
            }
        }
    }

    if !current_pattern.is_empty() {
        patterns.push(current_pattern);
    }

    patterns
}

/// {ext: lang}
static LANG_EXT_MD_MAP: phf::Map<&'static str, &'static str> = phf_map! {
    "rs" => "rust",
    "go" => "go",
    "swift" => "swift",
    "dart" => "dart",
    "js" => "javascript",
    "ts" => "typescript",
    "py" => "python",
    "yaml" => "yaml",
    "yml" => "yaml",
    "xml" => "xml",
    "toml" => "toml",
    "java" => "java",
    "sh" => "bash",
    "html" => "html",
    "htm" => "html",
    "css" => "css",
    "json" => "json",
    "md" => "markdown",
    "sql" => "sql",
    "c" => "c",
    "h" => "c",
    "cpp" => "cpp",
    "hpp" => "cpp",
    "cc" => "cpp",
    "cxx" => "cpp",
    "kt" => "kotlin",
    "kts" => "kotlin",
    "r" => "r",
    "ex" => "elixir",
    "exs" => "elixir",
    "hs" => "haskell",
    "pl" => "perl",
    "cs" => "csharp",
    "fs" => "fsharp",
    "fsx" => "fsharp",
    "rb" => "ruby",
    "php" => "php",
    "csv" => "csv",
    "bat" => "batch",
    "ps1" => "powershell",
    "psm1" => "powershell",
    "psd1" => "powershell",
    "ps1xml" => "powershell",
    "cmd" => "batch",
    "vbs" => "vbscript",
};
