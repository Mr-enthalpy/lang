mod support;

use lang_build::{
    elaborate_binding_result_demand, elaborate_namespace_declaration_policy, normalize_p2_policy,
    MetaInstancePolicy, NamespaceDeclarationPosition, OpenHereAvailability, PolicyMode, Provenance,
    Stage,
};
use lang_syntax::{NormDecl, NormForm, NormPolicySpec};

fn policy(source: &str) -> NormPolicySpec {
    let parsed = lang_syntax::parse(&format!("{source} let x = value;"));
    assert!(parsed.diagnostics.is_empty());
    let normalized = lang_syntax::normalize_program(&parsed.program);
    let [NormForm::Let(NormDecl::Let { slot, .. })] = normalized.forms.as_slice() else {
        panic!("one policy declaration");
    };
    slot.policy.clone().unwrap()
}

#[test]
fn omitted_mode_requires_meta_and_current_open_here_for_mut() {
    for (qualification, open_here, expected) in [
        ("meta", OpenHereAvailability::Known(true), PolicyMode::Mut),
        (
            "meta",
            OpenHereAvailability::Known(false),
            PolicyMode::Const,
        ),
        (
            "close",
            OpenHereAvailability::Known(true),
            PolicyMode::Const,
        ),
        (
            "close",
            OpenHereAvailability::Unavailable,
            PolicyMode::Const,
        ),
    ] {
        let pending = elaborate_binding_result_demand(
            Some(&policy(qualification)),
            Provenance::new(qualification),
        )
        .unwrap();
        assert_eq!(pending.mode, None);
        assert_eq!(
            pending
                .complete(open_here, Provenance::new("current continuation"))
                .unwrap()
                .mode,
            expected
        );
    }
    let pending =
        elaborate_binding_result_demand(Some(&policy("meta")), Provenance::new("unknown source"))
            .unwrap();
    assert!(pending
        .complete(
            OpenHereAvailability::Unavailable,
            Provenance::new("no witness")
        )
        .unwrap_err()
        .message
        .contains("OpenHere"));
}

#[test]
fn explicit_or_deduced_mode_is_never_overwritten_by_contextual_completion() {
    for source in ["meta + const", "meta + mut", "close + const", "close + mut"] {
        let pending =
            elaborate_binding_result_demand(Some(&policy(source)), Provenance::new(source))
                .unwrap();
        let mode = pending.mode.unwrap();
        for observation in [
            OpenHereAvailability::Known(true),
            OpenHereAvailability::Known(false),
            OpenHereAvailability::Unavailable,
        ] {
            assert_eq!(
                pending
                    .clone()
                    .complete(observation, Provenance::new("explicit result"))
                    .unwrap()
                    .mode,
                mode
            );
        }
    }
    let mut deduced =
        elaborate_binding_result_demand(Some(&policy("meta")), Provenance::new("deduced")).unwrap();
    deduced.mode = Some(PolicyMode::Const);
    assert_eq!(
        deduced
            .complete(
                OpenHereAvailability::Known(true),
                Provenance::new("deduction")
            )
            .unwrap()
            .mode,
        PolicyMode::Const
    );
}

#[test]
fn completion_policy_and_evaluation_stage_are_positionally_independent() {
    let meta =
        elaborate_binding_result_demand(Some(&policy("meta + const")), Provenance::new("P1 meta"))
            .unwrap();
    assert_eq!(meta.meta_instance_policy, MetaInstancePolicy::Meta);
    assert!(
        matches!(meta.pair_query, lang_build::P1Projection::ValueDominant { value } if value.stage.is_none())
    );
    let p2 = normalize_p2_policy(&policy("meta"), Provenance::new("P2 meta")).unwrap();
    assert_eq!(p2.pair.value.stage(), Some(Stage::Meta));
    assert_eq!(p2.mode, PolicyMode::Const);
    assert!(normalize_p2_policy(&policy("close"), Provenance::new("not a stage or mode")).is_err());
    for source in ["plain", "const + mut", "meta + close"] {
        assert!(
            elaborate_binding_result_demand(Some(&policy(source)), Provenance::new(source))
                .is_err()
        );
    }
}

#[test]
fn explicit_mode_does_not_erase_meta_completion_from_a_declaration() {
    let declaration = elaborate_namespace_declaration_policy(
        Some(&policy("meta + const")),
        NamespaceDeclarationPosition::DirectTopLevel,
        Provenance::new("retained qualification"),
    )
    .unwrap();
    assert_eq!(declaration.mode, PolicyMode::Const);
    assert_eq!(declaration.meta_instance_policy, MetaInstancePolicy::Meta);
    let error = support::AssociatedFamily::try_new(&[
        "meta + const let f = (self): compile -> let r => { self; };",
    ])
    .err()
    .expect("explicit mode does not witness instance completion");
    assert!(error
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("completion consumer")));
}
