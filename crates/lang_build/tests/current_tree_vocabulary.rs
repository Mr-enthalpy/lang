use std::{fs, path::Path};

fn visit_files(root: &Path, path: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(path).expect("read active tree") {
        let entry = entry.expect("read active-tree entry");
        let path = entry.path();
        let relative = path.strip_prefix(root).expect("path under repository root");
        let normalized = relative.to_string_lossy().replace('\\', "/");
        if entry.file_type().expect("read file type").is_dir() {
            if matches!(
                normalized.as_str(),
                ".git" | ".opencode" | "node_modules" | "target" | "spec/history"
            ) || normalized.starts_with(".git/")
                || normalized.starts_with(".opencode/")
                || normalized.ends_with("/node_modules")
                || normalized.contains("/node_modules/")
                || normalized.starts_with("target/")
                || normalized.starts_with("spec/history/")
            {
                continue;
            }
            visit_files(root, &path, out);
        } else {
            out.push(path);
        }
    }
}

fn contains_numbered_stage_marker(text: &str) -> bool {
    let chars = text.char_indices().collect::<Vec<_>>();
    for (index, &(byte_index, ch)) in chars.iter().enumerate() {
        if ch != 'S' && ch != 's' {
            continue;
        }
        if index > 0 {
            let previous = chars[index - 1].1;
            if previous.is_alphanumeric() || previous == '_' {
                continue;
            }
        }
        let suffix = &text[byte_index + ch.len_utf8()..];
        let digit_count = suffix.chars().take_while(|c| c.is_ascii_digit()).count();
        if digit_count == 0 {
            continue;
        }
        let remaining = suffix.chars().skip(digit_count).next();
        if remaining.map_or(true, |next| !next.is_alphanumeric() && next != '_') {
            return true;
        }
    }
    false
}

#[test]
fn active_tree_uses_current_semantic_vocabulary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let phrases = fs::read_to_string(root.join("spec/history/active-tree-archaeology-phrases.txt"))
        .expect("read active-tree vocabulary boundary")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let path_markers = fs::read_to_string(root.join("spec/history/active-tree-path-markers.txt"))
        .expect("read active-tree path boundary")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let retired_symbols =
        fs::read_to_string(root.join("spec/history/active-tree-retired-symbols.txt"))
            .expect("read semantic-symbol boundary")
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(str::to_owned)
            .collect::<Vec<_>>();

    let mut files = Vec::new();
    visit_files(root, root, &mut files);
    let mut violations = Vec::new();
    for path in files {
        let relative = path
            .strip_prefix(root)
            .expect("path under repository root")
            .to_string_lossy()
            .replace('\\', "/");
        let lower_path = relative.to_ascii_lowercase();
        if path_markers
            .iter()
            .any(|marker| lower_path.contains(marker))
        {
            violations.push(format!("non-current path marker: {relative}"));
        }

        let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
            continue;
        };
        if ![
            "rs", "md", "toml", "yml", "yaml", "lang", "ast", "norm", "tokens", "diag",
        ]
        .contains(&extension)
        {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let lower = content.to_ascii_lowercase();
        if contains_numbered_stage_marker(&content) {
            violations.push(format!(
                "{relative}: contains a numbered implementation stage"
            ));
        }
        for phrase in &phrases {
            if lower.contains(phrase) {
                violations.push(format!("{relative}: contains `{phrase}`"));
            }
        }
        for symbol in &retired_symbols {
            if lower.contains(symbol) {
                violations.push(format!(
                    "{relative}: contains non-current symbol `{symbol}`"
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "active artifacts must describe only the current semantic architecture:\n{}",
        violations.join("\n")
    );
}
