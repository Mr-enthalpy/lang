mod support;
use support::*;

use std::path::PathBuf;

use lang_build::{
    BuildResult, BuildSession, BuildWorkspace, CacheStatus, NamespaceMount, PackageBuildArtifact,
    PackageBuildSpec, ResolveExpectation, SourceRoot, StaticDependencySpec, SymbolKind,
};

fn package_spec(name: &str, src_root: PathBuf) -> PackageBuildSpec {
    let mut spec = PackageBuildSpec::new(name, vec![name.to_string()]);
    spec.source_roots.push(SourceRoot {
        path: src_root,
        namespace_root: vec![name.to_string()],
    });
    spec
}

fn dependency(package: &str, mount_path: &[&str]) -> StaticDependencySpec {
    StaticDependencySpec {
        package: package.to_string(),
        mount_path: mount_path
            .iter()
            .map(|segment| segment.to_string())
            .collect(),
    }
}

fn artifact_named<'a>(result: &'a BuildResult, name: &str) -> &'a PackageBuildArtifact {
    result
        .artifacts
        .iter()
        .find(|artifact| artifact.package_name == name)
        .unwrap_or_else(|| panic!("missing artifact for `{name}`"))
}

// A. Build a single package through BuildSession.
#[test]
fn builds_single_package_through_build_session() {
    let project = TempProject::new("static_single");
    project.write("app/src/main.lang", "let T: type = uint8");
    let workspace = BuildWorkspace {
        packages: vec![package_spec("app", project.path().join("app/src"))],
    };

    let mut session = BuildSession::new();
    let result = session
        .build_workspace(&workspace)
        .expect("build workspace");

    assert_eq!(result.artifacts.len(), 1);
    let artifact = &result.artifacts[0];
    assert_eq!(artifact.package_name, "app");
    assert!(artifact.world.resolve("T").is_ok());
    assert_eq!(artifact.metadata.source_units.len(), 1);
    assert_eq!(artifact.metadata.cache_status, CacheStatus::Miss);
}

// B. Cache hit on a repeated build with the same session and workspace.
#[test]
fn cache_hit_on_repeated_build() {
    let project = TempProject::new("static_cache_hit");
    project.write("app/src/main.lang", "let T: type = uint8");
    let workspace = BuildWorkspace {
        packages: vec![package_spec("app", project.path().join("app/src"))],
    };

    let mut session = BuildSession::new();
    let first = session.build_workspace(&workspace).expect("first build");
    assert_eq!(first.artifacts[0].metadata.cache_status, CacheStatus::Miss);
    let first_fingerprint = first.artifacts[0].fingerprint.clone();

    let second = session.build_workspace(&workspace).expect("second build");
    assert_eq!(second.artifacts[0].metadata.cache_status, CacheStatus::Hit);
    assert_eq!(second.artifacts[0].fingerprint, first_fingerprint);

    let stats = session.cache_stats();
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.hits, 1);
}

// C. Cache miss when source content changes.
#[test]
fn cache_miss_when_source_content_changes() {
    let project = TempProject::new("static_cache_miss");
    project.write("app/src/main.lang", "let T: type = uint8");
    let workspace = BuildWorkspace {
        packages: vec![package_spec("app", project.path().join("app/src"))],
    };

    let mut session = BuildSession::new();
    let first = session.build_workspace(&workspace).expect("first build");
    let first_fingerprint = first.artifacts[0].fingerprint.clone();
    let first_hash = first.artifacts[0].metadata.source_units[0]
        .content_hash
        .clone();

    project.write("app/src/main.lang", "let U: type = uint8");
    let second = session.build_workspace(&workspace).expect("second build");

    assert_eq!(second.artifacts[0].metadata.cache_status, CacheStatus::Miss);
    assert_ne!(
        second.artifacts[0].metadata.source_units[0].content_hash,
        first_hash
    );
    assert_ne!(second.artifacts[0].fingerprint, first_fingerprint);
}

// D. Static dependency order is deterministic, with lexical package-name
// tie-break among independent roots.
#[test]
fn static_dependency_order_is_deterministic() {
    let project = TempProject::new("static_order");
    project.write("aaa/src/lib.lang", "let X: type = uint8");
    project.write("corelib/src/lib.lang", "let C: type = uint8");
    project.write("math/src/lib.lang", "let M: type = uint8");
    project.write("app/src/main.lang", "let A: type = uint8");

    let aaa = package_spec("aaa", project.path().join("aaa/src"));
    let corelib = package_spec("corelib", project.path().join("corelib/src"));
    let mut math = package_spec("math", project.path().join("math/src"));
    math.dependencies.push(dependency("corelib", &["corelib"]));
    let mut app = package_spec("app", project.path().join("app/src"));
    app.dependencies.push(dependency("math", &["math"]));
    app.dependencies.push(dependency("corelib", &["corelib"]));

    // Provide packages out of topological order to prove ordering is computed.
    let workspace = BuildWorkspace {
        packages: vec![app, math, corelib, aaa],
    };
    let mut session = BuildSession::new();
    let result = session
        .build_workspace(&workspace)
        .expect("build workspace");

    let order: Vec<&str> = result
        .artifacts
        .iter()
        .map(|artifact| artifact.package_name.as_str())
        .collect();
    // `aaa` and `corelib` are both initially buildable; lexical tie-break puts
    // `aaa` first. The corelib -> math -> app chain follows.
    assert_eq!(order, vec!["aaa", "corelib", "math", "app"]);
}

// E. Unknown dependency is a hard error.
#[test]
fn unknown_dependency_is_a_hard_error() {
    let project = TempProject::new("static_unknown_dep");
    project.write("app/src/main.lang", "let T: type = uint8");
    let mut app = package_spec("app", project.path().join("app/src"));
    app.dependencies.push(dependency("dep", &["dep"]));
    let workspace = BuildWorkspace {
        packages: vec![app],
    };

    let mut session = BuildSession::new();
    let error = session
        .build_workspace(&workspace)
        .expect_err("unknown dependency");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("unknown package `dep`")));
    assert_eq!(session.cache_stats().misses, 0);
}

// F. Dependency cycle is a hard error and nothing builds.
#[test]
fn dependency_cycle_is_a_hard_error() {
    let project = TempProject::new("static_cycle");
    project.write("a/src/main.lang", "let A: type = uint8");
    project.write("b/src/main.lang", "let B: type = uint8");

    let mut a = package_spec("a", project.path().join("a/src"));
    a.dependencies.push(dependency("b", &["b"]));
    let mut b = package_spec("b", project.path().join("b/src"));
    b.dependencies.push(dependency("a", &["a"]));
    let workspace = BuildWorkspace {
        packages: vec![a, b],
    };

    let mut session = BuildSession::new();
    let error = session.build_workspace(&workspace).expect_err("cycle");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("cycle")));
    assert_eq!(
        session.cache_stats().misses,
        0,
        "no package should be built"
    );
}

// G. Duplicate package names are a hard error.
#[test]
fn duplicate_package_names_are_a_hard_error() {
    let project = TempProject::new("static_dup_pkg");
    project.write("app/src/main.lang", "let T: type = uint8");
    let workspace = BuildWorkspace {
        packages: vec![
            package_spec("app", project.path().join("app/src")),
            package_spec("app", project.path().join("app/src")),
        ],
    };

    let mut session = BuildSession::new();
    let error = session
        .build_workspace(&workspace)
        .expect_err("duplicate package");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("duplicate package name `app`")));
}

// H. Self-dependency is a hard error.
#[test]
fn self_dependency_is_a_hard_error() {
    let project = TempProject::new("static_self_dep");
    project.write("app/src/main.lang", "let T: type = uint8");
    let mut app = package_spec("app", project.path().join("app/src"));
    app.dependencies.push(dependency("app", &["app"]));
    let workspace = BuildWorkspace {
        packages: vec![app],
    };

    let mut session = BuildSession::new();
    let error = session
        .build_workspace(&workspace)
        .expect_err("self dependency");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("cannot depend on itself")));
}

// I. Dependency fingerprint participates in the dependent cache key.
#[test]
fn dependency_fingerprint_participates_in_dependent_cache_key() {
    let project = TempProject::new("static_dep_fp");
    project.write("dep/src/lib.lang", "let DepType: type = uint8");
    project.write("app/src/main.lang", "let T: type = uint8");

    let dep = package_spec("dep", project.path().join("dep/src"));
    let mut app = package_spec("app", project.path().join("app/src"));
    app.dependencies.push(dependency("dep", &["dep"]));
    let workspace = BuildWorkspace {
        packages: vec![dep, app],
    };

    let mut session = BuildSession::new();
    let first = session.build_workspace(&workspace).expect("first build");
    let dep_fingerprint_1 = artifact_named(&first, "dep").fingerprint.clone();
    let app_fingerprint_1 = artifact_named(&first, "app").fingerprint.clone();
    let app_cache_key_1 = artifact_named(&first, "app").metadata.cache_key.clone();

    // Change only the dependency's source.
    project.write(
        "dep/src/lib.lang",
        "let DepType: type = (uint8 a) |> struct",
    );
    let second = session.build_workspace(&workspace).expect("second build");
    let dep_fingerprint_2 = artifact_named(&second, "dep").fingerprint.clone();
    let app_fingerprint_2 = artifact_named(&second, "app").fingerprint.clone();
    let app_cache_key_2 = artifact_named(&second, "app").metadata.cache_key.clone();

    assert_ne!(dep_fingerprint_1, dep_fingerprint_2);
    assert_ne!(
        app_fingerprint_1, app_fingerprint_2,
        "dependency fingerprint must flow into the dependent fingerprint"
    );
    assert_ne!(
        app_cache_key_1, app_cache_key_2,
        "a dependency source change must invalidate the dependent cache key"
    );
}

// J. Dependency mount namespace marker exists, but dependency symbols are not
// auto-imported. This locks the non-goal for this PR.
#[test]
fn dependency_mount_marker_exists_but_symbols_are_not_imported() {
    let project = TempProject::new("static_mount_marker");
    project.write("dep/src/lib.lang", "let DepType: type = uint8");
    project.write("app/src/main.lang", "let T: type = uint8");

    let dep = package_spec("dep", project.path().join("dep/src"));
    let mut app = package_spec("app", project.path().join("app/src"));
    app.dependencies.push(dependency("dep", &["dep"]));
    let workspace = BuildWorkspace {
        packages: vec![dep, app],
    };

    let mut session = BuildSession::new();
    let result = session
        .build_workspace(&workspace)
        .expect("build workspace");

    let app_artifact = artifact_named(&result, "app");
    let capability = app_artifact.world.snapshot().capability();
    let root_context = app_artifact.world.root_context();

    assert!(
        capability
            .resolve_str_with_expectation(
                "dep",
                &root_context,
                ResolveExpectation::NamespaceSubspace
            )
            .is_ok(),
        "dependency mount namespace marker must exist"
    );
    assert!(
        capability
            .resolve_str("DepType::dep", &root_context)
            .is_err(),
        "dependency symbols must not be auto-imported in this PR"
    );
}

// Changing an explicit dependency mount (its synthetic symbols) must change the
// fingerprint / cache key and rebuild, even when source and static dependencies
// are unchanged. This guards against a stale cache hit returning the old world.
#[test]
fn explicit_mount_change_invalidates_cache_and_rebuilds_world() {
    let project = TempProject::new("static_mount_cache");
    project.write("app/src/main.lang", "let T: type = uint8");

    let workspace_with_symbol = |symbol_name: &str| {
        let mut app = package_spec("app", project.path().join("app/src"));
        app.dependency_mounts.push(
            NamespaceMount::synthetic_root("dep", vec!["dep".to_string()])
                .with_symbol(symbol_name, SymbolKind::Placeholder),
        );
        BuildWorkspace {
            packages: vec![app],
        }
    };

    let mut session = BuildSession::new();

    let first = session
        .build_workspace(&workspace_with_symbol("A"))
        .expect("first build");
    let first_app = &first.artifacts[0];
    assert_eq!(first_app.metadata.cache_status, CacheStatus::Miss);
    let first_fingerprint = first_app.fingerprint.clone();
    let first_cache_key = first_app.metadata.cache_key.clone();
    {
        let capability = first_app.world.snapshot().capability();
        let root_context = first_app.world.root_context();
        assert!(capability.resolve_str("A::dep", &root_context).is_ok());
        assert!(capability.resolve_str("B::dep", &root_context).is_err());
    }
    // Metadata explains the cache key source.
    assert_eq!(first_app.metadata.explicit_mounts.len(), 1);
    assert_eq!(
        first_app.metadata.explicit_mounts[0].synthetic_symbols[0].name,
        "A"
    );

    let second = session
        .build_workspace(&workspace_with_symbol("B"))
        .expect("second build");
    let second_app = &second.artifacts[0];
    assert_eq!(
        second_app.metadata.cache_status,
        CacheStatus::Miss,
        "changing an explicit mount must not be served from the stale cache"
    );
    assert_ne!(second_app.fingerprint, first_fingerprint);
    assert_ne!(second_app.metadata.cache_key, first_cache_key);
    {
        let capability = second_app.world.snapshot().capability();
        let root_context = second_app.world.root_context();
        assert!(
            capability.resolve_str("B::dep", &root_context).is_ok(),
            "rebuilt world must reflect the new synthetic symbol"
        );
        assert!(
            capability.resolve_str("A::dep", &root_context).is_err(),
            "rebuilt world must not return the stale synthetic symbol"
        );
    }
}

// Guard: an explicit synthetic mount and a static dependency sharing the same
// mount path must fail at build-graph validation, not later at
// CompilationWorld::from_manifest.
#[test]
fn explicit_mount_and_dependency_sharing_mount_path_fail_at_validation() {
    let project = TempProject::new("static_mount_conflict");
    project.write("dep/src/lib.lang", "let DepType: type = uint8");
    project.write("app/src/main.lang", "let T: type = uint8");

    let dep = package_spec("dep", project.path().join("dep/src"));
    let mut app = package_spec("app", project.path().join("app/src"));
    app.dependency_mounts.push(NamespaceMount::synthetic_root(
        "dep",
        vec!["dep".to_string()],
    ));
    app.dependencies.push(dependency("dep", &["dep"]));
    let workspace = BuildWorkspace {
        packages: vec![dep, app],
    };

    let mut session = BuildSession::new();
    let error = session
        .build_workspace(&workspace)
        .expect_err("mount path conflict must fail validation");
    assert!(error.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("duplicate dependency mount path")));
    assert_eq!(
        session.cache_stats().misses,
        0,
        "validation must fail before any package is built"
    );
}
