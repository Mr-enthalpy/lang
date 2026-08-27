//! B9 — the three control-flow end events at execution level.
//!
//! The syntax/normalizer contract distinguishes three end forms
//! (`spec/contracts/v0.9-control-flow-end-events.md`):
//!
//! ```text
//! expr;              deliver to the directly enclosing layer
//! expr return;       return to the outermost function layer
//! expr (T return);   return to the layer selected by function-object type T
//! ```
//!
//! Execution status is asymmetric and must stay explicit: only the `expr;`
//! delivery executes in the restricted meta body evaluator.  Both
//! return-event forms are contract-complete (parsed, normalized, and bound)
//! but not yet executable; each fails with its own per-form execution-gap
//! record instead of one blanket "return not supported" message, in both the
//! cluster-construction path (`r: symbol`) and the single-type evaluation
//! path (`let r: type`).

mod support;

use lang_build::{
    extract_single_call_site, CompilationWorld, InvocationOutcome, OrdinaryInvocationContext,
    OrdinaryInvocationFailure, PolicyMode, Provenance,
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
        OrdinaryInvocationContext::open_static(&[PolicyMode::Const]),
        Provenance::new(provenance),
    )
}

fn body_error_message(result: Result<InvocationOutcome, OrdinaryInvocationFailure>) -> String {
    let Err(OrdinaryInvocationFailure::SelectedBody { failure, .. }) = result else {
        panic!("expected a hard selected-body error, got: {result:?}");
    };
    failure.diagnostic.message
}

const OUTERMOST_GAP: &str = "control-flow end `expr return;` (return to the outermost function layer) is not yet executable";
const SELECTED_GAP: &str = "control-flow end `expr (T return);` (return to the layer selected by the function-object type) is not yet executable";

/// `expr;` — the delivery to the directly enclosing layer is the one
/// executable end form: the meta construction completes and delivers its
/// cluster.
#[test]
fn nearest_layer_delivery_executes() {
    let mut world = build_single_fixture_world("control_flow_terminals", "app");
    let outcome = try_invoke(
        &mut world,
        "let A: type = uint8 deliver_nearest;",
        "b9 nearest delivery",
    )
    .expect("the `expr;` delivery terminal executes");
    let lang_build::InvocationResult::SemanticResult {
        value: lang_build::ProjectedInvocationOutcome::ClusterSymbol(meta),
        ..
    } = outcome
    else {
        panic!("meta-declared source callable returns a cluster construction");
    };
    assert_eq!(meta.construction.member_views.len(), 1);
}

/// `expr return;` in the cluster-construction path — an explicit per-form
/// execution-gap record naming the outermost-function-layer semantics.
#[test]
fn outermost_return_in_construction_path_is_an_explicit_gap() {
    let mut world = build_single_fixture_world("control_flow_terminals", "app");
    let message = body_error_message(try_invoke(
        &mut world,
        "let A: type = uint8 return_outermost;",
        "b9 outermost return, construction path",
    ));
    assert!(
        message.contains(OUTERMOST_GAP),
        "the gap record names the outermost-layer form, got: {message}"
    );
}

/// `expr (T return);` in the cluster-construction path — an explicit
/// per-form execution-gap record naming the selected-layer semantics.
#[test]
fn selected_layer_return_in_construction_path_is_an_explicit_gap() {
    let mut world = build_single_fixture_world("control_flow_terminals", "app");
    let message = body_error_message(try_invoke(
        &mut world,
        "let A: type = uint8 return_selected;",
        "b9 selected return, construction path",
    ));
    assert!(
        message.contains(SELECTED_GAP),
        "the gap record names the selected-layer form, got: {message}"
    );
}

/// `expr return;` in the single-type evaluation path (`let r: type`) — the
/// same per-form record, proving both evaluator entries report the gap
/// identically instead of collapsing onto a blanket message.
#[test]
fn outermost_return_in_evaluation_path_is_an_explicit_gap() {
    let mut world = build_single_fixture_world("control_flow_terminals", "app");
    let message = body_error_message(try_invoke(
        &mut world,
        "let A: type = uint8 forward_outermost;",
        "b9 outermost return, evaluation path",
    ));
    assert!(
        message.contains(OUTERMOST_GAP),
        "the gap record names the outermost-layer form, got: {message}"
    );
}

/// `expr (T return);` in the single-type evaluation path.
#[test]
fn selected_layer_return_in_evaluation_path_is_an_explicit_gap() {
    let mut world = build_single_fixture_world("control_flow_terminals", "app");
    let message = body_error_message(try_invoke(
        &mut world,
        "let A: type = uint8 forward_selected;",
        "b9 selected return, evaluation path",
    ));
    assert!(
        message.contains(SELECTED_GAP),
        "the gap record names the selected-layer form, got: {message}"
    );
}
