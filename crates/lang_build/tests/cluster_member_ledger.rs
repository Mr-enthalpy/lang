//! B5/B6 — cluster member ledger semantics (world level).
//!
//! The `cluster_member_ledger` fixture exercises the member-view ledger of a
//! source meta cluster construction through a real invocation:
//!
//! * B6 — each member's own written binding P1 projects the member views of
//!   its initializer's complete result.  A value-component P1 (`const let r`)
//!   admits no view of a pure-P generated type member, which is a hard error
//!   instead of silently collapsing onto the callable's function P2.
//!
//! The placeholder write (`r = expr;`, internally `PlaceholderOverwrite`)
//! cannot be exercised at world
//! level yet: the frozen v0.2 grammar has no expression-level `=` operator,
//! so the fixed write spelling is unparseable today.  Its scaffold selection
//! and
//! harvest-shape behavior (not a final write algebra) are pinned by unit
//! tests in `ordinary_invocation.rs`
//! (`select_overwrite_target`) and `overload_set.rs`
//! (`overwrite_assignment_rhs`).
//!
//! A positive member-specific P1 over a pure-P type member is also not
//! spellable: the `Absent` value-component policy has no frozen source
//! spelling, and every spellable value-component P1 (`const`, stage prefixes)
//! demands a Present value the pure-P member does not carry.

mod support;

use lang_build::{
    extract_single_call_site, CompilationWorld, InvocationOutcome, OrdinaryInvocationContext,
    OrdinaryInvocationFailure, Provenance, ValueMutability,
};
use support::{build_single_fixture_world, initializer_from_source};

fn try_invoke(
    world: &mut CompilationWorld,
    spelling: &str,
    provenance: &str,
) -> Result<InvocationOutcome, OrdinaryInvocationFailure> {
    let initializer = initializer_from_source(spelling);
    let call_site = extract_single_call_site(&initializer).expect("normalized call");
    world.invoke_ordinary_call(
        world.package_root_node(),
        &call_site,
        OrdinaryInvocationContext::open_static(&[ValueMutability::Const]),
        Provenance::new(provenance),
    )
}

fn body_error_message(result: Result<InvocationOutcome, OrdinaryInvocationFailure>) -> String {
    let Err(OrdinaryInvocationFailure::SelectedBody { failure, .. }) = result else {
        panic!("expected a hard selected-body error, got: {result:?}");
    };
    failure.diagnostic.message
}

/// B6 — the member's own written binding P1 projects the member views of its
/// initializer.  `const` is a value-component P1, and a generated type member
/// is pure-P (no value component), so the projection is empty: a hard error,
/// proving the member P1 is applied instead of collapsing onto the callable's
/// function P2.
#[test]
fn member_value_p1_over_a_pure_p_member_is_an_empty_projection_error() {
    let mut world = build_single_fixture_world("cluster_member_ledger", "app");
    let message = body_error_message(try_invoke(
        &mut world,
        "let A: type = uint8 member_value_p1;",
        "b6 member value p1",
    ));
    assert!(
        message.contains("member binding P1 admits no view"),
        "the empty member projection names the B6 rule, got: {message}"
    );
}
