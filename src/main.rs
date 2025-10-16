use colored::Colorize;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Default, Clone)]
struct Config {
    case_insensitive: bool, // -i
    show_line_numbers: bool, // -n
    invert_match: bool,     // -v
    recursive: bool,        // -r
    show_filenames: bool,   // -f
    colored: bool,          // -c
    help: bool,             // -h/--help
    pattern: String,
    targets: Vec<String>,
}

const HELP: &str = r#"Usage: grep [OPTIONS] <pattern> <files...>

Options:
-i                Case-insensitive search
-n                Print line numbers
-v                Invert match (exclude lines that match the pattern)
-r                Recursive directory search
-f                Print filenames
-c                Enable colored output
-h, --help        Show help information
"#;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let cfg = match parse_args(args) {
        Ok(c) => c,
        Err(_) => {
            print!("{}", HELP);
            panic!("Invalid arguments");
        }
    };

    if cfg.help || cfg.pattern.is_empty() || cfg.targets.is_empty() {
        print!("{}", HELP);
        return;
    }

    // Gather files
    let mut files: Vec<PathBuf> = Vec::new();
    for t in &cfg.targets {
        let p = PathBuf::from(t);
        if cfg.recursive {
            if p.is_dir() {
                for entry in WalkDir::new(&p)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|e| e.file_type().is_file())
                {
                    files.push(entry.path().to_path_buf());
                }
            } else {
                files.push(p);
            }
        } else {
            files.push(p);
        }
    }

    for f in files {
        if let Err(e) = search_file(&f, &cfg) {
            eprintln!("Failed to read {}: {}", f.display(), e);
        }
    }
}

fn parse_args(args: Vec<String>) -> Result<Config, ()> {
    let mut cfg = Config::default();
    let mut operands: Vec<String> = Vec::new();

    for a in args {
        if a == "-h" || a == "--help" {
            cfg.help = true;
            continue;
        }
        if a.starts_with('-') && a.len() >= 2 {
            for ch in a.chars().skip(1) {
                match ch {
                    'i' => cfg.case_insensitive = true,
                    'n' => cfg.show_line_numbers = true,
                    'v' => cfg.invert_match = true,
                    'r' => cfg.recursive = true,
                    'f' => cfg.show_filenames = true,
                    'c' => cfg.colored = true,
                    'h' => cfg.help = true,
                    '-' => { /* allow --help handled above */ }
                }
            }
        } else {
            operands.push(a);
        }
    }

    if !operands.is_empty() {
        cfg.pattern = operands[0].clone();
        cfg.targets = operands[1..].to_vec();
    }

    Ok(cfg)
}

fn search_file(path: &Path, cfg: &Config) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let pattern = if cfg.case_insensitive {
        cfg.pattern.to_lowercase()
    } else {
        cfg.pattern.clone()
    };

    for (idx, line_res) in reader.lines().enumerate() {
        let line = line_res?;
        let is_match = contains(&line, &pattern, cfg.case_insensitive);

        let pass = if cfg.invert_match { !is_match } else { is_match };
        if !pass {
            continue;
        }

        let mut out = String::new();

        if cfg.show_filenames {
            out.push_str(&format!("{}: ", path.display()));
        }

        if cfg.show_line_numbers {
            out.push_str(&format!("{}: ", idx + 1));
        }

        if cfg.colored && is_match && !cfg.invert_match {
            out.push_str(&highlight(&line, &pattern, cfg.case_insensitive));
        } else {
            out.push_str(&line);
        }

        println!("{}", out);
    }

    Ok(())
}

fn contains(line: &str, pattern: &str, case_insensitive: bool) -> bool {
    if case_insensitive {
        line.to_lowercase().contains(pattern)
    } else {
        line.contains(pattern)
    }
}

fn highlight(line: &str, pattern: &str, case_insensitive: bool) -> String {
    if pattern.is_empty() {
        return line.to_string();
    }

    let (haystack, needle) = if case_insensitive {
        (line.to_lowercase(), pattern.to_lowercase())
    } else {
        (line.to_string(), pattern.to_string())
    };

    let mut result = String::with_capacity(line.len());
    let mut i = 0;
    while let Some(pos) = haystack[i..].find(&needle) {
        let start = i + pos;
        let end = start + needle.len();
        result.push_str(&line[i..start]);
        result.push_str(&line[start..end].red().to_string());
        i = end;
    }
    
    result.push_str(&line[i..]);
    result
}
