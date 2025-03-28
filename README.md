# code_prompt.rs

A command-line tool for gathering source code files into a single consolidated file, making it easy to create code prompts for AI tools, documentation, or sharing.

## Features

- Search for files in specified directories
- Include or exclude files using regex patterns
- Respect `.gitignore` rules
- Add line numbers to output
- Show matched files before processing
- Format output with file paths and code fences

## Installation

### From source

```bash
# Clone the repository
git clone https://github.com/yourusername/code_prompt.rs.git
cd code_prompt.rs

# Build and install using cargo
cargo install --path .
```

## Usage

```bash
# Basic usage (searches current directory, outputs to code_prompt.txt)
code_prompt.rs

# Specify a different output file
code_prompt.rs -o code_prompt.txt

# Search in a specific directory
code_prompt.rs -d /path/to/project

# Include only rust and typescript files
code_prompt.rs -i "\.rs$,\.ts$"

# Exclude test files and temporary files
code_prompt.rs -e "test_.*\.rs$,\.tmp$"

# Include line numbers in the output
code_prompt.rs -l

# Don't respect .gitignore rules
code_prompt.rs -g false

# Show matched files
code_prompt.rs --show-matched
```

## Command Line Options

| Option | Long Form | Description | Default |
|--------|-----------|-------------|---------|
| `-o` | `--output` | Output file name | `code_prompt.txt` |
| `-d` | `--dir` | Directory to search for files | `.` (current directory) |
| `-e` | `--exclude` | Regex patterns to exclude files (comma separated) | none |
| `-i` | `--include` | Regex patterns to include files (comma separated) | none |
| `-l` | `--line-number` | Enable line numbers in output | `false` |
| `-g` | `--respect-gitignore` | Respect .gitignore rules | `true` |
|  | `--show-matched` | Show matched files | `false` |

## License

```
MIT License
lollipopkit & code_prompt.rs contributors
```

