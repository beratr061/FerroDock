use std::fs;
use std::path::{Path, PathBuf};

const MAX_LINES: usize = 250;
const SKIPPED_DIRS: [&str; 8] = [
    "target",
    "node_modules",
    "dist",
    ".git",
    ".sqlx",
    ".claude",
    "data",
    ".idea",
];
const GENERATED_FILES: [&str; 2] = ["schema.d.ts", "pnpm-lock.yaml"];

#[derive(Clone, Copy)]
enum Syntax {
    Slash,
    Hash,
    Sql,
    Markup,
}

fn syntax_of(path: &Path) -> Option<Syntax> {
    let name = path.file_name()?.to_str()?;
    if name == "Dockerfile" || name == ".gitignore" || name == ".dockerignore" {
        return Some(Syntax::Hash);
    }
    match path.extension()?.to_str()? {
        "rs" | "ts" | "tsx" | "js" | "mjs" | "css" => Some(Syntax::Slash),
        "toml" | "yml" | "yaml" | "sh" | "py" => Some(Syntax::Hash),
        "sql" => Some(Syntax::Sql),
        "html" => Some(Syntax::Markup),
        _ => None,
    }
}

fn is_directive(syntax: Syntax, index: usize, line: &str) -> bool {
    matches!(syntax, Syntax::Hash)
        && index == 0
        && (line.starts_with("#!") || line.starts_with("# syntax="))
}

fn is_comment(syntax: Syntax, line: &str) -> bool {
    let trimmed = line.trim_start();
    match syntax {
        Syntax::Slash => {
            trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("* ")
        }
        Syntax::Hash => trimmed.starts_with('#'),
        Syntax::Sql => trimmed.starts_with("--"),
        Syntax::Markup => trimmed.contains("<!--"),
    }
}

fn collect(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_symlink() {
            continue;
        }
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if kind.is_dir() {
            if !SKIPPED_DIRS.contains(&name.as_ref()) {
                collect(&path, files);
            }
        } else if !GENERATED_FILES.contains(&name.as_ref()) {
            files.push(path);
        }
    }
}

fn violations_in(root: &Path, path: &Path) -> Vec<String> {
    let Some(syntax) = syntax_of(path) else {
        return Vec::new();
    };
    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let shown = path
        .strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string();
    let lines: Vec<&str> = content.lines().collect();
    let mut found = Vec::new();
    if lines.len() > MAX_LINES {
        found.push(format!(
            "{shown}: {} lines (limit {MAX_LINES})",
            lines.len()
        ));
    }
    for (index, line) in lines.iter().enumerate() {
        if is_comment(syntax, line) && !is_directive(syntax, index, line) {
            found.push(format!("{shown}:{}: comment line", index + 1));
        }
    }
    found
}

pub fn check(root: &Path) -> Result<(), String> {
    let mut files = Vec::new();
    collect(root, &mut files);
    let violations: Vec<String> = files
        .iter()
        .flat_map(|path| violations_in(root, path))
        .collect();
    if violations.is_empty() {
        println!("code rules: {} files checked, no violations", files.len());
        Ok(())
    } else {
        Err(format!("code rule violations:\n{}", violations.join("\n")))
    }
}
