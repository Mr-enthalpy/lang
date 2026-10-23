use lang_syntax::{
    normalize_program, parse, NormDecl, NormExpr, NormForm, NormPattern, NormPatternElem,
    NormPolicyAtom,
};

fn norm(source: &str) -> lang_syntax::NormProgram {
    let p = parse(source);
    assert!(
        p.diagnostics.is_empty(),
        "{}",
        lang_syntax::dump_diagnostics(&p.diagnostics)
    );
    normalize_program(&p.program)
}

#[test]
fn underscore_has_polarity_specific_material_at_arbitrary_depth() {
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(|| {
            for n in [0, 1, 2, 3, 12, 13] {
                let mut operand = "_".to_owned();
                for _ in 0..n {
                    operand = format!("({operand})$");
                }
                let p = norm(&format!("let r = {operand};"));
                let dump = lang_syntax::dump_norm_program(&p);
                assert_eq!(dump.contains("UniverseRank"), n % 2 == 0, "{dump}");
                assert!(!dump.contains("Name \"_\""), "{dump}");
                assert!(!dump.contains("PolicyLet"));
                assert!(!dump.contains("runtime"));
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn branch_head_is_the_explicit_anonymous_extraction_head() {
    let shorthand = "value |> Branch { body; };";
    let explicit = "value |> (_, let _: (Branch)$) { body; };";
    assert_eq!(
        lang_syntax::dump_ast(&parse(shorthand).program),
        lang_syntax::dump_ast(&parse(explicit).program)
    );
    let erase = |p: &lang_syntax::NormProgram| {
        lang_syntax::dump_norm_program(p)
            .lines()
            .map(|l| l.split(" origin=").next().unwrap().to_owned())
            .collect::<Vec<_>>()
    };
    assert_eq!(erase(&norm(shorthand)), erase(&norm(explicit)));
}

#[test]
fn hole_local_policy_preserves_outer_hole_identity() {
    let p = norm("let f = <p>(self, p let <p let h:type::> x:h):compile -> r => { x; };");
    let NormForm::Let(NormDecl::Let { slot, .. }) = &p.forms[0] else {
        panic!("let")
    };
    let NormExpr::Closure(c) = slot.initializer.as_deref().unwrap() else {
        panic!("closure")
    };
    let head = c.head.as_ref().unwrap();
    let outer = head.deduce[0].id;
    let NormPatternElem::BindingSlot(param) = &head.params[1] else {
        panic!("param")
    };
    let h = &param.deduce[0];
    assert!(
        matches!(&h.policy.as_ref().unwrap().constraint.atoms[0], NormPolicyAtom::HoleRef { target, .. } if *target == outer)
    );
    assert!(
        matches!(&param.annotation.as_ref().unwrap().pattern, NormPattern::HoleRef { target, .. } if *target == h.id)
    );
    assert_ne!(outer, h.id);
}

#[test]
fn bare_classifier_and_explicit_external_pattern_have_distinct_carriers() {
    for (annotation, external) in [("b", false), ("uint8::", true), ("R$", true)] {
        let p = norm(&format!("let a:{annotation} = input;"));
        let NormForm::Let(NormDecl::Let { slot, .. }) = &p.forms[0] else {
            panic!("let")
        };
        assert_eq!(
            !matches!(
                slot.annotation.as_ref().unwrap().pattern,
                NormPattern::Name { .. }
            ),
            external
        );
    }
}
