# paperx

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A modern Rust CLI tool for LaTeX paper management: scaffold, build, watch, and organize academic papers with ease.

## Features

- 🚀 **Quick Setup**: Generate LaTeX paper templates with a single command
- 🔄 **Live Rebuild**: Watch mode automatically rebuilds your PDF when files change
- 📝 **Section Management**: Easily add new sections and figures to your paper
- 🎯 **Multiple Engines**: Support for Tectonic, LaTeXmk, pdfLaTeX, and LuaLaTeX
- 🌍 **Internationalization**: Built-in support for English and Japanese papers
- 📚 **Bibliography**: Integrated BibTeX support with example references
- 🖼️ **Figure Management**: Copy and include figures with proper LaTeX snippets

## Installation

### From Source

```bash
git clone https://github.com/naoki-takata/paperx.git
cd paperx
cargo build --release
```

### Running without installation

```bash
# Run directly with cargo
cargo run --release -- <command>

# Example:
cargo run --release -- build --open
```

## Quick Start

### Create a New Paper

```bash
# Create an English article
cargo run --release -- new my-paper \
  --template article-en \
  --title "Fast Turbulence DNS at Exascale" \
  --author "Jane Smith" \
  --affiliation "Department of Mechanical Engineering, Example University" \
  --keywords "DNS, turbulence, HPC, computational fluid dynamics" \
  --abstract "We demonstrate a novel approach to direct numerical simulation of turbulent flows at exascale computing facilities."

# Create a Japanese article
cargo run --release -- new my-japanese-paper \
  --template ltjs-ja \
  --title "大規模乱流シミュレーションの高速化" \
  --author "田中太郎" \
  --affiliation "工学部機械工学科, サンプル大学" \
  --keywords "乱流, 数値シミュレーション, 高性能計算" \
  --abstract "本論文では、大規模乱流シミュレーションの高速化手法について述べる。"
```

### Build Your Paper

```bash
cd my-paper

# Build once and open the PDF
cargo run --release -- build --open

# Build with a specific engine
cargo run --release -- build --engine latexmk

# Watch for changes and rebuild automatically
cargo run --release -- watch
```

### Add Content

```bash
# Add a new section
cargo run --release -- add section methodology

# Add a figure
cargo run --release -- add figure path/to/plot.png --label fig:results --caption "Simulation results showing velocity contours"
```

## Commands

### `new` - Create a new paper workspace

Creates a new directory with a complete LaTeX paper template.

**Options:**
- `--template`: Template to use (`article-en` or `ltjs-ja`)
- `--title`: Paper title
- `--author`: Author name
- `--affiliation`: Author affiliation
- `--keywords`: Comma-separated keywords
- `--abstract`: Abstract text

### `build` - Build the paper to PDF

Compiles the LaTeX source to PDF.

**Options:**
- `--engine`: LaTeX engine preference (`tectonic`, `latexmk`, `pdflatex`, `lualatex`)
- `--outdir`: Output directory (default: `build`)
- `--open`: Open PDF after build

### `watch` - Watch files and rebuild on change

Monitors the workspace for changes and automatically rebuilds the PDF.

**Options:**
- `--engine`: LaTeX engine preference
- `--outdir`: Output directory

### `add` - Add content to the paper

#### `add section <name>` - Add a new section

Creates a new `.tex` file in `tex/sections/` and includes it in `main.tex`.

#### `add figure <path>` - Add a figure

Copies an image file to `figures/` and prints a LaTeX snippet to include it.

**Options:**
- `--label`: Figure label (default: `fig:example`)
- `--caption`: Figure caption (default: `Caption here.`)

### `open` - Open the built PDF

Opens the most recently built PDF in the default viewer.

### `clean` - Remove build artifacts

Removes the `build/` directory and all generated files.

## Project Structure

When you create a new paper, paperx generates the following structure:

```
my-paper/
├── paperx.toml          # Configuration file
├── README.md            # Project documentation
├── .gitignore           # Git ignore rules
├── tex/
│   ├── main.tex         # Main LaTeX file
│   └── sections/        # Section files
│       └── introduction.tex
├── bib/
│   └── references.bib   # Bibliography file
└── figures/             # Figure files
```

## Configuration

The `paperx.toml` file contains your paper's metadata and build settings:

```toml
main_tex = "tex/main.tex"
engine = "tectonic"
title = "Your Paper Title"
author = "Your Name"
affiliation = "Your Affiliation"
keywords = "keyword1, keyword2"
abstract_text = "Your abstract here."
```

## Requirements

- Rust 1.70+ (for building from source)
- A LaTeX distribution:
  - [Tectonic](https://tectonic-typesetting.github.io/) (recommended)
  - [TeX Live](https://www.tug.org/texlive/)
  - [MiKTeX](https://miktex.org/)

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Clap](https://github.com/clap-rs/clap) for command-line parsing
- Uses [Tectonic](https://tectonic-typesetting.github.io/) as the default LaTeX engine
- Inspired by modern academic paper workflows