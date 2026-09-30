//! Canonical semantic-spine integration tests.
//!
//! These tests pin the canonical invariants end to end:
//! one resolved binding per call target, member views as the canonical
//! fact (never a flat Symbol/Policy aggregate), declaration-time return
//! ontology shared by core and source, ordinary let-binding of meta outcomes
//! (bind the RHS value to the LHS symbol — types have value semantics too),
//! and the single canonical P1 authority chain.

mod support;

use lang_build::{
    extract_single_call_site, BuildManifest, CompilationWorld, InvocationOutcome,
    OrdinaryInvocationContext, OrdinaryInvocationFailure, OrdinaryPipelineTrace, PolicyMode,
    Provenance, ResolverCode, SemanticOwnerKind, SemanticValuePayload,
};

use support::{build_fixture_error, build_single_fixture_world, initializer_from_source};

/// Extract the pipeline trace from an invocation result, success or failure.
/// Exposure regressions are trace facts and must stay observable even when
/// body execution of the selected candidate is not (yet) supported.
fn trace_of<'a>(
    result: &'a Result<InvocationOutcome, OrdinaryInvocationFailure>,
) -> &'a OrdinaryPipelineTrace {
    match result {
        Ok(lang_build::InvocationResult::SemanticResult {
            value: lang_build::ProjectedInvocationOutcome::Unit(u),
            ..
        }) => &u.trace,
        Ok(lang_build::InvocationResult::SemanticResult {
            value: lang_build::ProjectedInvocationOutcome::SingleMember(r),
            ..
        }) => &r.trace,
        Ok(lang_build::InvocationResult::Residual(_))
        | Ok(lang_build::InvocationResult::Diagnostic(_)) => {
            panic!("ordinary invocation did not produce a semantic result")
        }
        Err(OrdinaryInvocationFailure::NoTargetValues { trace })
        | Err(OrdinaryInvocationFailure::NoFullyAdmissibleCandidate { trace, .. })
        | Err(OrdinaryInvocationFailure::ApplicabilityUnsupported { trace, .. })
        | Err(OrdinaryInvocationFailure::Ambiguous { trace, .. })
        | Err(OrdinaryInvocationFailure::DynamicLegality { trace, .. })
        | Err(OrdinaryInvocationFailure::SelectedDelete { trace, .. })
        | Err(OrdinaryInvocationFailure::SelectedBody { trace, .. })
        | Err(OrdinaryInvocationFailure::SelectedCoreBody { trace, .. })
        | Err(OrdinaryInvocationFailure::MetaReturnTypeRootMismatch { trace, .. })
        | Err(OrdinaryInvocationFailure::ResultTypeHasNoPattern { trace, .. })
        | Err(OrdinaryInvocationFailure::MigrationResultTypeChanged { trace, .. })
        | Err(OrdinaryInvocationFailure::MigrationOutputProjectionFailed { trace })
        | Err(OrdinaryInvocationFailure::Residual { trace, .. })
        | Err(OrdinaryInvocationFailure::CyclicVal2 { trace, .. }) => trace,
    }
}

fn invoke(
    world: &mut CompilationWorld,
    spelling: &str,
    context: OrdinaryInvocationContext<'_>,
    provenance: &str,
) -> Result<InvocationOutcome, OrdinaryInvocationFailure> {
    let initializer = initializer_from_source(spelling);
    let call_site = extract_single_call_site(&initializer).expect("normalized call");
    world.invoke_ordinary_call(
        world.package_root_node(),
        &call_site,
        context,
        Provenance::new(provenance),
    )
}

#[test]
fn unknown_actual_uses_primitive_plain_and_never_world_fabricated_const() {
    let mut world = support::AssociatedFamily::new(&[
        "let first = (self, const let x): compile -> let r => { x; };",
        "let second = (self, let x): compile -> let r => { x; };",
    ]);
    let no_fabricated_modes = [];
    let call =
        extract_single_call_site(&initializer_from_source("let r = mystery probe;")).unwrap();
    let result = world.invoke_ordinary_call(
        world.package_root_node(),
        &call,
        OrdinaryInvocationContext::open_static(&no_fabricated_modes),
        Provenance::new("unknown actual defaults to Plain"),
    );
    let selected = trace_of(&result).selected.unwrap_or_else(|| {
        panic!("selection must seal before the unknown body result is diagnosed: {result:?}")
    });
    let selected = world
        .semantic_world()
        .value(selected)
        .expect("selected call entry");
    let SemanticValuePayload::CallEntry(entry) = &selected.payload else {
        panic!("probe selection is an ordinary call entry");
    };
    let formal = entry
        .closure
        .as_ref()
        .and_then(|closure| closure.head.as_ref())
        .and_then(|head| head.formal_frame().explicit_parameters.first())
        .expect("one explicit formal");
    let lang_syntax::NormPatternElem::BindingSlot(formal) = formal else {
        panic!("probe formal is a binding slot");
    };
    assert!(
        formal.policy.is_none(),
        "Plain formal must beat the const formal for an unknown actual; a fabricated const would select the other candidate"
    );
}

// ---------------------------------------------------------------------------
// Fixture build smoke: the committed semantic workspaces must build.
// ---------------------------------------------------------------------------

#[test]
fn fixture_type_binding_builds() {
    let _ = build_single_fixture_world("type_binding", "app");
}

#[test]
fn type_binding_is_fresh_symbol_no_alias_no_reroot() {
    let world = build_single_fixture_world("type_binding", "app");
    let t = world
        .semantic_world()
        .symbol_in_namespace(world.package_root_node(), "T")
        .expect("destination symbol T installed");
    let uint8 = world
        .semantic_world()
        .symbol_in_namespace(world.core_node(), "uint8")
        .expect("core uint8");

    // Fresh destination Symbol: T is its own name binding, not an alias
    // facet of uint8.
    assert_ne!(
        t.identity, uint8.identity,
        "let binding creates a fresh destination Symbol, never an alias"
    );

    // Ordinary binding semantics: the RHS value (the uint8 type value) is
    // bound to the symbol T — T reads the same PatternValue.
    assert_eq!(
        t.pure_p_pattern(),
        uint8.pure_p_pattern(),
        "the bound type value is the RHS value itself"
    );

    // No reroot: carrier rebinding does not rewrite the Pattern's owning
    // binding; uint8's PatternValue stays owned by uint8.
    let pattern = uint8.pure_p_pattern().expect("core uint8 pure-P");
    assert_eq!(
        world.semantic_world().pattern_declaration(pattern),
        Some(uint8.identity),
        "carrier rebinding must not reroot the RHS PatternValue"
    );
}

// ---------------------------------------------------------------------------
#[test]
fn callable_member_owns_function_object_and_terminal_call_entry() {
    let world = support::AssociatedFamily::new(&[
        "let member = (self, t:type):meta -> let r:type => { t; };",
    ]);
    let make_type = world.target_binding();
    assert_eq!(make_type.ordinary_value().iter().count(), 1);
    let function_value = make_type.ordinary_value().unwrap();
    let function_obj = world
        .semantic_world()
        .value(function_value)
        .expect("function object value");
    assert!(matches!(
        function_obj.payload,
        SemanticValuePayload::FunctionObject { .. }
    ));

    let entries = world
        .semantic_world()
        .associated_values_for_value(function_value, "()")
        .unwrap_or(&[]);
    assert_eq!(entries.len(), 1, "one () call entry on the function object");
    let call_obj = world
        .semantic_world()
        .value(entries[0])
        .expect("call entry value");
    assert!(matches!(
        call_obj.payload,
        SemanticValuePayload::CallEntry(_)
    ));

    // Terminal FunctionItem: the call entry has its own type/pattern and an
    // empty Val2.
    assert_ne!(function_obj.pattern, call_obj.pattern);
    assert_ne!(function_obj.type_value, call_obj.type_value);
    assert!(
        world
            .semantic_world()
            .associated_values_for_pattern(call_obj.pattern, "()")
            .is_none(),
        "call entry is terminal: no further ()"
    );
}

// ---------------------------------------------------------------------------
// ⑥ Privileged `struct` goes through the normal overload path: privilege is
// a selected-body capability, never a resolution bypass.
// ---------------------------------------------------------------------------

#[test]
fn privileged_struct_uses_the_normal_overload_path() {
    let mut world =
        CompilationWorld::from_manifest(&BuildManifest::new("app", vec!["app".to_string()]))
            .expect("core semantic world builds");
    let result = invoke(
        &mut world,
        "let T: type = (uint8 a) struct;",
        OrdinaryInvocationContext::open_static(&[PolicyMode::Const]),
        "privileged struct",
    )
    .expect("struct is selected through the ordinary spine");
    let lang_build::InvocationResult::SemanticResult {
        value: lang_build::ProjectedInvocationOutcome::SingleMember(result),
        ..
    } = result
    else {
        panic!("struct declares one complete-type result");
    };
    assert_eq!(result.trace.c0_target_values.len(), 1);
    assert_eq!(
        result.trace.c1_visible_values,
        result.trace.c0_target_values
    );
    assert_eq!(result.trace.c3_call_entries.len(), 1);
    assert!(
        result.trace.selected.is_some(),
        "privilege applies only after ordinary selection"
    );
    let lang_build::ReturnedSemanticEntity::CompleteType(returned) = &result.returned else {
        panic!("struct semantic result is complete tau");
    };
    assert!(result.complete_result[0].value.is_some());
    let owner = world
        .semantic_world()
        .pattern_owner(returned.pattern)
        .expect("struct result Pattern owner")
        .owner;
    // Direct `struct` never creates a `MetaInstance(struct, arguments)`
    // scope of its own: the complete type attaches to the ambient
    // declaration environment.
    let ambient_owner = world
        .semantic_world()
        .namespace_owner(world.package_root_node())
        .expect("package root owner");
    assert_eq!(owner, ambient_owner);
    assert!(matches!(
        world
            .semantic_world()
            .owners()
            .node(owner)
            .expect("owner node")
            .kind,
        SemanticOwnerKind::PackageRoot { .. }
    ));
}

// ---------------------------------------------------------------------------
#[test]
fn unwired_lexical_alias_creates_no_semantic_entity() {
    let error = build_fixture_error("lexical_alias_unwired", "app");
    assert_eq!(error.diagnostics.len(), 1);
    assert_eq!(
        error.diagnostics[0].code,
        Some(ResolverCode::UnsupportedLexicalAlias)
    );
    assert!(error.diagnostics[0]
        .message
        .contains("lexical alias resolution is not implemented"));
    assert!(error.diagnostics[0]
        .message
        .contains("must not install or forward a semantic entity"));
}
