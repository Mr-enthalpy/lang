use std::{fs, path::Path};

fn visit_files(root: &Path, path: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(path).expect("read active tree") {
        let entry = entry.expect("read active-tree entry");
        let path = entry.path();
        let relative = path.strip_prefix(root).expect("path under repository root");
        let normalized = relative.to_string_lossy().replace('\\', "/");
        if entry.file_type().expect("read file type").is_dir() {
            if matches!(normalized.as_str(), ".git" | "target" | "spec/history")
                || normalized.starts_with(".git/")
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
            violations.push(format!("version-lineage path: {relative}"));
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
        for phrase in &phrases {
            if lower.contains(phrase) {
                violations.push(format!("{relative}: contains `{phrase}`"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "active artifacts must describe only the current semantic architecture:\n{}",
        violations.join("\n")
    );
}
