use lang_build::{
    decode_param_pattern, match_pack_param_pattern, OverloadArgShape, RestrictedParamPattern,
    SpecificityTuple,
};
use lang_syntax::{
    validate_pack_pattern_element_level, NormBindingSlot, NormOrigin, NormPattern, NormPatternElem,
    Span,
};

fn origin() -> NormOrigin {
    NormOrigin::Source(Span::new(0, 0, 1, 1))
}

fn slot(pattern: NormPattern) -> NormPatternElem {
    NormPatternElem::BindingSlot(NormBindingSlot {
        policy: None,
        has_let: false,
        deduce: Vec::new(),
        value_pattern: pattern,
        annotation: None,
        with_clause: None,
        initializer: None,
        origin: origin(),
    })
}

fn pack(name: &str) -> NormPatternElem {
    slot(NormPattern::Pack {
        inner: Box::new(NormPattern::Binder {
            name: name.to_string(),
            origin: origin(),
        }),
        origin: origin(),
    })
}

fn arg(index: usize) -> OverloadArgShape {
    OverloadArgShape {
        top_pattern_name: Some(format!("arg{index}")),
        type_symbol_id: None,
        provenance: lang_build::Provenance::new(format!("arg {index}")),
    }
}

#[test]
fn normalized_pack_validation_is_per_structural_level() {
    let duplicate = vec![pack("left"), pack("right")];
    let error = validate_pack_pattern_element_level(&duplicate)
        .expect_err("two packs at one normalized level must fail");
    assert_eq!(error.pack_count, 2);

    let nested = NormPattern::Product {
        elements: vec![
            slot(NormPattern::Product {
                elements: vec![pack("inner")],
                origin: origin(),
            }),
            pack("outer"),
        ],
        origin: origin(),
    };
    lang_syntax::validate_pack_pattern_layers(&nested)
        .expect("different normalized levels may each contain one pack");

    let same_level_nesting = NormPattern::Pack {
        inner: Box::new(NormPattern::Pack {
            inner: Box::new(NormPattern::Binder {
                name: "args".to_string(),
                origin: origin(),
            }),
            origin: origin(),
        }),
        origin: origin(),
    };
    assert_eq!(
        lang_syntax::validate_pack_pattern_layers(&same_level_nesting)
            .expect_err("adjacent packs do not create a new structural level")
            .pack_count,
        2
    );
}

#[test]
fn pack_binding_captures_the_remainder_without_counting_its_length() {
    let pattern = decode_param_pattern(&pack("args"));
    assert!(matches!(
        pattern,
        RestrictedParamPattern::PackBinder { ref name, .. } if name == "args"
    ));

    let two = match_pack_param_pattern(&pattern, &[arg(0), arg(1)]).unwrap();
    let two_hundred =
        match_pack_param_pattern(&pattern, &(0..200).map(arg).collect::<Vec<_>>()).unwrap();
    assert_eq!(two.specificity, two_hundred.specificity);
    assert_eq!(two.specificity.explicit_pack_match_count, 1);
    assert_eq!(two.pack_bindings["args"].len(), 2);
    assert_eq!(two_hundred.pack_bindings["args"].len(), 200);
}

#[test]
fn node_class_evidence_orders_explicit_above_pack_above_discards() {
    let base = SpecificityTuple {
        max_depth: 1,
        sum_depth: 1,
        ..SpecificityTuple::default()
    };
    let explicit = SpecificityTuple {
        non_discard_explicit_node_count: 1,
        ..base
    };
    let explicit_pack = SpecificityTuple {
        explicit_pack_match_count: 1,
        ..base
    };
    let discard = SpecificityTuple {
        explicit_discard_count: 1,
        ..base
    };
    let pack_discard = SpecificityTuple {
        pack_discard_count: 1,
        ..base
    };

    assert!(explicit > explicit_pack);
    assert!(explicit_pack > discard);
    assert!(discard > pack_discard);
}
