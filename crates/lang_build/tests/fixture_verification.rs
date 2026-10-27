mod support;
use support::*;

use lang_build::{BuildSession, BuildWorkspace, ToolchainGlobalSourceRoot};

const PASS_SINGLE_PACKAGE_FIXTURES: &[(&str, &str)] = &[
    ("verify_compile_conflict", "app"),
    ("resolver_core_conflict", "app"),
    ("single_package_type_binding", "app"),
    ("non_lang_files_ignored", "app"),
    ("empty_navigation_beside_binding", "app"),
    ("empty_inherited_navigation", "app"),
];

const PASS_WORKSPACE_FIXTURES: &[(&str, fn() -> BuildWorkspace)] = &[
    (
        "dependency_mount_no_import",
        dependency_mount_no_import_fixture,
    ),
    (
        "dependency_mount_no_import_dep_changed",
        dependency_mount_no_import_dep_changed_fixture,
    ),
];

fn runtime_transport_fixture(workspace: &str) -> BuildWorkspace {
    let mut app = fixture_package_spec(workspace, "app");
    app.global_implementation_roots
        .push(ToolchainGlobalSourceRoot::under(
            fixture_root()
                .join("global_implementation")
                .join("uint8_transport"),
            vec!["core".to_string(), "uint8".to_string()],
        ));
    BuildWorkspace {
        packages: vec![app],
    }
}

#[test]
fn source_transport_requires_ordinary_object_path_wiring() {
    for workspace in [
        "verify_runtime_shadow",
        "policy_aware_static_program",
        "user_runtime_values",
    ] {
        let mut session = BuildSession::new();
        let error = session
            .build_workspace(&runtime_transport_fixture(workspace))
            .expect_err("unconnected single-stage transport cannot produce a migrated value");
        assert!(
            format!("{error:?}").contains("ordinary Path consumer"),
            "{workspace}: {error:?}"
        );
    }
}

// These fixtures exercise connected declaration diagnostics and explicit
// unavailable completion frontiers.
const FAIL_SINGLE_PACKAGE_FIXTURES: &[(&str, &str, &str)] = &[
    ("external_classifier_mismatch", "app", "classifier"),
    (
        "inherited_navigation",
        "app",
        "inherited navigation requires the qualified NameExpr",
    ),
    (
        "child_navigation_frontier",
        "app",
        "inherited navigation requires the qualified NameExpr",
    ),
    (
        "same_name_distinct_namespaces",
        "app",
        "inherited navigation requires the qualified NameExpr",
    ),
    (
        "multi_file_same_namespace",
        "app",
        "inherited navigation requires the qualified NameExpr",
    ),
    (
        "source_expression_frontier",
        "app",
        "source expression completion requires common E",
    ),
    (
        "no_import_syntax",
        "app",
        "source expression completion requires common E",
    ),
    (
        "structural_single_child_frontier",
        "app",
        "structural type formation consumer is unavailable",
    ),
    (
        "structural_nested_layer_frontier",
        "app",
        "structural type formation consumer is unavailable",
    ),
    (
        "structural_empty_layer_frontier",
        "app",
        "structural type formation consumer is unavailable",
    ),
    (
        "structural_named_slots_frontier",
        "app",
        "structural type formation consumer is unavailable",
    ),
    (
        "structural_duplicate_navigation_frontier",
        "app",
        "structural type formation consumer is unavailable",
    ),
    (
        "struct_helper_frontier",
        "app",
        "struct helper formation consumer is unavailable",
    ),
    (
        "struct_helper_explicit_core_frontier",
        "app",
        "struct helper formation consumer is unavailable",
    ),
    (
        "struct_helper_argument_arity",
        "app",
        "candidate preparation arity mismatch",
    ),
    (
        "struct_helper_unobserved_argument",
        "app",
        "parameter shape compatibility is not established",
    ),
    (
        "qualified_name_destination_frontier",
        "app",
        "qualified NameExpr formation and destination Place consumer is unavailable",
    ),
    (
        "nested_qualified_name_destination_frontier",
        "app",
        "qualified NameExpr formation and destination Place consumer is unavailable",
    ),
    (
        "product_binder_rejected",
        "app",
        "unsupported top-level declaration binder",
    ),
    (
        "discard_binder_rejected",
        "app",
        "qualified NameExpr formation and destination Place consumer is unavailable",
    ),
    (
        "path_alias_destination_frontier",
        "app",
        "lexical Path alias formation/composition consumer is unavailable",
    ),
    (
        "diagnostic_source_contribution_prefix",
        "app",
        "source contribution error:",
    ),
    (
        "diagnostic_descendant",
        "app",
        "qualified NameExpr formation",
    ),
    ("duplicate_declaration", "app", "conflict"),
    (
        "noncallable_target",
        "app",
        "ordinary invocation found no fully admissible candidate",
    ),
    (
        "identity_type_missing_rank",
        "app",
        "parameter shape compatibility is not established",
    ),
];

#[test]
fn declaration_fixtures_build_without_source_evaluation() {
    // Collect every failing fixture instead of stopping at the first one so
    // a single run reports the complete pass-fixture status.
    let mut failures = Vec::new();
    for (workspace, package) in PASS_SINGLE_PACKAGE_FIXTURES {
        let mut session = BuildSession::new();
        if let Err(error) = session.build_workspace(&single_package_fixture(workspace, package)) {
            failures.push(format!("fixture `{workspace}` failed: {error:#?}"));
        }
    }

    for (name, workspace) in PASS_WORKSPACE_FIXTURES {
        let mut session = BuildSession::new();
        if let Err(error) = session.build_workspace(&workspace()) {
            failures.push(format!("fixture `{name}` failed: {error:#?}"));
        }
    }
    assert!(
        failures.is_empty(),
        "{} pass fixture(s) failed:\n\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}

#[test]
fn fail_fixtures_report_expected_diagnostics() {
    for (workspace, package, expected) in FAIL_SINGLE_PACKAGE_FIXTURES {
        let error = build_fixture_error(workspace, package);
        assert!(
            error
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "fixture `{workspace}` missing expected diagnostic {expected:?}: {:#?}",
            error.diagnostics
        );
    }
}
