use lang_syntax::{NormClosureBody, NormDecl, NormExpr, NormForm, NormPattern, NormPolicyAtom};

#[test]
fn runtime_callable_retains_distinct_output_hole_policies() {
    let parsed = lang_syntax::parse(
        "compile let f=(self,x):runtime -> let <runtime let r,compile let t> r:t => { x; };",
    );
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let norm = lang_syntax::normalize_program(&parsed.program);
    let NormForm::Let(NormDecl::Let { slot, .. }) = &norm.forms[0] else {
        panic!("let");
    };
    let NormExpr::Closure(closure) = slot.initializer.as_deref().unwrap() else {
        panic!("closure");
    };
    let head = closure.head.as_ref().unwrap();
    let result = head.returns.as_ref().unwrap();
    for (hole, spelling) in result.deduce.iter().zip(["runtime", "compile"]) {
        assert!(
            matches!(&hole.policy.as_ref().unwrap().constraint.atoms[0],NormPolicyAtom::Name {text,..} if text==spelling)
        );
    }
    assert_eq!(result.deduce.len(), 2);
    assert_ne!(result.deduce[0].id, result.deduce[1].id);
    assert!(
        matches!(result.value_pattern,NormPattern::HoleRef {target,..} if target==result.deduce[0].id)
    );
    assert!(
        matches!(result.annotation.as_ref().unwrap().pattern,NormPattern::HoleRef {target,..} if target==result.deduce[1].id)
    );
}

#[test]
fn reify_joint_extraction_preserves_each_holes_policy_and_identity() {
    let parsed=lang_syntax::parse("let compare=(self,obj):reify -> _:bool:: => { let <runtime let o, compile let t> o:t = obj; o; };");
    assert!(parsed.diagnostics.is_empty());
    let norm = lang_syntax::normalize_program(&parsed.program);
    let NormForm::Let(NormDecl::Let { slot, .. }) = &norm.forms[0] else {
        panic!("let")
    };
    let Some(NormExpr::Closure(closure)) = slot.initializer.as_deref() else {
        panic!("closure")
    };
    let p2 = closure.head.as_ref().unwrap().call_policy.as_ref().unwrap();
    assert!(matches!(&p2.constraint.atoms[0], NormPolicyAtom::Name {text,..} if text=="reify"));
    let NormClosureBody::Block(body) = &closure.body else {
        panic!("body")
    };
    let NormForm::Let(NormDecl::Let { slot, .. }) = &body.forms[0] else {
        panic!("joint binding")
    };
    let (value, classifier) = (&slot.deduce[0], &slot.deduce[1]);
    assert_ne!(value.id, classifier.id);
    for (hole, spelling) in [(value, "runtime"), (classifier, "compile")] {
        assert!(
            matches!(&hole.policy.as_ref().unwrap().constraint.atoms[0],NormPolicyAtom::Name {text,..} if text==spelling)
        );
    }
    assert!(matches!(slot.value_pattern,NormPattern::HoleRef {target,..} if target==value.id));
    assert!(
        matches!(slot.annotation.as_ref().unwrap().pattern,NormPattern::HoleRef {target,..} if target==classifier.id)
    );
}
