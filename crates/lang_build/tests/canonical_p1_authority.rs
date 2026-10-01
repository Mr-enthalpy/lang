//! Canonical P1 authority tests.
//!
//! These tests form explicit ordinary callable substrate material and verify that the four
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
//! The mismatch tests use the substrate builder to assert that formation fails
//! with a canonical P1 mismatch diagnostic.

mod support;

use lang_build::{
    canonical_function_object_view, extract_single_call_site, BuildManifest, CompilationWorld,
    ExplicitP1Selection, ExposedInvocationResult, OrdinaryInvocationContext,
    PatternComponentPolicy, PatternValueId, PolicyMode, PolicyPair, PolicyResultEntry, PolicyView,
    Provenance, SemanticValueId, SemanticValuePayload, SemanticValueRef, Stage,
    ValueComponentPolicy, ValuePresence,
};
use support::initializer_from_source;

/// Outer explicit const + self explicit mut(ish) must produce a
/// hard canonical-P1-mismatch diagnostic, not be silently swallowed.
#[test]
fn canonical_p1_outer_self_mismatch_is_hard_error() {
    let error = support::AssociatedFamily::try_new(&[include_str!(
        "fixtures/workspaces/canonical_p1_outer_self_mismatch/app/src/main.lang"
    )])
    .err()
    .expect("substrate P1 mismatch");
    // Substrate formation must fail with a canonical P1 mismatch diagnostic. The
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
    let error = support::AssociatedFamily::try_new(&[include_str!(
        "fixtures/workspaces/canonical_p1_cross_dimension_mismatch/app/src/main.lang"
    )])
    .err()
    .expect("substrate P1 mismatch");
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
    let world = support::AssociatedFamily::from_fixture("canonical_p1_outer_only");
    assert_canonical_p1_unified(&world, "bad");
}

/// For a declaration with only a self explicit P1, the four
/// policy authorities must all read the same canonical P1.
#[test]
fn canonical_p1_self_only_unifies_all_authorities() {
    let world = support::AssociatedFamily::from_fixture("canonical_p1_self_only");
    assert_canonical_p1_unified(&world, "bad");
}

/// For a declaration with both outer and self explicit P1 that
/// are equal, the four policy authorities must all read the same canonical P1.
#[test]
fn canonical_p1_both_equal_unifies_all_authorities() {
    let world = support::AssociatedFamily::from_fixture("canonical_p1_both_equal");
    assert_canonical_p1_unified(&world, "bad");
}

/// For a declaration with neither outer nor self explicit P1,
/// the canonical P1 is Derive(P2). The four policy authorities must all read
/// the same canonical P1.
#[test]
fn canonical_p1_neither_unifies_all_authorities() {
    let world = support::AssociatedFamily::from_fixture("canonical_p1_neither");
    assert_canonical_p1_unified(&world, "bad");
}

/// An invocation candidate carries the declared canonical P1 as its
/// `function_object_p1`. For a core candidate the declared function policy and
/// result P2 differ (`IdentityType` is exported at P1 but its result P2 is not),
/// so the test distinguishes the two coordinates.
#[test]
fn invocation_candidate_function_object_p1_matches_declared_p1() {
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
            Provenance::new("canonical function-object P1"),
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
    // member view is the complete P2 result view: it carries the result
    // type/Pattern observations,
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
fn assert_canonical_p1_unified(world: &support::AssociatedFamily, name: &str) {
    let semantic_world = world.semantic_world();
    let symbol = world.target_binding();

    // The binding has one ordinary resident and Policy projections.
    // Explicit substrate formation installs one ordinary callable resident.
    assert_eq!(
        symbol.ordinary_value().iter().count(),
        1,
        "expected exactly one ordinary callable resident for `{name}`"
    );
    let function_value_id = symbol.ordinary_value().unwrap();
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
    assert_eq!(function_obj.mode, member_view.view.mode);
    assert_eq!(call_entry_obj.mode, member_view.view.mode);
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
//   CompleteResultView(P2) -> expose under callable P1 -> outer binding P1
// ---------------------------------------------------------------------------

fn exposure_window(value_stage: Stage, mode: PolicyMode, pattern_stage: Stage) -> PolicyView {
    PolicyView {
        pair: PolicyPair {
            value: ValueComponentPolicy::Present(value_stage),
            pattern: PatternComponentPolicy {
                stage: pattern_stage,
            },
        },
        mode,
    }
}

fn value_entry(
    value_stage: Stage,
    mode: PolicyMode,
    pattern_stage: Stage,
) -> PolicyResultEntry<SemanticValueRef, PatternValueId> {
    PolicyResultEntry {
        value: Some(SemanticValueRef {
            id: SemanticValueId(7),
            type_value: support::type_lookup_fixture("canonical-p1/value-entry"),
        }),
        pattern: PatternValueId(1),
        view: PolicyView {
            pair: PolicyPair {
                value: ValueComponentPolicy::Present(value_stage),
                pattern: PatternComponentPolicy {
                    stage: pattern_stage,
                },
            },
            mode,
        },
    }
}

fn pure_p_entry(
    value_stage: Stage,
    pattern_stage: Stage,
) -> PolicyResultEntry<SemanticValueRef, PatternValueId> {
    PolicyResultEntry {
        value: None,
        pattern: PatternValueId(1),
        view: PolicyView {
            pair: PolicyPair {
                value: ValueComponentPolicy::Present(value_stage),
                pattern: PatternComponentPolicy {
                    stage: pattern_stage,
                },
            },
            mode: PolicyMode::Plain,
        },
    }
}

/// Matching concrete observations preserve the independent mode.
#[test]
fn expose_preserves_matching_single_stage_and_independent_mode() {
    let outward = exposure_window(Stage::Compile, PolicyMode::Const, Stage::Compile);
    let complete = vec![value_entry(
        Stage::Compile,
        PolicyMode::Plain,
        Stage::Compile,
    )];
    let exposed = ExposedInvocationResult::expose(outward.pair, &complete);
    assert_eq!(exposed.material.len(), 1);
    let entry = &exposed.material[0];
    assert_eq!(entry.view.pair.value.stage(), Some(Stage::Compile));
    assert_eq!(entry.view.mode, PolicyMode::Plain);
    assert_eq!(entry.view.pair.pattern.stage, Stage::Compile);
}

/// B3 — an entry whose exposed window vanishes is not part of the outward
/// result at all; whole-slot mode is not another exposure-window facet.
#[test]
fn expose_hides_entries_whose_window_vanishes() {
    let stage_disjoint = ExposedInvocationResult::expose(
        exposure_window(Stage::Meta, PolicyMode::Plain, Stage::Meta).pair,
        &[value_entry(
            Stage::Compile,
            PolicyMode::Plain,
            Stage::Compile,
        )],
    );
    assert!(stage_disjoint.material.is_empty());

    let mode_orthogonal = ExposedInvocationResult::expose(
        exposure_window(Stage::Compile, PolicyMode::Const, Stage::Compile).pair,
        &[value_entry(Stage::Compile, PolicyMode::Mut, Stage::Compile)],
    );
    assert_eq!(mode_orthogonal.material.len(), 1);
    assert_eq!(mode_orthogonal.material[0].view.mode, PolicyMode::Mut);
}

/// A matching completed query preserves material identity.
#[test]
fn expose_is_identity_under_the_same_stage_observation() {
    let complete = vec![value_entry(
        Stage::Compile,
        PolicyMode::Plain,
        Stage::Compile,
    )];
    let exposed = ExposedInvocationResult::expose(
        exposure_window(Stage::Compile, PolicyMode::Plain, Stage::Compile).pair,
        &complete,
    );
    assert_eq!(exposed.material, complete);
}

/// A pure Object retains the same Pv/Pp observation; mismatch cannot clip Pv.
#[test]
fn expose_does_not_clip_a_pure_object_into_a_different_stage() {
    let exposed = ExposedInvocationResult::expose(
        exposure_window(Stage::Runtime, PolicyMode::Plain, Stage::Compile).pair,
        &[pure_p_entry(Stage::Compile, Stage::Compile)],
    );
    assert!(
        exposed.material.is_empty(),
        "a runtime query cannot relabel a pure compile Object"
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
    let error = support::AssociatedFamily::try_new(&[include_str!(
        "fixtures/workspaces/canonical_p1_stage_mismatch/app/src/main.lang"
    )])
    .err()
    .expect("substrate P1 mismatch");
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

/// Independent internal observations still participate in canonical P1 identity.
#[test]
fn pattern_stage_dimension_mismatch_is_hard_error() {
    let outer = ExplicitP1Selection {
        pattern_stage: Some(Stage::Meta),
        ..ExplicitP1Selection::default()
    };
    let initializer = initializer_from_source("let f = (compile let self): compile => { (); };");
    let lang_syntax::NormExpr::Closure(self_formal) = initializer else {
        panic!("closure")
    };
    let derived = exposure_window(Stage::Compile, PolicyMode::Plain, Stage::Compile);
    let error = canonical_function_object_view(
        Some(&outer),
        &derived,
        &derived,
        Some(&self_formal),
        &Provenance::new("internal Pattern observation mismatch"),
    )
    .expect_err("different internal observations do not merge");
    assert!(error.message.contains("canonical P1 mismatch"));
}

/// Absent value observation is distinct from an unhidden pure Object.
#[test]
fn explicit_absent_observation_has_no_stage_coordinate() {
    let outer_explicit = ExplicitP1Selection {
        presence: Some(ValuePresence::Absent),
        ..ExplicitP1Selection::default()
    };
    let derived = exposure_window(Stage::Compile, PolicyMode::Const, Stage::Compile);
    let p2 = exposure_window(Stage::Compile, PolicyMode::Plain, Stage::Compile);
    let provenance = Provenance::new("presence-dimension acceptance");
    let selected =
        canonical_function_object_view(Some(&outer_explicit), &derived, &p2, None, &provenance)
            .expect("absence removes the value observation rather than forming an empty stage set");
    assert_eq!(selected.pair.value, ValueComponentPolicy::Absent);
    assert_eq!(selected.pair.value.stage(), None);
}

/// With neither an outer nor a self explicit P1, every
/// dimension is Derive(P2): the canonical P1 is exactly the derived pair.
#[test]
fn full_omission_derives_every_dimension_from_p2() {
    let derived = exposure_window(Stage::Compile, PolicyMode::Const, Stage::Compile);
    let p2 = exposure_window(Stage::Compile, PolicyMode::Plain, Stage::Compile);
    let provenance = Provenance::new("full-omission acceptance");
    let canonical = canonical_function_object_view(None, &derived, &p2, None, &provenance)
        .expect("full omission elaborates without error");
    assert_eq!(
        canonical, derived,
        "with no explicit P1 anywhere, the canonical P1 is Derive(P2)"
    );
}
