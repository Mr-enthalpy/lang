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

#[test]
fn complete_type_results_cross_boundaries_explicitly() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let primitive =
        fs::read_to_string(src.join("meta_invocation.rs")).expect("read primitive executor source");
    assert!(
        !primitive.contains("InvocationResult::semantic("),
        "primitive execution material is installed before a declared semantic result is formed"
    );

    let world = fs::read_to_string(src.join("world.rs")).expect("read world source");
    let start = world
        .find("fn install_connected_semantic_binding(")
        .expect("connected binding installer exists");
    let tail = &world[start..];
    let end = tail
        .find("fn install_connected_generated_type_binding(")
        .expect("next binding helper exists");
    let installer = &tail[..end];
    assert!(
        installer.contains("semantic_complete_type"),
        "the binding installer receives the observed complete type explicitly"
    );
}
