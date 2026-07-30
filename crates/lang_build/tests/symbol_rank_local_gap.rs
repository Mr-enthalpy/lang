//! Non-return-slot `let x:symbol = ...` inside a restricted meta body is an
//! undefined future construct: the current stage rejects it explicitly
//! (execution gap) instead of accepting a checked-then-discarded dead local,
//! so the future pass that defines symbol-rank locals is the first to give
//! the form positive semantics.

mod support;

use lang_build::{
    extract_single_call_site, OrdinaryInvocationContext, OrdinaryInvocationFailure, Provenance,
    ValueMutability,
};
use support::{build_single_fixture_world, initializer_from_source};

#[test]
fn non_return_slot_symbol_local_is_rejected_explicitly() {
    let mut world = build_single_fixture_world("symbol_rank_local_gap", "app");
    let initializer = initializer_from_source("let R: type = uint8 dead_symbol_local;");
    let call_site = extract_single_call_site(&initializer).expect("normalized call");
    let result = world.invoke_ordinary_call(
        world.package_root_node(),
        &call_site,
        OrdinaryInvocationContext::open_static(&[ValueMutability::Const]),
        Provenance::new("symbol-rank local execution gap"),
    );
    let Err(OrdinaryInvocationFailure::SelectedBody { failure, .. }) = result else {
        panic!("a body-local `let x:symbol = ...` must fail explicitly, got: {result:?}");
    };
    assert!(
        failure
            .diagnostic
            .message
            .contains("symbol-rank local binding"),
        "the rejection must be the dedicated symbol-rank execution-gap diagnostic, got: {}",
        failure.diagnostic.message
    );
}
