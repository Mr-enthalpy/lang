mod support;
use support::*;

use lang_build::{BuildSession, BuildWorkspace};

const PASS_SINGLE_PACKAGE_FIXTURES: &[(&str, &str)] = &[
    ("vertical_slice", "app"),
    ("core_verify_namespace", "app"),
    ("resolver_core_paths", "app"),
    ("verify_runtime_shadow", "app"),
    ("early_struct_meta", "app"),
    ("struct_single_field", "app"),
    ("field_named_ref", "app"),
    ("field_named_share", "app"),
    ("policy_aware_early_meta", "app"),
    ("user_runtime_values", "app"),
    ("physical_subns", "app"),
    ("type_named_struct", "app"),
    ("non_meta_target", "app"),
    ("same_name_distinct_namespaces", "app"),
    ("resolver_core_conflict", "app"),
    ("single_package_type_binding", "app"),
    ("nested_physical_namespace", "app"),
    ("multi_file_same_namespace", "app"),
    ("no_import_syntax", "app"),
    ("non_lang_files_ignored", "app"),
];

const PASS_WORKSPACE_FIXTURES: &[(&str, fn() -> BuildWorkspace)] = &[(
    "dependency_mount_no_import",
    dependency_mount_no_import_fixture,
)];

// Temporary runner metadata, not semantic verification: these fixtures fail
// before source verification can run, so the runner checks only the expected
// diagnostic prefix/category.
const FAIL_SINGLE_PACKAGE_FIXTURES: &[(&str, &str, &str)] = &[
    (
        "source_verification_failure",
        "app",
        "source verification error:",
    ),
    ("verify_meta_conflict", "app", "source verification error:"),
    (
        "verify_unknown_operation",
        "app",
        "unknown verification operation",
    ),
    ("verify_malformed_arity", "app", "expects 2 argument(s)"),
    ("struct_duplicate_field", "app", "duplicate struct field"),
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
        "struct_invalid_field_syntax",
        "app",
        "invalid struct syntax",
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
        "ordinary parent-to-descendant injection",
    ),
    (
        "diagnostic_source_contribution_prefix",
        "app",
        "source contribution error:",
    ),
    ("diagnostic_conflict", "app", "conflict"),
    ("diagnostic_descendant", "app", "parent-to-descendant"),
    ("duplicate_declaration", "app", "conflict"),
];

#[test]
fn pass_fixtures_run_source_verification_loop() {
    for (workspace, package) in PASS_SINGLE_PACKAGE_FIXTURES {
        build_single_fixture_world(workspace, package);
    }

    for (name, workspace) in PASS_WORKSPACE_FIXTURES {
        let mut session = BuildSession::new();
        session
            .build_workspace(&workspace())
            .unwrap_or_else(|error| panic!("fixture `{name}` failed: {error:#?}"));
    }
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
