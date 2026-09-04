//! Positive source-fixture and semantic-boundary invariants.

use std::fs;
use std::path::Path;

#[test]
fn ordinary_tests_use_committed_source_fixtures() {
    const WRITE_BYTES_ALLOWED: &[&str] = &["source_discovery_boundary.rs"];

    let tests_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let this_file = "canonical_source_invariants.rs";
    let mut offenders = Vec::new();

    for entry in fs::read_dir(&tests_dir).expect("read tests dir") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let name = path
            .file_name()
            .expect("file name")
            .to_string_lossy()
            .to_string();
        if name == this_file {
            continue;
        }
        let content = fs::read_to_string(&path).expect("read test file");
        if content.contains(".write_boundary_source(")
            || content.contains(".write_bytes(") && !WRITE_BYTES_ALLOWED.contains(&name.as_str())
        {
            offenders.push(name);
        }
    }

    assert!(
        offenders.is_empty(),
        "ordinary build tests use committed fixtures; only invalid-byte boundary tests may write raw bytes: {offenders:?}"
    );
}
