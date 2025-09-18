use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use notify::{recommended_watcher, RecursiveMode, Watcher};
use open;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use toml;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(name = "paperx", version, about = "Rust LaTeX paper toolkit")]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new paper workspace
    New {
        /// Directory name for the new paper
        name: String,
        /// Template to use
        #[arg(long, value_enum, default_value_t = Template::ArticleEn)]
        template: Template,
        /// Paper title
        #[arg(long, default_value = "Untitled Paper")]
        title: String,
        /// Author name
        #[arg(long, default_value = "First Last")]
        author: String,
        /// Affiliation line
        #[arg(long, default_value = "Affiliation")]
        affiliation: String,
        /// Comma-separated keywords
        #[arg(long, default_value = "keyword1, keyword2")]
        keywords: String,
        /// Abstract text
        #[arg(long, default_value = "This is the abstract.")]
        r#abstract: String,
    },

    /// Build the paper to PDF
    Build {
        /// Engine preference
        #[arg(long, value_enum, default_value_t = EnginePref::Tectonic)]
        engine: EnginePref,
        /// Output directory
        #[arg(long, default_value = "build")]
        outdir: String,
        /// Open PDF after build
        #[arg(long, default_value_t = false)]
        open: bool,
    },

    /// Watch files and rebuild on change
    Watch {
        #[arg(long, value_enum, default_value_t = EnginePref::Tectonic)]
        engine: EnginePref,
        #[arg(long, default_value = "build")]
        outdir: String,
    },

    /// Add a new section under tex/sections and include it in main.tex
    Add {
        #[command(subcommand)]
        sub: AddSub,
    },

    /// Open the built PDF
    Open {},

    /// Remove build artifacts
    Clean {},
}

#[derive(Subcommand, Debug)]
enum AddSub {
    /// Create tex/sections/<name>.tex and include it from main.tex
    Section { name: String },
    /// Copy figure to figures/ and print a LaTeX snippet to include it
    Figure {
        /// Path to an existing image (png/jpg/pdf/svg etc.)
        path: String,
        /// Optional label
        #[arg(long)]
        label: Option<String>,
        /// Optional caption
        #[arg(long)]
        caption: Option<String>,
    },
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum Template { ArticleEn, LtjsJa }

#[derive(Copy, Clone, Debug, ValueEnum)]
enum EnginePref { Tectonic, Latexmk, Pdflatex, Lualatex }

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Config {
    main_tex: String,
    engine: String,
    title: String,
    author: String,
    affiliation: String,
    keywords: String,
    abstract_text: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Commands::New { name, template, title, author, affiliation, keywords, r#abstract } => {
            cmd_new(&name, template, &title, &author, &affiliation, &keywords, &r#abstract)?;
        }
        Commands::Build { engine, outdir, open } => {
            let pdf = cmd_build(engine, &outdir)?;
            if open { open::that(pdf)?; }
        }
        Commands::Watch { engine, outdir } => {
            cmd_watch(engine, &outdir)?;
        }
        Commands::Add { sub } => match sub {
            AddSub::Section { name } => cmd_add_section(&name)?,
            AddSub::Figure { path, label, caption } => cmd_add_figure(&path, label.as_deref(), caption.as_deref())?,
        },
        Commands::Open {} => {
            let pdf = default_pdf_path()?;
            open::that(pdf)?;
        }
        Commands::Clean {} => {
            if Path::new("build").exists() { fs::remove_dir_all("build").ok(); }
            println!("Cleaned build/");
        }
    }
    Ok(())
}

fn cmd_new(name: &str, template: Template, title: &str, author: &str, affiliation: &str, keywords: &str, abs: &str) -> Result<()> {
    let root = Path::new(name);
    if root.exists() { return Err(anyhow!("Directory '{}' already exists", name)); }
    fs::create_dir_all(root.join("tex/sections"))?;
    fs::create_dir_all(root.join("bib"))?;
    fs::create_dir_all(root.join("figures"))?;

    // Files
    write(&root.join(".gitignore"), GITIGNORE)?;
    write(&root.join("README.md"), README_PROJECT)?;
    write(&root.join("bib/references.bib"), BIB_TEMPLATE)?;

    let main_tex = match template {
        Template::ArticleEn => TEMPLATE_ARTICLE_EN
            .replace("${TITLE}", title)
            .replace("${AUTHOR}", author)
            .replace("${AFFIL}", affiliation)
            .replace("${KEYWORDS}", keywords)
            .replace("${ABSTRACT}", abs),
        Template::LtjsJa => TEMPLATE_LTJS_JA
            .replace("${TITLE}", title)
            .replace("${AUTHOR}", author)
            .replace("${AFFIL}", affiliation)
            .replace("${KEYWORDS}", keywords)
            .replace("${ABSTRACT}", abs),
    };
    write(&root.join("tex/main.tex"), &main_tex)?;
    write(&root.join("tex/sections/introduction.tex"), SECTION_INTRO)?;

    let cfg = Config {
        main_tex: "tex/main.tex".into(),
        engine: "tectonic".into(),
        title: title.into(),
        author: author.into(),
        affiliation: affiliation.into(),
        keywords: keywords.into(),
        abstract_text: abs.into(),
    };
    let cfg_toml = toml::to_string_pretty(&cfg)?;
    write(&root.join("paperx.toml"), &cfg_toml)?;

    println!("✅ Created paper workspace at '{}'.\nNext: \n  cd {}\n  cargo run --release -- build --open", name, name);
    Ok(())
}

fn cmd_build(engine_pref: EnginePref, outdir: &str) -> Result<PathBuf> {
    let cfg = read_config()?;
    let main = PathBuf::from(&cfg.main_tex);
    if !main.exists() { return Err(anyhow!("Main tex not found: {}", cfg.main_tex)); }

    fs::create_dir_all(outdir)?;
    let engine = pick_engine(engine_pref)?;
    println!("Using engine: {}", engine);

    let pdf_path = Path::new(outdir).join("main.pdf");

    match engine.as_str() {
        "tectonic" => {
            // tectonic -X compile tex/main.tex --outdir build --keep-logs --keep-intermediates
            run(Command::new("tectonic")
                .args(["-X","compile"])
                .arg(&cfg.main_tex)
                .args(["--outdir", outdir, "--keep-logs", "--keep-intermediates"]))?;
        }
        "latexmk" => {
            run(Command::new("latexmk")
                .args(["-pdf","-interaction=nonstopmode"])
                .arg(format!("-output-directory={}", outdir))
                .arg(&cfg.main_tex))?;
        }
        "pdflatex" => {
            run(Command::new("pdflatex")
                .arg(format!("-output-directory={}", outdir))
                .arg(&cfg.main_tex))?;
        }
        "lualatex" => {
            run(Command::new("lualatex")
                .arg(format!("-output-directory={}", outdir))
                .arg(&cfg.main_tex))?;
        }
        other => return Err(anyhow!("Unknown engine: {}", other)),
    }

    if !pdf_path.exists() {
        // latexmk places PDF alongside outdir/main.pdf; tectonic does too. If not, try fallback.
        let alt = guess_pdf_path(outdir)?;
        return Ok(alt);
    }
    Ok(pdf_path)
}

fn cmd_watch(engine: EnginePref, outdir: &str) -> Result<()> {
    let last = Arc::new(Mutex::new(Instant::now() - Duration::from_secs(10)));
    let debounce = Duration::from_millis(400);
    let outdir = outdir.to_string(); // Convert to owned String
    let last_clone = last.clone();

    let mut w = recommended_watcher(move |res: notify::Result<notify::Event>| {
        match res {
            Ok(_evt) => {
                if let Ok(last_instant) = last_clone.lock() {
                    if last_instant.elapsed() > debounce {
                        drop(last_instant);
                        if let Ok(mut last_instant) = last_clone.lock() {
                            *last_instant = Instant::now();
                        }
                        println!("\n🔁 Change detected. Rebuilding...");
                        if let Err(e) = cmd_build(engine, &outdir) { eprintln!("build error: {e:#}"); }
                    }
                }
            }
            Err(e) => eprintln!("watch error: {e}"),
        }
    })?;

    for p in ["tex", "bib", "figures"].iter() {
        if Path::new(p).exists() {
            w.watch(Path::new(p), RecursiveMode::Recursive)?;
        }
    }

    println!("Watching tex/, bib/, figures/ — press Ctrl+C to stop.");
    // Block forever
    loop { std::thread::sleep(Duration::from_secs(3600)); }
}

fn cmd_add_section(name: &str) -> Result<()> {
    let path = Path::new("tex/sections").join(format!("{}.tex", name));
    if path.exists() { return Err(anyhow!("Section already exists: {}", path.display())); }
    fs::create_dir_all(path.parent().unwrap())?;
    write(&path, &format!("% Section: {n}\n\\section{{{N}}}\nWrite here.\n", n=name, N = titleize(name)))?;

    // Append to main.tex after marker or before \end{document}
    let main_path = Path::new("tex/main.tex");
    let mut main = fs::read_to_string(&main_path).context("read main.tex")?;
    let include_line = format!("% paperx:include\n\\input{{sections/{}}}\n", path.file_name().unwrap().to_string_lossy());
    let marker = Regex::new(r"(?m)^%\s*paperx:sections\s*$").unwrap();
    if marker.is_match(&main) {
        main = marker.replace(&main, format!("% paperx:sections\n{}", include_line)).to_string();
    } else {
        let enddoc = Regex::new(r"(?m)^\\end\{document\}").unwrap();
        main = enddoc.replace(&main, format!("{}\n\\end{{document}}", include_line)).to_string();
    }
    write(&main_path, &main)?;
    println!("✅ Added section: {}", path.display());
    Ok(())
}

fn cmd_add_figure(src: &str, label: Option<&str>, caption: Option<&str>) -> Result<()> {
    let src_path = Path::new(src);
    if !src_path.exists() { return Err(anyhow!("Figure not found: {}", src)); }
    fs::create_dir_all("figures")?;
    let fname = src_path.file_name().unwrap();
    let dst = Path::new("figures").join(fname);
    fs::copy(src_path, &dst).context("copy figure")?;
    let lab = label.unwrap_or("fig:example");
    let cap = caption.unwrap_or("Caption here.");
    println!("\nLaTeX snippet to include (copy into a section):\n");
    println!("\\begin{{figure}}[t]\\centering\\includegraphics[width=0.9\\linewidth]{{figures/{}}}\\caption{{{}}}\\label{{{}}}\\end{{figure}}\n",
        fname.to_string_lossy(), cap, lab);
    println!("✅ Copied to {}", dst.display());
    Ok(())
}

fn read_config() -> Result<Config> {
    let s = fs::read_to_string("paperx.toml").context("read paperx.toml")?;
    Ok(toml::from_str(&s).context("parse paperx.toml")?)
}

fn pick_engine(pref: EnginePref) -> Result<String> {
    use which::which;
    let ordered = match pref {
        EnginePref::Tectonic => ["tectonic", "latexmk", "pdflatex", "lualatex"],
        EnginePref::Latexmk  => ["latexmk", "tectonic", "pdflatex", "lualatex"],
        EnginePref::Pdflatex => ["pdflatex", "latexmk", "tectonic", "lualatex"],
        EnginePref::Lualatex => ["lualatex", "latexmk", "tectonic", "pdflatex"],
    };
    for e in ordered { if which(e).is_ok() { return Ok(e.to_string()); } }
    Err(anyhow!("No TeX engine found. Please install tectonic or TeX Live."))
}

fn guess_pdf_path(outdir: &str) -> Result<PathBuf> {
    let pdf = Path::new(outdir).join("main.pdf");
    if pdf.exists() { return Ok(pdf); }
    for entry in WalkDir::new(outdir) {
        let e = entry?;
        if e.path().extension().map(|x| x == "pdf").unwrap_or(false) {
            return Ok(e.path().to_path_buf());
        }
    }
    Err(anyhow!("Could not find resulting PDF in {}", outdir))
}

fn default_pdf_path() -> Result<PathBuf> { guess_pdf_path("build") }

fn write(path: &Path, s: &str) -> Result<()> {
    if let Some(p) = path.parent() { fs::create_dir_all(p)?; }
    let mut f = File::create(path)?; f.write_all(s.as_bytes())?; Ok(())
}

fn run(cmd: &mut Command) -> Result<()> {
    let status = cmd.status()?;
    if !status.success() { return Err(anyhow!("Command failed: {:?}", cmd)); }
    Ok(())
}

fn titleize(s: &str) -> String {
    let mut out = String::new();
    for (i, part) in s.split(|c: char| c == '-' || c == '_' || c == ' ').enumerate() {
        if i > 0 { out.push(' '); }
        let mut ch = part.chars();
        if let Some(f) = ch.next() {
            out.push_str(&f.to_uppercase().to_string());
            out.push_str(&ch.as_str());
        }
    }
    out
}

// ------------------- Templates & Defaults -------------------
const GITIGNORE: &str = r#"build/\n*.aux\n*.log\n*.out\n*.toc\n*.bbl\n*.blg\n*.synctex.gz\n"#;

const README_PROJECT: &str = r#"# Paper Project\n\nCommands:\n- Build once: `cargo run --release -- build --open`\n- Watch/rebuild: `cargo run --release -- watch`\n- Add section: `cargo run --release -- add section related-work`\n- Add figure: `cargo run --release -- add figure path/to/plot.png --label fig:plot --caption \"Plot caption\"`\n\nEdit `paperx.toml` to change engine or metadata.\n"#;

const BIB_TEMPLATE: &str = r#"@article{knuth1984,\n  author  = {Knuth, Donald E.},\n  title   = {Literate Programming},\n  journal = {The Computer Journal},\n  year    = {1984},\n}\n"#;

const SECTION_INTRO: &str = r#"% paperx: example section\n\\section{Introduction}\nThis is the introduction. Cite like~\\cite{knuth1984} and refer to Fig.~\\ref{fig:example}.\n"#;

const TEMPLATE_ARTICLE_EN: &str = r#"% !TEX TS-program = tectonic\n\\documentclass[11pt]{article}\n\\usepackage[a4paper,margin=1in]{geometry}\n\\usepackage{graphicx}\n\\usepackage{booktabs}\n\\usepackage{hyperref}\n\\usepackage{amsmath,amssymb}\n\\usepackage{siunitx}\n\\usepackage{authblk}\n\\usepackage[numbers]{natbib}\n\n\\title{${TITLE}}\n\\author[1]{${AUTHOR}}\n\\affil[1]{${AFFIL}}\n\n\\date{\\today}\n\n\\begin{document}\n\\maketitle\n\n\\begin{abstract}\n${ABSTRACT}\n\\end{abstract}\n\n\\textbf{Keywords:} ${KEYWORDS}\n\n% paperx:sections\n\n\\input{sections/introduction}\n\n\\bibliographystyle{plainnat}\n\\bibliography{../bib/references}\n\\end{document}\n"#;

const TEMPLATE_LTJS_JA: &str = r#"% !TEX TS-program = lualatex\n\\documentclass[11pt]{ltjsarticle}\n\\usepackage[a4paper,margin=25mm]{geometry}\n\\usepackage{graphicx}\n\\usepackage{booktabs}\n\\usepackage{luatexja-fontspec}\n\\usepackage{hyperref}\n\\usepackage{amsmath,amssymb}\n\\usepackage{siunitx}\n\\usepackage[numbers]{natbib}\n\\setmainjfont{Noto Serif CJK JP}\n\n\\title{${TITLE}}\n\\author{${AUTHOR}}\\\\\\textit{${AFFIL}}\n\\date{\\today}\n\n\\begin{document}\n\\maketitle\n\n\\begin{abstract}\n${ABSTRACT}\n\\end{abstract}\n\n\\textbf{キーワード:} ${KEYWORDS}\n\n% paperx:sections\n\n\\input{sections/introduction}\n\n\\bibliographystyle{plainnat}\n\\bibliography{../bib/references}\n\\end{document}\n"#;