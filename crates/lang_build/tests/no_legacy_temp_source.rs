//! Anti-regression guard.
//!
//! Ordinary lang_build build/discovery/early-meta tests must use committed
//! fixtures under `tests/fixtures/workspaces/`, not temp-constructed source
//! trees. `TempProject::write_boundary_source` is forbidden everywhere; invalid
//! byte boundary tests may still use `write_bytes`.
//!
//! This is a deliberately simple static string scan, not a linter. If a new
//! ordinary test reconstructs source files in temp directories, this test fails.

use std::fs;
use std::path::Path;

#[test]
fn ordinary_tests_do_not_construct_temp_source() {
    // Files allowed to write raw bytes, with the reason each is a boundary case.
    const WRITE_BYTES_ALLOWED: &[&str] = &[
        // Invalid UTF-8 cannot be represented as a committed source fixture.
        "source_discovery_boundary.rs",
    ];

    let tests_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    // This scanner file contains the search patterns as string literals, so it
    // must exclude itself.
    let this_file = "no_legacy_temp_source.rs";

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
        if content.contains(".write_boundary_source(") {
            offenders.push(name.clone());
            continue;
        }
        if content.contains(".write_bytes(") && !WRITE_BYTES_ALLOWED.contains(&name.as_str()) {
            offenders.push(name);
        }
    }

    assert!(
        offenders.is_empty(),
        "ordinary build tests must use committed fixtures under \
         tests/fixtures/workspaces/, not temp-constructed source trees; \
         `.write_boundary_source(` is forbidden in tests and `.write_bytes(` \
         is restricted to invalid-byte boundaries; \
         offending files: {offenders:?}"
    );
}
