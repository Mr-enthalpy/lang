//! Canonical P1 authority tests.
//!
//! These tests enter through source declarations and verify that the four
//! policy authorities (function object policy, call entry callable_value_policy,
//! member view value/pattern_policy, and candidate function_object_p1) all read
//! the same canonical P1. They also verify the complete-pair identity rule
//! (each spelling is completed independently across value stage /
//! value mutability / value presence / Pattern stage):
//!
//!   outer explicit + self explicit => completed pairs must agree (hard error)
//!   outer explicit only            => complete outer with Derive(P2)
//!   self explicit only             => complete self with Derive(P2)
//!   neither                        => canonical P1 = Derive(P2)
//!
//! The mismatch tests use `build_fixture_error` to assert that the build fails
//! with a canonical P1 mismatch diagnostic.

mod support;

use lang_build::{
    canonical_function_object_view, extract_single_call_site, BuildManifest, CompilationWorld,
    ExplicitP1Selection, ExposedInvocationResult, OrdinaryInvocationContext, P1Projection,
    PatternComponentPolicy, PatternValueId, PolicyMigrationRequest, PolicyMode, PolicyPair,
    PolicyResultEntry, PolicyStage, PolicyView, Provenance, ResolveExpectation, ResultPolicyDemand,
    SemanticValueId, SemanticValuePayload, SemanticValueRef, StageSet, SymbolPayload,
    ToolchainGlobalSourceRoot, TypeValueId, ValueComponentPolicy, ValuePresence,
};
use support::{
    build_fixture_error, build_single_fixture_world, fixture_root, initializer_from_source,
};

/// Outer explicit const + self explicit mut(ish) must produce a
/// hard canonical-P1-mismatch diagnostic, not be silently swallowed.
#[test]
fn canonical_p1_outer_self_mismatch_is_hard_error() {
    let error = build_fixture_error("canonical_p1_outer_self_mismatch", "app");
    // The build must fail with a canonical P1 mismatch diagnostic. The
    // specific wording is owned by `canonical_function_object_p1`.
    let found = error
        .diagnostics
        .iter()
        .any(|d| d.message.contains("canonical P1 mismatch"));
    assert!(
        found,
        "expected a canonical P1 mismatch diagnostic, got: {:?}",
        error
            .diagnostics
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

/// B2 acceptance — different dimensions written at different sites are not
/// combined. Outer writes only stage; self writes only mutability. Completing
/// each spelling against Derive(P2) yields different pairs and must fail.
#[test]
fn canonical_p1_cross_dimension_assembly_is_rejected() {
    let error = build_fixture_error("canonical_p1_cross_dimension_mismatch", "app");
    assert!(
        error.diagnostics.iter().any(|d| d
            .message
            .contains("canonical P1 mismatch: completed outer P1")),
        "expected a complete-pair canonical P1 mismatch, got: {:?}",
        error
            .diagnostics
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

/// For a declaration with only an outer explicit P1, the four
/// policy authorities must all read the same canonical P1:
///
///   function object policy        == canonical_p1
///   call entry callable_value_policy == canonical_p1
///   member view value/pattern_policy == canonical_p1
///
/// (The candidate `function_object_p1` is populated at invocation time; this
/// test checks the three declaration-time authorities. The invocation-time
/// candidate is exercised by the spine tests in `ordinary_invocation_spine.rs`.)
#[test]
fn canonical_p1_outer_only_unifies_all_authorities() {
    let world = build_single_fixture_world("canonical_p1_outer_only", "app");
    assert_canonical_p1_unified(&world, "bad");
}

/// For a declaration with only a self explicit P1, the four
/// policy authorities must all read the same canonical P1.
#[test]
fn canonical_p1_self_only_unifies_all_authorities() {
    let world = build_single_fixture_world("canonical_p1_self_only", "app");
    assert_canonical_p1_unified(&world, "bad");
}

/// For a declaration with both outer and self explicit P1 that
/// are equal, the four policy authorities must all read the same canonical P1.
#[test]
fn canonical_p1_both_equal_unifies_all_authorities() {
    let world = build_single_fixture_world("canonical_p1_both_equal", "app");
    assert_canonical_p1_unified(&world, "bad");
}

/// For a declaration with neither outer nor self explicit P1,
/// the canonical P1 is Derive(P2). The four policy authorities must all read
/// the same canonical P1.
#[test]
fn canonical_p1_neither_unifies_all_authorities() {
    let world = build_single_fixture_world("canonical_p1_neither", "app");
    assert_canonical_p1_unified(&world, "bad");
}

/// S7 — no independent complete P3: the invocation-time candidate carries exactly the canonical
/// P1 as its `function_object_p1`.  For a core candidate the declared
/// function policy (canonical P1) and the result P2 genuinely differ
/// (`IdentityType` is exported at P1 but its result P2 is not), so this
/// test proves the candidate does not smuggle the result P2 — or any fresh
/// third policy — into the function-object P1 authority.
#[test]
fn invocation_candidate_function_object_p1_is_canonical_p1_no_p3() {
    let mut world =
        CompilationWorld::from_manifest(&BuildManifest::new("app", vec!["app".to_string()]))
            .expect("core semantic world builds");
    let initializer = initializer_from_source("let result = uint8 IdentityType;");
    let call_site = extract_single_call_site(&initializer).expect("normalized core call");
    let result = world
        .invoke_ordinary_call(
            world.package_root_node(),
            &call_site,
            OrdinaryInvocationContext::open_static(&[PolicyMode::Const]),
            Provenance::new("S7 no-independent-P3 regression"),
        )
        .expect("core primitive is selected through the ordinary spine");
    let lang_build::InvocationResult::SemanticResult {
        value: lang_build::ProjectedInvocationOutcome::SingleMember(result),
        ..
    } = result
    else {
        panic!("expected ordinary outcome");
    };
    let selected = &result.selected;

    // Read the declaration-boundary authorities from the selected call entry.
    let call_entry_obj = world
        .semantic_world()
        .value(selected.call_entry_value)
        .expect("selected call entry exists");
    let SemanticValuePayload::CallEntry(entry) = &call_entry_obj.payload else {
        panic!("expected CallEntry payload");
    };

    // The candidate's function-object P1 IS the canonical P1 — the same
    // authority as the call entry's callable_value_policy and the call
    // entry object's own policy.  No re-derivation, no fresh policy.
    assert_eq!(
        selected.function_object_view.pair, entry.callable_view.pair,
        "candidate function_object_p1 must read the canonical P1"
    );
    assert_eq!(
        selected.function_object_view.pair, call_entry_obj.policy,
        "call entry object policy and candidate function_object_p1 are the same canonical P1"
    );

    // P2 stays a separately stored result-domain authority. This particular
    // pure-meta core declaration happens to give P1 and P2 equal values after
    // declaration visibility/export were removed from PolicyPair; equality of
    // the values does not create a third policy coordinate.
    assert_eq!(
        selected.complete_result_view.pair, entry.complete_result_view.pair,
        "candidate result P2 must read the declared complete result policy"
    );

    // Layered result exposure.  The complete result
    // member view is the P2 compatibility domain (CompleteResultDomain):
    // it carries the result P2 type/pattern compatibility information,
    // NOT the outward visibility of the invocation result.
    let view = &result.complete_result[0];
    assert_eq!(
        view.view.pair.value,
        selected.complete_result_view.pair.value
    );
    assert_eq!(
        view.view.pair.pattern,
        selected.complete_result_view.pair.pattern
    );

    // The outward exposure layer (ExposedInvocationResult) reads the
    // canonical P1 — the same single output authority as the migration
    // output endpoint — independently of the complete-result P2 field.
    let exposed = result.exposed();
    assert_eq!(
        exposed.outward_policy, selected.function_object_view.pair,
        "invocation result outward visibility is the canonical P1"
    );
    assert_eq!(
        exposed.material.len(),
        result.complete_result.len(),
        "the exposure window of this core callable covers its complete result \
         P2 domain, so no entry is hidden here"
    );

    // Outside migration there is no output endpoint coordinate at all —
    // nothing for a third policy to hide in.
    assert!(selected.migration_input_endpoint.is_none());
    assert!(selected.migration_output_endpoint.is_none());
}

/// Helper: assert that the function object policy, call entry
/// callable_value_policy, and member view value/pattern_policy all read the
/// same `PolicyPair` for the named callable symbol.
fn assert_canonical_p1_unified(world: &lang_build::CompilationWorld, name: &str) {
    let semantic_world = world.semantic_world();
    let package_root = world.package_root_node();
    let symbol = semantic_world
        .symbol_in_namespace(package_root, name)
        .unwrap_or_else(|| panic!("`{name}` symbol should be registered"));

    // The symbol cell has sibling_vals (function objects) and member_views.
    // For an ordinary `let name = closure` declaration, there is exactly one
    // sibling function object and one corresponding member view.
    assert_eq!(
        symbol.sibling_vals.len(),
        1,
        "expected exactly one sibling function object for `{name}`"
    );
    let function_value_id = symbol.sibling_vals[0];
    let function_obj = semantic_world
        .value(function_value_id)
        .expect("function object exists");
    let function_object_policy = function_obj.policy.clone();

    // Look up the call entry via the function object's associated Val2["()"].
    let call_entries = semantic_world
        .associated_values_for_value(function_value_id, "()")
        .unwrap_or(&[]);
    assert_eq!(
        call_entries.len(),
        1,
        "expected exactly one () call entry for `{name}`"
    );
    let call_entry_id = call_entries[0];
    let call_entry_obj = semantic_world
        .value(call_entry_id)
        .expect("call entry exists");
    let call_entry_policy = match &call_entry_obj.payload {
        SemanticValuePayload::CallEntry(entry) => entry.callable_view.pair.clone(),
        other => panic!("expected CallEntry payload, got {other:?}"),
    };

    // Member view policy — there should be exactly one for the function
    // object, and its value/pattern policy must match the canonical P1.
    assert_eq!(
        symbol.member_views.len(),
        1,
        "expected exactly one member view for `{name}`"
    );
    let member_view = &symbol.member_views[0];
    let member_value_policy = member_view.view.pair.value.clone();
    let member_pattern_policy = member_view.view.pair.pattern.clone();

    // All three authorities must read the same canonical P1.
    assert_eq!(
        function_object_policy, call_entry_policy,
        "function object policy != call entry callable_value_policy for `{name}`"
    );
    assert_eq!(
        function_object_policy.value, member_value_policy,
        "function object policy.value != member view value_policy for `{name}`"
    );
    assert_eq!(
        function_object_policy.pattern, member_pattern_policy,
        "function object policy.pattern != member view pattern_policy for `{name}`"
    );
}

// ---------------------------------------------------------------------------
// The exposure window is a real slice restriction and
// the ordinary binding path must pass through it:
//
//   CompleteResultDomain(P2) -> expose under callable P1 -> outer binding P1
// ---------------------------------------------------------------------------

fn stage_set(stages: &[PolicyStage]) -> StageSet {
    let mut set = StageSet::new();
    for stage in stages {
        set.insert(*stage);
    }
    set
}

fn exposure_window(
    value_stages: &[PolicyStage],
    mode: PolicyMode,
    pattern_stages: &[PolicyStage],
) -> PolicyView {
    PolicyView {
        pair: PolicyPair {
            value: ValueComponentPolicy {
                stages: stage_set(value_stages),
                presence: ValuePresence::Present,
            },
            pattern: PatternComponentPolicy {
                stages: stage_set(pattern_stages),
            },
        },
        mode,
    }
}

fn value_entry(
    value_stages: &[PolicyStage],
    mode: PolicyMode,
    pattern_stages: &[PolicyStage],
) -> PolicyResultEntry<SemanticValueRef, PatternValueId> {
    PolicyResultEntry {
        value: Some(SemanticValueRef {
            id: SemanticValueId(7),
            type_value: TypeValueId(3),
        }),
        pattern: PatternValueId(1),
        view: PolicyView {
            pair: PolicyPair {
                value: ValueComponentPolicy {
                    stages: stage_set(value_stages),
                    presence: ValuePresence::Present,
                },
                pattern: PatternComponentPolicy {
                    stages: stage_set(pattern_stages),
                },
            },
            mode,
        },
    }
}

fn pure_p_entry(
    value_stages: &[PolicyStage],
    pattern_stages: &[PolicyStage],
) -> PolicyResultEntry<SemanticValueRef, PatternValueId> {
    PolicyResultEntry {
        value: None,
        pattern: PatternValueId(1),
        view: PolicyView {
            pair: PolicyPair {
                value: ValueComponentPolicy {
                    stages: stage_set(value_stages),
                    presence: ValuePresence::Absent,
                },
                pattern: PatternComponentPolicy {
                    stages: stage_set(pattern_stages),
                },
            },
            mode: PolicyMode::Plain,
        },
    }
}

/// B3 — a constrained canonical P1 crops the pair's stage window while the
/// material's primitive whole-slot mode remains independent.
#[test]
fn expose_crops_stage_window_and_unconstrained_mutability() {
    let outward = exposure_window(
        &[PolicyStage::Compile],
        PolicyMode::Const,
        &[PolicyStage::Compile],
    );
    let complete = vec![value_entry(
        &[PolicyStage::Meta, PolicyStage::Compile],
        PolicyMode::Plain,
        &[PolicyStage::Meta, PolicyStage::Compile],
    )];
    let exposed = ExposedInvocationResult::expose(outward.pair, &complete);
    assert_eq!(exposed.material.len(), 1);
    let entry = &exposed.material[0];
    assert_eq!(
        entry.view.pair.value.stages,
        stage_set(&[PolicyStage::Compile])
    );
    assert_eq!(entry.view.mode, PolicyMode::Plain);
    assert_eq!(
        entry.view.pair.pattern.stages,
        stage_set(&[PolicyStage::Compile])
    );
}

/// B3 — an entry whose exposed window vanishes is not part of the outward
/// result at all; whole-slot mode is not another exposure-window facet.
#[test]
fn expose_hides_entries_whose_window_vanishes() {
    let stage_disjoint = ExposedInvocationResult::expose(
        exposure_window(
            &[PolicyStage::Meta],
            PolicyMode::Plain,
            &[PolicyStage::Meta],
        )
        .pair,
        &[value_entry(
            &[PolicyStage::Compile],
            PolicyMode::Plain,
            &[PolicyStage::Compile],
        )],
    );
    assert!(stage_disjoint.material.is_empty());

    let mode_orthogonal = ExposedInvocationResult::expose(
        exposure_window(&[PolicyStage::Compile], PolicyMode::Const, &[]).pair,
        &[value_entry(
            &[PolicyStage::Compile],
            PolicyMode::Mut,
            &[PolicyStage::Compile],
        )],
    );
    assert_eq!(mode_orthogonal.material.len(), 1);
    assert_eq!(mode_orthogonal.material[0].view.mode, PolicyMode::Mut);
}

/// B3 — when the canonical P1 is the P2 derivation (no explicit P1 written),
/// the window is a superset of the material and exposure is an identity.
#[test]
fn expose_is_identity_under_the_derived_superset_window() {
    let complete = vec![value_entry(
        &[PolicyStage::Compile],
        PolicyMode::Plain,
        &[PolicyStage::Compile],
    )];
    let exposed = ExposedInvocationResult::expose(
        exposure_window(
            &[PolicyStage::Meta, PolicyStage::Compile],
            PolicyMode::Plain,
            &[PolicyStage::Meta, PolicyStage::Compile],
        )
        .pair,
        &complete,
    );
    assert_eq!(exposed.material, complete);
}

/// B3 — a pure-P entry is carried by its Pattern facet: its recorded static
/// value stages are clipped to the window, but an empty value window does
/// not hide the entry.
#[test]
fn expose_keeps_pure_p_entries_with_clipped_value_stages() {
    let exposed = ExposedInvocationResult::expose(
        exposure_window(
            &[PolicyStage::Runtime],
            PolicyMode::Plain,
            &[PolicyStage::Compile],
        )
        .pair,
        &[pure_p_entry(
            &[PolicyStage::Compile],
            &[PolicyStage::Compile],
        )],
    );
    assert_eq!(exposed.material.len(), 1);
    let entry = &exposed.material[0];
    assert!(entry.value.is_none());
    assert!(entry.view.pair.value.stages.is_empty());
    assert_eq!(
        entry.view.pair.pattern.stages,
        stage_set(&[PolicyStage::Compile])
    );
}

/// B3 — the exposure window is computed on a REAL invocation result that
/// travelled the full ordinary spine, not only on hand-built entries: the
/// transport callables declare an explicit outer mode (`const let uint8` /
/// `mut let uint8`) while the complete result carries its own concrete mode;
/// the pair exposure is cropped to the selected callable's canonical P1
/// without deriving either mode from the other.
#[test]
fn exposure_crops_a_real_invocation_result_under_the_canonical_p1() {
    let mut manifest = BuildManifest::new("app", vec!["app".to_string()]);
    manifest
        .global_implementation_roots
        .push(ToolchainGlobalSourceRoot::under(
            fixture_root()
                .join("global_implementation")
                .join("uint8_transport"),
            vec!["core".to_string(), "uint8".to_string()],
        ));
    let mut world = CompilationWorld::from_manifest(&manifest).expect("transport bundle builds");

    let uint8_type = match &world
        .resolve_with_expectation("uint8", ResolveExpectation::TypeObject)
        .expect("core uint8 type")
        .payload
    {
        SymbolPayload::Type(t) => t.represented_type,
        _ => panic!("uint8 resolves as a Type object"),
    };
    let source_policy = exposure_window(
        &[PolicyStage::Compile],
        PolicyMode::Const,
        &[PolicyStage::Compile],
    );
    let target_policy = exposure_window(
        &[PolicyStage::Runtime],
        PolicyMode::Mut,
        &[PolicyStage::Compile],
    );
    let source = world
        .install_semantic_value(
            uint8_type,
            source_policy.pair.clone(),
            Provenance::new("B3 compile uint8 source value"),
        )
        .expect("installed source value");
    let request = PolicyMigrationRequest::new(
        source_policy,
        ResultPolicyDemand {
            pair_query: P1Projection::Pair(target_policy.pair),
            mode: target_policy.mode,
        },
        uint8_type,
        source,
        Provenance::new("B3 const compile -> mut runtime demand"),
    )
    .expect("legal migration request");
    let migration = world
        .invoke_policy_migration(&request)
        .expect("migration invocation succeeds");
    let invocation = &migration.invocation;

    let raw = &invocation.complete_result[0];
    assert_eq!(raw.view.mode, invocation.selected.complete_result_view.mode);
    let canonical_mode = invocation.selected.function_object_view.mode;

    let exposed = invocation.exposed();
    assert_eq!(exposed.material.len(), 1);
    assert_eq!(
        exposed.material[0].view.mode, raw.view.mode,
        "stage exposure preserves the complete result's orthogonal mode"
    );
    assert_eq!(canonical_mode, PolicyMode::Mut);
}

/// Boundary fact — a stage-only outer declaration prefix
/// (`compile let narrow = ...`) IS an explicit canonical P1 value-stage
/// selection: the complete `Pv:Pp` elaboration no longer degrades a
/// stage-only policy to "no explicit P1".  The canonical P1 value window is
/// cropped to `compile`, so the downstream `meta let X` result demand rejects
/// the producer before maxima (or, equivalently for a non-call result, cannot
/// satisfy the completed view). The build must fail either way; it may not
/// widen the declared P1.
#[test]
fn stage_only_outer_prefix_is_an_explicit_canonical_p1() {
    let error = build_fixture_error("s4_stage_prefix_is_p1", "app");
    let found = error.diagnostics.iter().any(|d| {
        d.message.contains("cannot satisfy binding P1")
            || d.message
                .contains("requested binding policy selects no runtime value slice")
            || d.message.contains("no fully admissible candidate")
    });
    assert!(
        found,
        "the stage-only `compile` prefix on `narrow` is an explicit P1, so \
         the `meta let X` demand must remain outside the cropped result \
         position view, got: {:?}",
        error
            .diagnostics
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

/// B3 positive control — a binding P1 inside the exposure window still
/// succeeds through the same gated path.
#[test]
fn binding_inside_the_exposure_window_succeeds() {
    let world = build_single_fixture_world("s4_exposure_window_pass", "app");
    assert!(
        world
            .semantic_world()
            .symbol_in_namespace(world.package_root_node(), "X")
            .is_some(),
        "`X` binds through the exposure window"
    );
}

// ---------------------------------------------------------------------------
// Per-dimension canonical P1 elaboration:
// stage / mutability / presence / Pattern-stage disagreements between the
// outer explicit P1 and the written-self explicit P1 are hard errors at the
// single elaboration point; only full omission derives from P2.
// ---------------------------------------------------------------------------

/// Outer explicit `meta` vs self explicit `compile let self`
/// disagree on the value-stage dimension: hard error at elaboration.
#[test]
fn value_stage_dimension_mismatch_is_hard_error() {
    let error = build_fixture_error("canonical_p1_stage_mismatch", "app");
    let found = error.diagnostics.iter().any(|d| {
        d.message
            .contains("canonical P1 mismatch: completed outer P1")
    });
    assert!(
        found,
        "expected a value-stage-dimension canonical P1 mismatch, got: {:?}",
        error
            .diagnostics
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

/// Outer explicit `compile:meta` vs self explicit
/// `compile:compile let self` agree on the value component but disagree on
/// the Pattern-stage dimension: hard error at elaboration.
#[test]
fn pattern_stage_dimension_mismatch_is_hard_error() {
    let error = build_fixture_error("canonical_p1_pattern_mismatch", "app");
    let found = error.diagnostics.iter().any(|d| {
        d.message
            .contains("canonical P1 mismatch: completed outer P1")
    });
    assert!(
        found,
        "expected a Pattern-stage-dimension canonical P1 mismatch, got: {:?}",
        error
            .diagnostics
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}

/// The presence dimension participates in the merge on its own:
/// an explicit `Pv = absent` selection that would be recombined with
/// non-empty derived value stages / mutability violates the canonical P1
/// value-component invariant and is a hard error, not a silent recombination.
#[test]
fn presence_dimension_absent_recombination_is_hard_error() {
    let outer_explicit = ExplicitP1Selection {
        presence: Some(ValuePresence::Absent),
        ..ExplicitP1Selection::default()
    };
    let derived = exposure_window(
        &[PolicyStage::Compile],
        PolicyMode::Const,
        &[PolicyStage::Compile],
    );
    let p2 = exposure_window(
        &[PolicyStage::Compile],
        PolicyMode::Plain,
        &[PolicyStage::Compile],
    );
    let provenance = Provenance::new("presence-dimension acceptance");
    let error =
        canonical_function_object_view(Some(&outer_explicit), &derived, &p2, None, &provenance)
            .expect_err(
                "an absent explicit presence over a present derived value component must fail",
            );
    assert!(
        error.message.contains("`Pv = absent` cannot carry"),
        "expected the absent-value invariant diagnostic, got: {}",
        error.message
    );
}

/// With neither an outer nor a self explicit P1, every
/// dimension is Derive(P2): the canonical P1 is exactly the derived pair.
#[test]
fn full_omission_derives_every_dimension_from_p2() {
    let derived = exposure_window(
        &[PolicyStage::Meta, PolicyStage::Compile],
        PolicyMode::Const,
        &[PolicyStage::Compile],
    );
    let p2 = exposure_window(
        &[PolicyStage::Meta, PolicyStage::Compile],
        PolicyMode::Plain,
        &[PolicyStage::Compile],
    );
    let provenance = Provenance::new("full-omission acceptance");
    let canonical = canonical_function_object_view(None, &derived, &p2, None, &provenance)
        .expect("full omission elaborates without error");
    assert_eq!(
        canonical, derived,
        "with no explicit P1 anywhere, the canonical P1 is Derive(P2)"
    );
}
