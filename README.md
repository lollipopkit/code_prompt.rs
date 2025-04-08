# code_prompt.rs

A command-line tool for gathering source code files into a single consolidated file, making it easy to create code prompts for AI tools, documentation, or sharing.

<div style="height: 210px; width: 400px; overflow: hidden;">
  <img src="https://cdn.lpkt.cn/img/capture/code_prompt.png" alt="code_prompt.rs" width="600"/>
</div>

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/lollipopkit/code_prompt.rs/refs/heads/main/install.sh | bash
```

Or download from the [release page](https://github.com/lollipopkit/code_prompt.rs/releases) manually.

## Usage

```bash
# Basic usage (searches current directory, outputs to code_prompt.txt)
code_prompt

# Specify a different output file
code_prompt -o code_prompt.txt

# Search in a specific directory
code_prompt -d /path/to/project

# Include only specific files
code_prompt -i "*.rs,*.ts"

# Exclude specific files
code_prompt -e "*.png,*.ico,lib/{generated,l10n}*"
```

More attrs can be found via `code_prompt --help`.

## Command Line Options

| Option | Long Form | Description | Default |
|--------|-----------|-------------|---------|
| `-o` | `--output` | Output file name | `code_prompt.txt` |
| `-d` | `--dir` | Directory to search for files | `.` (current directory) |
| `-e` | `--exclude` | Glob patterns to exclude files (comma separated) | none |
| `-i` | `--include` | Glob patterns to include files (comma separated) | none |
| `-l` | `--line-number` | Enable line numbers in output | `false` |
| `-f` | `--standard-filter` | Respect standard filters like .gitignore | `true` |
|  | `--show-matched` | Show matched files | `false` |
|  | `--ignore-comments` | Ignore comment lines | `false` |
|  | `--ignore-empty-lines` | Ignore empty lines | `false` |

## License

```
MIT License
lollipopkit & code_prompt.rs contributors
```

