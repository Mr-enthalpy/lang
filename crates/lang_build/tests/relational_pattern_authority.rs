use lang_build::{
    solve_parameter_product_relation, CallableOwnerPlacement, CanonicalValueAddr,
    LocalCallableIdentity, OverloadArgShape, PackageId, PatternRelationContext,
    PatternRelationFailure, PatternValueId, Provenance, SemanticOwnerGraph, TypeValueId,
};
use lang_syntax::{normalize_program, NormDecl, NormExpr, NormForm};

fn normalized_closure(source: &str) -> lang_syntax::NormClosure {
    let parsed = lang_syntax::parse(source);
    assert!(
        parsed.diagnostics.is_empty(),
        "{}",
        lang_syntax::dump_diagnostics(&parsed.diagnostics)
    );
    let normalized = normalize_program(&parsed.program);
    let [NormForm::Let(NormDecl::Let { slot, .. })] = normalized.forms.as_slice() else {
        panic!("expected one let declaration");
    };
    let Some(NormExpr::Closure(closure)) = slot.initializer.as_deref() else {
        panic!("expected a closure initializer");
    };
    closure.clone()
}

fn callable_owner() -> lang_build::SemanticOwnerId {
    let mut owners = SemanticOwnerGraph::new();
    let package = owners.package_root(PackageId(77), "pattern-relation-test");
    owners.callable(
        package,
        LocalCallableIdentity(9),
        CallableOwnerPlacement::Ordinary,
    )
}

fn typed_value(
    value_type: TypeValueId,
    pattern: PatternValueId,
    core: CanonicalValueAddr,
) -> OverloadArgShape {
    OverloadArgShape {
        top_pattern_name: Some("diagnostic-only-name".into()),
        type_symbol_id: None,
        value_type: Some(value_type),
        pattern_value: Some(pattern),
        type_core_observation: Some(core),
        complete_type_observation: None,
        effective_view: None,
        semantic_value: None,
        is_value: true,
        provenance: Provenance::new("typed relational argument"),
    }
}

#[test]
fn unnamed_binder_binds_by_identity_without_becoming_a_wildcard() {
    let closure = normalized_closure("let f = (receiver, value) => { value; };");
    let context = PatternRelationContext::for_source_callable(&closure, callable_owner(), None)
        .expect("qualified Pattern root");
    let mut params = closure
        .head
        .as_ref()
        .unwrap()
        .formal_frame()
        .explicit_parameters
        .to_vec();
    let argument = typed_value(
        support::type_lookup_fixture("relational-pattern/unnamed"),
        PatternValueId(10),
        CanonicalValueAddr(100),
    );
    let named = solve_parameter_product_relation(&params, &[argument.clone()], &context).unwrap();
    let lang_syntax::NormPatternElem::BindingSlot(slot) = &mut params[0] else {
        panic!("slot")
    };
    let lang_syntax::NormPattern::Binder { name, identity, .. } = &mut slot.value_pattern else {
        panic!("ordinary binder")
    };
    assert!(identity.is_some());
    *name = None;
    let unnamed = solve_parameter_product_relation(&params, &[argument.clone()], &context).unwrap();
    assert_eq!(unnamed.solutions[0].local_bindings.len(), 1);
    assert_eq!(named.specificity, unnamed.specificity);
    assert!(unnamed.named_bindings().is_empty());
    assert_eq!(
        named.solutions[0].local_bindings.keys().collect::<Vec<_>>(),
        unnamed.solutions[0]
            .local_bindings
            .keys()
            .collect::<Vec<_>>()
    );
    let lang_syntax::NormPatternElem::BindingSlot(slot) = &mut params[0] else {
        unreachable!()
    };
    let lang_syntax::NormPattern::Binder { name, .. } = &mut slot.value_pattern else {
        unreachable!()
    };
    *name = Some("_".into());
    let wildcard = solve_parameter_product_relation(&params, &[argument], &context).unwrap();
    assert!(wildcard.solutions[0].local_bindings.is_empty());
}

#[test]
fn shared_deduce_hole_requires_one_relational_valuation() {
    let closure = normalized_closure("let f = <T: type>(self, x: T, y: T) -> r => { r };");
    let params = closure
        .head
        .as_ref()
        .expect("head")
        .formal_frame()
        .explicit_parameters;
    let context = PatternRelationContext::for_source_callable(&closure, callable_owner(), None)
        .expect("qualified Pattern root");

    let same = solve_parameter_product_relation(
        params,
        &[
            typed_value(
                support::type_lookup_fixture("relational-pattern/type-a"),
                PatternValueId(10),
                CanonicalValueAddr(100),
            ),
            typed_value(
                support::type_lookup_fixture("relational-pattern/type-b"),
                PatternValueId(11),
                CanonicalValueAddr(100),
            ),
        ],
        &context,
    )
    .expect("equal Core observations satisfy one shared hole");
    assert_eq!(same.solutions.len(), 1);
    assert_eq!(same.solutions[0].holes.len(), 1);
    assert_eq!(same.solutions[0].local_bindings.len(), 2);

    let different = solve_parameter_product_relation(
        params,
        &[
            typed_value(
                support::type_lookup_fixture("relational-pattern/type-a"),
                PatternValueId(10),
                CanonicalValueAddr(100),
            ),
            typed_value(
                support::type_lookup_fixture("relational-pattern/type-a"),
                PatternValueId(10),
                CanonicalValueAddr(101),
            ),
        ],
        &context,
    );
    assert!(matches!(
        different,
        Err(PatternRelationFailure::Inapplicable(_))
    ));
}

#[test]
fn bare_type_lookup_and_display_name_cannot_decide_pattern_applicability() {
    let closure = normalized_closure("let f = <T: type>(self, x: T, y: T) -> r => { r };");
    let params = closure
        .head
        .as_ref()
        .expect("head")
        .formal_frame()
        .explicit_parameters;
    let context = PatternRelationContext::for_source_callable(&closure, callable_owner(), None)
        .expect("qualified Pattern root");

    let result = solve_parameter_product_relation(
        params,
        &[
            typed_value(
                support::type_lookup_fixture("relational-pattern/shared-lookup"),
                PatternValueId(20),
                CanonicalValueAddr(200),
            ),
            typed_value(
                support::type_lookup_fixture("relational-pattern/shared-lookup"),
                PatternValueId(20),
                CanonicalValueAddr(201),
            ),
        ],
        &context,
    );
    assert!(
        matches!(result, Err(PatternRelationFailure::Inapplicable(_))),
        "equal TypeValueId and equal spelling cannot override unequal Core observations"
    );
}
mod support;
