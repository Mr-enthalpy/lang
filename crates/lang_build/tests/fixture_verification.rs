mod support;
use support::*;

use lang_build::{BuildSession, BuildWorkspace, ToolchainGlobalSourceRoot};

const PASS_SINGLE_PACKAGE_FIXTURES: &[(&str, &str)] = &[
    ("vertical_slice", "app"),
    ("core_verify_namespace", "app"),
    ("resolver_core_paths", "app"),
    ("verify_meta_conflict", "app"),
    ("early_struct_meta", "app"),
    ("struct_single_field", "app"),
    ("struct_invalid_field_syntax", "app"),
    ("field_named_ref", "app"),
    ("field_named_share", "app"),
    ("physical_subns", "app"),
    ("type_named_struct", "app"),
    ("v08_unary_construction", "app"),
    ("v08_struct_uint8", "app"),
    ("v08_struct_uint16", "app"),
    ("same_name_distinct_namespaces", "app"),
    ("resolver_core_conflict", "app"),
    ("single_package_type_binding", "app"),
    ("nested_physical_namespace", "app"),
    ("multi_file_same_namespace", "app"),
    ("no_import_syntax", "app"),
    ("non_lang_files_ignored", "app"),
];

const PASS_WORKSPACE_FIXTURES: &[(&str, fn() -> BuildWorkspace)] = &[
    (
        "verify_runtime_shadow",
        verify_runtime_shadow_with_migration_fixture,
    ),
    (
        "policy_aware_early_meta",
        policy_aware_early_meta_with_migration_fixture,
    ),
    (
        "user_runtime_values",
        user_runtime_values_with_migration_fixture,
    ),
    (
        "dependency_mount_no_import",
        dependency_mount_no_import_fixture,
    ),
    (
        "dependency_mount_no_import_dep_changed",
        dependency_mount_no_import_dep_changed_fixture,
    ),
];

fn runtime_literal_verification_fixture(workspace: &str) -> BuildWorkspace {
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

fn verify_runtime_shadow_with_migration_fixture() -> BuildWorkspace {
    runtime_literal_verification_fixture("verify_runtime_shadow")
}

fn policy_aware_early_meta_with_migration_fixture() -> BuildWorkspace {
    runtime_literal_verification_fixture("policy_aware_early_meta")
}

fn user_runtime_values_with_migration_fixture() -> BuildWorkspace {
    runtime_literal_verification_fixture("user_runtime_values")
}

// Temporary runner metadata, not semantic verification: these fixtures fail
// before source verification can run, so the runner checks only the expected
// diagnostic prefix/category.
const FAIL_SINGLE_PACKAGE_FIXTURES: &[(&str, &str, &str)] = &[
    (
        "source_verification_failure",
        "app",
        "source verification error:",
    ),
    (
        "verify_unknown_operation",
        "app",
        "unknown verification operation",
    ),
    ("verify_malformed_arity", "app", "expects 2 argument(s)"),
    ("struct_duplicate_field", "app", "duplicate field name"),
    ("struct_non_type_field", "app", "unknown struct field type"),
    ("struct_nested_product", "app", "invalid struct syntax"),
    ("struct_unit_field", "app", "unit field or trailing unit"),
    (
        "struct_target_not_name",
        "app",
        "expected a field binder name",
    ),
    (
        "struct_operator_private_syntax",
        "app",
        "invalid struct syntax",
    ),
    (
        "struct_unknown_field_type",
        "app",
        "unknown struct field type",
    ),
    (
        "runtime_value_as_struct_field_type",
        "app",
        "unknown struct field type",
    ),
    ("source_conflict_physical_dir_symbol", "app", "conflict"),
    (
        "descendant_injection",
        "app",
        "ordinary parent-to-descendant injection",
    ),
    (
        "deep_descendant_injection",
        "app",
        "ordinary parent-to-descendant injection",
    ),
    (
        "product_binder_rejected",
        "app",
        "unsupported top-level declaration binder",
    ),
    (
        "discard_binder_rejected",
        "app",
        "ordinary parent-to-descendant injection",
    ),
    (
        "alias_external_injection_future",
        "app",
        "declaration alias semantics are retired",
    ),
    (
        "diagnostic_source_contribution_prefix",
        "app",
        "source contribution error:",
    ),
    ("diagnostic_conflict", "app", "conflict"),
    ("diagnostic_descendant", "app", "parent-to-descendant"),
    ("duplicate_declaration", "app", "conflict"),
    ("non_meta_target", "app", "UnsupportedDeferredTypeAssertion"),
    (
        "ambient_struct_collision",
        "app",
        "ambient struct collision",
    ),
    (
        "v08_identity_type_notype",
        "app",
        "could not be resolved as a type object",
    ),
];

#[test]
fn pass_fixtures_run_source_verification_loop() {
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
