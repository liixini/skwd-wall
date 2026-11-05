use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const GPU_MARKS: [&str; 3] = ["headless::", "Instance::new(", "Engine::new("];
const HEADLESS: &str = "headless.rs";
const RICH_CALL: &str = "rich_text(";

struct Func {
    file: String,
    name: String,
    body: String,
    test: bool,
    ignored: bool,
}

fn rust_files(directory: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).expect("read dir") {
        let path = entry.expect("entry").path();
        if path.is_dir() {
            rust_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

fn declared(line: &str) -> Option<&str> {
    let after = line.trim_start().split_once("fn ")?;
    let head = after.0;
    if !head.is_empty()
        && !head
            .split_whitespace()
            .all(|word| word.starts_with("pub") || matches!(word, "const" | "async" | "unsafe"))
    {
        return None;
    }
    after.1.split(['(', '<']).next().map(str::trim)
}

fn functions(file: &str, source: &str) -> Vec<Func> {
    let lines: Vec<&str> = source.lines().collect();
    let mut found: Vec<Func> = Vec::new();
    let mut attrs = String::new();
    for line in &lines {
        let trimmed = line.trim_start();
        if trimmed.starts_with("#[") {
            attrs.push_str(trimmed);
            continue;
        }
        if let Some(name) = declared(line) {
            found.push(Func {
                file: file.to_string(),
                name: name.to_string(),
                body: String::new(),
                test: attrs.contains("#[test]"),
                ignored: attrs.contains("#[ignore"),
            });
            attrs.clear();
        }
        if !trimmed.is_empty() {
            attrs.clear();
        }
        if let Some(last) = found.last_mut() {
            last.body.push_str(line);
            last.body.push('\n');
        }
    }
    found
}

fn near(left: &str, right: &str) -> bool {
    parent(left) == parent(right)
}

fn parent(file: &str) -> &str {
    let dir = file.rsplit_once('/').map_or("", |(dir, _)| dir);
    dir.strip_suffix("/tests").unwrap_or(dir)
}

fn calls(body: &str, name: &str) -> bool {
    let call = format!("{name}(");
    body.match_indices(&call).any(|(at, _)| {
        body[..at]
            .chars()
            .next_back()
            .is_none_or(|prev| !(prev.is_alphanumeric() || prev == '_' || prev == '.'))
    })
}

#[test]
fn gpu_tests_stay_ignored() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&root, &mut files);
    let mut all = Vec::new();
    for path in &files {
        let source = fs::read_to_string(path).expect("read source");
        let name = path.strip_prefix(&root).unwrap_or(path).display().to_string();
        all.extend(functions(&name, &source));
    }
    let mut reaching: BTreeSet<(String, String)> = all
        .iter()
        .filter(|func| !func.file.ends_with(HEADLESS))
        .filter(|func| GPU_MARKS.iter().any(|mark| func.body.contains(mark)))
        .map(|func| (func.file.clone(), func.name.clone()))
        .collect();
    loop {
        let before = reaching.len();
        for func in all.iter().filter(|func| !func.file.ends_with(HEADLESS)) {
            let reached = reaching
                .iter()
                .any(|(file, name)| near(file, &func.file) && calls(&func.body, name));
            if reached {
                reaching.insert((func.file.clone(), func.name.clone()));
            }
        }
        if reaching.len() == before {
            break;
        }
    }
    let loose: Vec<_> = all
        .iter()
        .filter(|func| {
            func.test && !func.ignored && reaching.contains(&(func.file.clone(), func.name.clone()))
        })
        .map(|func| format!("{}::{}", func.file, func.name))
        .collect();
    assert!(loose.is_empty(), "{loose:?}");
}

#[test]
fn no_rich_text() {
    let mut files = Vec::new();
    rust_files(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut files);
    let strays: Vec<_> = files
        .iter()
        .filter(|path| fs::read_to_string(path).expect("source").contains(RICH_CALL))
        .collect();
    assert!(strays.is_empty(), "{strays:?}");
}
