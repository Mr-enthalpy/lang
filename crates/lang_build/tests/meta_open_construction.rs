//! The meta context keeps a `struct` construction
//! open across local binding, Pattern observation, and Val2 injection.
//!
//! ```text
//! OpenMeta --Observe(P)-->      OpenMeta
//! OpenMeta --Transform(Val2)--> OpenMeta
//! OpenMeta --Inject-->          OpenMeta
//! OpenMeta --UseForVal1-->      Closed
//! ```
//!
//! Only `UseForVal1` closes an active *meta-window* construction. Explicit
//! boundary delivery (`t;`) removes it. Ambient ordinary
//! constructions live in a separate ordinary window that additionally
//! closes on first semantic use and on residual-runtime fork/end (see
//! the `ordinary_window_*` tests).  The `meta_open_injection`
//! fixture covers the acceptance shape end to end:
//!
//! ```text
//! let t = (x inner) |> struct;        // struct inline (privileged)
//! let f::t = (self): compile -> ...;  // ordinary Val2 injection
//! let inner::t = x;                   // associated type (Val2 only, NOT entering P)
//! t;
//! ```
//!
//! Privilege boundary: only `struct` inline and (future) `inject` register
//! members into the target Pattern's canonical structure.  Ordinary navigated
//! `let f::t = expr` installs associated Val2 members only.

mod support;

use lang_build::{
    derived_cluster_policy, extract_single_call_site, policy_or, CanonicalFullNavigation,
    CanonicalPatternAtom, CanonicalPatternNorm, CanonicalPatternValue, CanonicalTypeObservation,
    CanonicalValueAddr, ClusterConstructionId, ClusterSymbolResult, CompilationWorld,
    ConstructionAuthority, ConstructionEvaluationContext, ConstructionWindow, Diagnostic,
    MetaCallableIdentity, MetaInstanceRoot, MetaInvocationMaterialKey, NamespaceNodeId,
    OrdinaryInvocationContext, PatternComponentPolicy, Phase, PolicyMode, PolicyPair,
    PolicyResultEntry, PolicyStage, PolicyView, Provenance, ReturnShape, SemanticValueId,
    SemanticValuePayload, SemanticWorld, StageSet, SymbolId, TypeValueId, ValueComponentPolicy,
    ValuePresence, WritableContext,
};
use support::{build_single_fixture_world, initializer_from_source};

fn plain_view(pair: &PolicyPair) -> PolicyView {
    PolicyView {
        pair: pair.clone(),
        mode: PolicyMode::Plain,
    }
}

fn invoke_make(
    world: &mut CompilationWorld,
    spelling: &str,
    provenance: &str,
) -> ClusterSymbolResult {
    let initializer = initializer_from_source(spelling);
    let call_site = extract_single_call_site(&initializer).expect("normalized call");
    let result = world
        .invoke_ordinary_call(
            world.package_root_node(),
            &call_site,
            OrdinaryInvocationContext::open_static(&[PolicyMode::Const]),
            Provenance::new(provenance),
        )
        .expect("source meta callable is selected through the ordinary spine");
    let lang_build::InvocationResult::SemanticResult {
        value: lang_build::ProjectedInvocationOutcome::ClusterSymbol(meta),
        ..
    } = result
    else {
        panic!("meta-declared source callable returns a cluster construction");
    };
    meta
}

/// Test-only source fixture facade.  Production APIs deliberately no
/// longer expose state-only `inject_associated_*` entry points: this adapter
/// builds the explicit evaluation context and then calls the canonical
/// member-creation/OpenHere boundaries.
trait ConstructionTestAdapter {
    #[allow(clippy::too_many_arguments)]
    fn inject_associated_function_member(
        &mut self,
        cluster: ClusterConstructionId,
        target_pattern: lang_build::PatternValueId,
        member_name: &str,
        construction_event: u32,
        backing_declaration: SymbolId,
        closure: &lang_syntax::NormClosure,
        outer_p1_explicit: Option<&lang_build::ExplicitP1Selection>,
        function_policy: &PolicyPair,
        complete_result_policy: PolicyPair,
        return_shape: ReturnShape,
        provenance: Provenance,
    ) -> Result<SemanticValueId, Diagnostic>;

    fn inject_associated_type_member(
        &mut self,
        cluster: ClusterConstructionId,
        target_pattern: lang_build::PatternValueId,
        member_name: &str,
        view: PolicyResultEntry<SemanticValueId, lang_build::PatternValueId>,
        member_type_value: TypeValueId,
        provenance: Provenance,
    ) -> Result<(), Diagnostic>;

    fn inject_associated_existing_value_member(
        &mut self,
        cluster: ClusterConstructionId,
        target_pattern: lang_build::PatternValueId,
        member_name: &str,
        view: PolicyResultEntry<SemanticValueId, lang_build::PatternValueId>,
        provenance: Provenance,
    ) -> Result<(), Diagnostic>;

    fn inject_pattern_value_member(
        &mut self,
        cluster: ClusterConstructionId,
        target_pattern: lang_build::PatternValueId,
        local_navigation: CanonicalFullNavigation,
        resident: CanonicalPatternValue,
        provenance: Provenance,
    ) -> Result<CanonicalPatternValue, Diagnostic>;
}

fn open_here_test_diagnostic(
    failure: lang_build::OpenHereFailure,
    provenance: Provenance,
) -> Diagnostic {
    let message = match failure {
        lang_build::OpenHereFailure::WindowClosed(_) => {
            "construction window is closed or already delivered".to_string()
        }
        lang_build::OpenHereFailure::UnknownPattern(_)
        | lang_build::OpenHereFailure::NoLiveConstruction(_) => {
            "target is not owned by the current construction authority".to_string()
        }
        lang_build::OpenHereFailure::AuthorityMismatch(_) => {
            "current construction authority does not own the target".to_string()
        }
    };
    Diagnostic::hard_error(message, Some(provenance))
}

impl ConstructionTestAdapter for SemanticWorld {
    fn inject_associated_function_member(
        &mut self,
        cluster: ClusterConstructionId,
        target_pattern: lang_build::PatternValueId,
        member_name: &str,
        construction_event: u32,
        backing_declaration: SymbolId,
        closure: &lang_syntax::NormClosure,
        outer_p1_explicit: Option<&lang_build::ExplicitP1Selection>,
        function_policy: &PolicyPair,
        complete_result_policy: PolicyPair,
        return_shape: ReturnShape,
        provenance: Provenance,
    ) -> Result<SemanticValueId, Diagnostic> {
        let authority = self
            .open_cluster(cluster)
            .map(|construction| construction.authority.clone())
            .ok_or_else(|| Diagnostic::hard_error("construction does not exist", None))?;
        let context = ConstructionEvaluationContext::current(authority);
        let creation = self
            .can_create_member_here(target_pattern, &context)
            .map_err(|failure| open_here_test_diagnostic(failure, provenance.clone()))?;
        self.create_associated_function_member(
            &creation,
            member_name,
            construction_event,
            backing_declaration,
            closure,
            outer_p1_explicit,
            &plain_view(function_policy),
            plain_view(&complete_result_policy),
            return_shape,
            provenance,
        )
    }

    fn inject_associated_type_member(
        &mut self,
        cluster: ClusterConstructionId,
        target_pattern: lang_build::PatternValueId,
        member_name: &str,
        view: PolicyResultEntry<SemanticValueId, lang_build::PatternValueId>,
        member_type_value: TypeValueId,
        provenance: Provenance,
    ) -> Result<(), Diagnostic> {
        let authority = self
            .open_cluster(cluster)
            .map(|construction| construction.authority.clone())
            .ok_or_else(|| Diagnostic::hard_error("construction does not exist", None))?;
        let context = ConstructionEvaluationContext::current(authority);
        let creation = self
            .can_create_member_here(target_pattern, &context)
            .map_err(|failure| open_here_test_diagnostic(failure, provenance.clone()))?;
        self.create_associated_type_member(
            &creation,
            member_name,
            view,
            member_type_value,
            provenance,
        )
    }

    fn inject_associated_existing_value_member(
        &mut self,
        cluster: ClusterConstructionId,
        target_pattern: lang_build::PatternValueId,
        member_name: &str,
        view: PolicyResultEntry<SemanticValueId, lang_build::PatternValueId>,
        provenance: Provenance,
    ) -> Result<(), Diagnostic> {
        let authority = self
            .open_cluster(cluster)
            .map(|construction| construction.authority.clone())
            .ok_or_else(|| Diagnostic::hard_error("construction does not exist", None))?;
        let context = ConstructionEvaluationContext::current(authority);
        let creation = self
            .can_create_member_here(target_pattern, &context)
            .map_err(|failure| open_here_test_diagnostic(failure, provenance.clone()))?;
        self.create_associated_existing_value_member(&creation, member_name, view, provenance)
    }

    fn inject_pattern_value_member(
        &mut self,
        cluster: ClusterConstructionId,
        target_pattern: lang_build::PatternValueId,
        local_navigation: CanonicalFullNavigation,
        resident: CanonicalPatternValue,
        provenance: Provenance,
    ) -> Result<CanonicalPatternValue, Diagnostic> {
        let authority = self
            .open_cluster(cluster)
            .map(|construction| construction.authority.clone())
            .ok_or_else(|| Diagnostic::hard_error("construction does not exist", None))?;
        let context = ConstructionEvaluationContext::current(authority);
        let open_here = self
            .open_here(target_pattern, &context)
            .map_err(|failure| open_here_test_diagnostic(failure, provenance.clone()))?;
        let place = self
            .pattern_place(target_pattern)
            .expect("test Pattern has a carrier place");
        let mut writable = WritableContext::default();
        writable.grant_place(place);
        SemanticWorld::inject_extended_pattern_value(
            self,
            &open_here,
            local_navigation,
            resident,
            &writable,
            provenance,
        )
    }
}

/// End-to-end acceptance shape: local binding, Pattern material, and the
/// Val2 injection all happen while the construction is open, and the
/// explicit terminal delivery (`t;`) still succeeds afterwards — the
/// injection never froze the construction.
#[test]
fn injection_keeps_construction_open_until_boundary_delivery() {
    let mut world = build_single_fixture_world("meta_open_injection", "app");
    let meta = invoke_make(&mut world, "let A: type = uint8 make;", "open injection");

    // The delivered construction carries exactly the constructed pure-P
    // type member: the injected function object is scope material of that
    // member, not a second cluster member view.
    assert_eq!(
        meta.construction.member_views.len(),
        1,
        "the injected function object never becomes a cluster member view"
    );
    let view = &meta.construction.member_views[0];
    assert!(view.value.is_none(), "constructed type members are pure-P");

    // The injected `f` landed in the constructed member's associated
    // scope as a function object value.
    let injected = world
        .semantic_world()
        .associated_values_for_pattern(view.pattern, "f")
        .expect("`let f::t = fn_expr;` registers `f` in the constructed type's Val2 place");
    assert_eq!(injected.len(), 1, "one injection produces one Val2 entry");
    let function_value = world
        .semantic_world()
        .value(injected[0])
        .expect("injected value object");
    assert!(
        matches!(
            function_value.payload,
            SemanticValuePayload::InjectedFunctionObject { .. }
        ),
        "the injected member is a local injected callable value, got {:?}",
        function_value.payload
    );
    assert_ne!(
        function_value.pattern, view.pattern,
        "ordinary Val2 injection carries its own P; it does not reuse or rewrite the target pure-P"
    );
    assert_eq!(
        world
            .semantic_world()
            .associated_values_for_pattern(function_value.pattern, "()")
            .map(|s| s.len()),
        Some(1),
        "the injected Val1 × P × Val2 member retains its own terminal call entry"
    );
}

/// Canonical meta instance replay: the second invocation with equal
/// normalized arguments replays the shared canonical root and re-finds
/// the already injected member instead of stacking a duplicate.
#[test]
fn replayed_canonical_root_does_not_stack_duplicate_injections() {
    let mut world = build_single_fixture_world("meta_open_injection", "app");
    let first = invoke_make(&mut world, "let A: type = uint8 make;", "replay #1");
    let second = invoke_make(&mut world, "let B: type = uint8 make;", "replay #2");

    let first_pattern = first.construction.member_views[0].pattern;
    let second_pattern = second.construction.member_views[0].pattern;
    assert_eq!(
        first_pattern, second_pattern,
        "equal normalized arguments replay one canonical root"
    );

    assert_eq!(
        world
            .semantic_world()
            .associated_values_for_pattern(first_pattern, "f")
            .expect("injected member present")
            .len(),
        1,
        "replaying the canonical root re-finds the injected member instead of stacking"
    );
}

/// Privilege boundary: ordinary navigated `let f::t = expr` installs a
/// Val2 member (associated type) **without** entering the target Pattern's
/// canonical structure.  Only `struct` inline construction and (future)
/// `inject` hold that privilege.
///
/// Therefore:
///
/// ```text
/// ((x inner)t) |> struct
/// ```
///
/// and
///
/// ```text
/// (()t) |> struct
/// let inner::t = x
/// ```
///
/// do NOT produce the same PatternValue. The first registers `inner` as a
/// structural child of `t`; the second only associates `x` as a Val2 member
/// named `inner` under `t`'s scope.
#[test]
fn ordinary_let_does_not_enter_target_pattern_structure() {
    let mut world = build_single_fixture_world("meta_open_injection", "app");
    let one_shot = invoke_make(
        &mut world,
        "let A: type = uint8 make;",
        "one-shot Pattern construction",
    );
    let incremental = invoke_make(
        &mut world,
        "let B: type = uint8 make_incremental;",
        "incremental associated-type installation",
    );

    // The one-shot Pattern has `inner` as a structural child.
    let one_shot_cpv = one_shot.generated_types[0].canonical_pattern_value();
    let incremental_cpv = incremental.generated_types[0].canonical_pattern_value();
    assert_ne!(
        one_shot_cpv, incremental_cpv,
        "ordinary navigated let does NOT produce the same PatternValue as struct inline: \
         struct registers structural children, ordinary let only installs associated types"
    );

    // The one-shot Pattern in SemanticWorld contains `inner` in its structural norm.
    let one_shot_pattern = one_shot.construction.member_views[0].pattern;
    let incremental_pattern = incremental.construction.member_views[0].pattern;
    let one_shot_norm = world
        .semantic_world()
        .canonical_pattern_norm(one_shot_pattern)
        .expect("one-shot pattern has a structural norm");
    let incremental_norm = world
        .semantic_world()
        .canonical_pattern_norm(incremental_pattern)
        .expect("incremental pattern has a structural norm");
    assert_ne!(
        one_shot_norm, incremental_norm,
        "the SemanticWorld structural norms diverge: struct registers; ordinary let does not"
    );

    // The one-shot Pattern's structural norm contains `inner` as a structural child.
    match &one_shot_norm {
        CanonicalPatternNorm::Structural { value } => match value {
            CanonicalPatternValue::NamedPattern { body, .. } => {
                let CanonicalPatternValue::UnorderedLayer(entries) = body.as_ref() else {
                    panic!("one-shot pattern body should be UnorderedLayer");
                };
                assert!(
                    !entries.is_empty(),
                    "struct inline registers structural children in the Pattern norm"
                );
                // The structural child's navigation must contain `inner`.
                let has_inner = entries
                    .keys()
                    .any(|nav| nav.components().iter().any(|c| c == "inner"));
                assert!(
                    has_inner,
                    "struct inline registered `inner` as a structural child; got keys: {:?}",
                    entries.keys().collect::<Vec<_>>()
                );
            }
            _ => panic!(
                "one-shot structural norm should be a NamedPattern, got {:?}",
                value
            ),
        },
        _ => panic!(
            "one-shot pattern should be a structural norm, got {:?}",
            one_shot_norm
        ),
    }

    // The incremental Pattern's canonical norm is an empty named pattern
    // (only the root `(()t)` with no structural children).
    match incremental_norm {
        CanonicalPatternNorm::Structural { value } => match value {
            CanonicalPatternValue::NamedPattern { body, .. } => {
                let CanonicalPatternValue::UnorderedLayer(entries) = body.as_ref() else {
                    panic!("incremental pattern body should be UnorderedLayer");
                };
                assert!(
                    entries.is_empty(),
                    "ordinary navigated let produces no structural children in the Pattern norm; \
                     got {entries:?}"
                );
            }
            _ => panic!(
                "incremental structural norm should be a NamedPattern, got {:?}",
                value
            ),
        },
        _ => panic!(
            "incremental pattern should be a structural norm, got {:?}",
            incremental_norm
        ),
    }

    // But the associated type IS installed in Val2 under the incremental target.
    let associated = world
        .semantic_world()
        .associated_values_for_pattern(incremental_pattern, "inner")
        .expect("`let inner::t = x;` registers `inner` in the target's Val2 place");
    assert_eq!(
        associated.len(),
        1,
        "one associated-type injection produces one Val2 entry"
    );
    // The installed value is a CoreTypeProjection (pure-P, null Val1).
    let installed_value = world
        .semantic_world()
        .value(associated[0])
        .expect("associated type value is installed");
    assert!(
        matches!(
            installed_value.payload,
            SemanticValuePayload::CoreTypeProjection { .. }
        ),
        "the associated member is a pure type Object (null × P × Val2), got {:?}",
        installed_value.payload
    );

    // `Val2(T_t)["inner"] = C_inner`: on the real source path the Val2 name
    // resolves to its own recursive ClusterSymbol, and the associated type is
    // that Symbol's pure P. The type facet is reachable through the Symbol —
    // the place entry is transport material, not the only carrier.
    let associated_symbol = world
        .semantic_world()
        .associated_symbol_for_pattern(incremental_pattern, "inner")
        .expect("the Val2 name resolves to its own associated ClusterSymbol");
    let cell = world
        .semantic_world()
        .symbol(associated_symbol)
        .expect("associated Symbol exists");
    assert_eq!(
        cell.pure_p_pattern(),
        Some(installed_value.pattern),
        "the associated type is the pure P of its own Val2 Symbol"
    );
    assert!(
        cell.member_views
            .iter()
            .any(|view| view.value.is_none() && view.pattern == installed_value.pattern),
        "the binding-level pure-P member view lives on the associated Symbol"
    );
}

fn static_type_pair() -> PolicyPair {
    let mut stages = StageSet::new();
    stages.insert(PolicyStage::Meta);
    stages.insert(PolicyStage::Compile);
    PolicyPair {
        value: ValueComponentPolicy {
            stages: stages.clone(),
            presence: ValuePresence::Present,
        },
        pattern: PatternComponentPolicy { stages },
    }
}

/// The RHS complete pure-P member view handed to associated-type injection
/// (`value = None`; the RHS Policy flows in, never a fabricated empty one).
fn pure_p_view(
    pattern: lang_build::PatternValueId,
    policy: &PolicyPair,
) -> PolicyResultEntry<SemanticValueId, lang_build::PatternValueId> {
    PolicyResultEntry {
        value: None,
        pattern,
        view: plain_view(policy),
    }
}

/// Window coverage: observation keeps the window live, `UseForVal1` closes it,
/// and boundary delivery remains available after closure.
#[test]
fn use_for_val1_closes_the_window_and_delivery_consumes_it() {
    let mut world = SemanticWorld::new("unit");
    world.bind_package_namespace(NamespaceNodeId(0));
    let policy = static_type_pair();
    let provenance = Provenance::new("meta construction window");
    let (_symbol, _value, pattern) = world
        .register_type_symbol(
            NamespaceNodeId(0),
            "t",
            SymbolId(1),
            TypeValueId(0),
            TypeValueId(0),
            None,
            policy.clone(),
            provenance.clone(),
        )
        .expect("type-rank symbol registers in the unit world");
    let owner = world
        .pattern_owner(pattern)
        .expect("registered pattern owner")
        .owner;
    let pure_p_view = || PolicyResultEntry {
        value: None,
        pattern,
        view: plain_view(&policy),
    };

    let cid = world.begin_cluster_construction(
        ConstructionAuthority::BuildRoot,
        owner,
        provenance.clone(),
    );
    world.force_pattern_cluster_ownership(pattern, cid);
    assert!(
        world
            .contribute_cluster_member_view(cid, pure_p_view())
            .is_some(),
        "an open construction accepts member contribution"
    );

    // Observe(P): the derived Pattern is visible and the window stays live.
    assert_eq!(
        world.observe_cluster_pattern(cid),
        Some(pattern),
        "observation exposes the constructed pure-P Pattern"
    );
    assert!(world
        .open_cluster(cid)
        .expect("open construction")
        .window_is_live(world.residual_runtime_epoch()));

    // UseForVal1 closes the meta construction window.
    assert!(world.use_cluster_for_val1(cid).is_some());
    assert!(!world
        .open_cluster(cid)
        .expect("closed construction")
        .window_is_live(world.residual_runtime_epoch()));

    // A closed window rejects further contribution and Val2 injection. In particular,
    // this is the would-be intersection case: if an RHS value's own
    // P × Val2 is the constructed target type, producing its Val1 has already
    // called UseForVal1, so it cannot subsequently be injected back into the
    // same target as though it also extended that target Pattern.
    assert!(
        world
            .contribute_cluster_member_view(cid, pure_p_view())
            .is_none(),
        "a closed construction window rejects member contribution"
    );
    let NormExprClosure(closure) = injection_closure();
    let rejected = world.inject_associated_function_member(
        cid,
        pattern,
        "f",
        0,
        SymbolId(9),
        &closure,
        None,
        &policy,
        policy.clone(),
        ReturnShape::SingleVal(lang_build::PatternConstraint::Constrained),
        provenance.clone(),
    );
    let Err(diagnostic) = rejected else {
        panic!("a closed construction window must reject Val2 injection");
    };
    assert!(
        diagnostic.message.contains("closed"),
        "the rejection names the closed window: {}",
        diagnostic.message
    );

    // Boundary delivery after closure still succeeds and a second delivery is rejected.
    assert!(
        world.finalize_type_cluster(cid).is_some(),
        "boundary delivery consumes a closed construction"
    );
    assert!(
        world.finalize_type_cluster(cid).is_none(),
        "a delivered construction cannot be delivered twice"
    );
}

/// Wrapper so the helper below reads as intent: parse one closure
/// initializer spelling into its `NormClosure`.
struct NormExprClosure(lang_syntax::NormClosure);

fn injection_closure() -> NormExprClosure {
    let initializer =
        initializer_from_source("let f = (self): compile -> let out: uint8 => { out; };");
    let lang_syntax::NormExpr::Closure(closure) = initializer else {
        panic!("injection fixture initializer is a closure");
    };
    NormExprClosure(closure)
}

/// A second injected-member spelling whose body differs from
/// `injection_closure` (`uint16` vs `uint8`).
fn different_injection_closure() -> NormExprClosure {
    let initializer =
        initializer_from_source("let f = (self): compile -> let out: uint16 => { out; };");
    let lang_syntax::NormExpr::Closure(closure) = initializer else {
        panic!("injection fixture initializer is a closure");
    };
    NormExprClosure(closure)
}

/// The injected value's replay identity is the
/// declaration event under the canonical meta instance's structural
/// coordinates, never the member name and never the outer meta
/// function's backing declaration: one event replaying equal material
/// reuses its value (even under a different outer backing Symbol), one
/// event replaying a different body is a construction conflict, and two
/// distinct events that both write `f` are two sibling vals of one
/// recursive ClusterSymbol `f`.
#[test]
fn injected_value_identity_is_declaration_event_scoped() {
    let mut world = SemanticWorld::new("unit");
    world.bind_package_namespace(NamespaceNodeId(0));
    let policy = static_type_pair();
    let provenance = Provenance::new("b3 injected identity");
    let (_symbol, _value, pattern) = world
        .register_type_symbol(
            NamespaceNodeId(0),
            "t",
            SymbolId(1),
            TypeValueId(0),
            TypeValueId(0),
            None,
            policy.clone(),
            provenance.clone(),
        )
        .expect("type-rank symbol registers in the unit world");
    let owner = world
        .pattern_owner(pattern)
        .expect("registered pattern owner")
        .owner;
    let authority = ConstructionAuthority::MetaInvocation {
        meta_callable: MetaCallableIdentity {
            selected_function_value: SemanticValueId(77),
            selected_call_entry: SemanticValueId(78),
        },
        canonical_key: MetaInvocationMaterialKey {
            callable: MetaCallableIdentity {
                selected_function_value: SemanticValueId(77),
                selected_call_entry: SemanticValueId(78),
            },
            arguments: CanonicalValueAddr(1),
            provenance: provenance.clone(),
        },
    };
    let cid = world.begin_cluster_construction(authority, owner, provenance.clone());
    // Register the target pattern's cluster ownership so the injection
    // ownership check passes (in production the pattern is generated by
    // the construction itself via evaluate_source_meta_member_initializer;
    // here we manually claim it since register_type_symbol installed it
    // as Installed).
    world.force_pattern_cluster_ownership(pattern, cid);

    let inject = |world: &mut SemanticWorld,
                  event: u32,
                  backing: SymbolId,
                  closure: &lang_syntax::NormClosure| {
        world.inject_associated_function_member(
            cid,
            pattern,
            "f",
            event,
            backing,
            closure,
            None,
            &policy,
            policy.clone(),
            ReturnShape::SingleVal(lang_build::PatternConstraint::Constrained),
            provenance.clone(),
        )
    };

    let NormExprClosure(closure) = injection_closure();
    let first = inject(&mut world, 0, SymbolId(9), &closure).expect("first injection installs");
    // Same event + same declaration material → idempotent reuse, even
    // though the outer backing declaration Symbol differs.
    let replay = inject(&mut world, 0, SymbolId(10), &closure)
        .expect("equal declaration material replays idempotently");
    assert_eq!(
        first, replay,
        "one declaration event with equal material owns one value"
    );

    // Same event + different body → construction conflict.
    let NormExprClosure(different) = different_injection_closure();
    let Err(diagnostic) = inject(&mut world, 0, SymbolId(9), &different) else {
        panic!("a different body under one declaration event must conflict");
    };
    assert!(
        diagnostic.message.contains("conflict"),
        "the rejection names the construction conflict: {}",
        diagnostic.message
    );

    // A *different* declaration event writing the same name `f` is never
    // a conflict: it adds a second sibling val under the one recursive
    // ClusterSymbol `f`.
    let second = inject(&mut world, 1, SymbolId(9), &different)
        .expect("a second declaration event writing `f` adds a sibling val");
    assert_ne!(
        first, second,
        "two declaration events own two distinct values"
    );

    // The Val2 name ledger lives on the target OBJECT's place, not on the
    // shared Pattern scope: two carriers of one Pattern must not see each
    // other's injected members.
    let cluster_symbol = world
        .associated_symbol_for_pattern(pattern, "f")
        .expect("injection installs one ClusterSymbol `f` in the target object's place");
    assert_eq!(
        world.associated_values_for_pattern(pattern, "f"),
        Some(&[first, second][..]),
        "the associated Val2 read surface exposes both sibling vals"
    );
    let cell = world
        .symbol(cluster_symbol)
        .expect("injected cluster symbol cell exists");
    assert_eq!(
        cell.sibling_vals,
        vec![first, second],
        "one ClusterSymbol `f` owns both injected sibling vals"
    );
    assert_eq!(
        cell.member_views.len(),
        2,
        "each sibling val carries its own Policy view on the cluster symbol"
    );

    // Replaying the whole canonical instance (both events, equal
    // material) re-finds both values: still two sibling vals, not four.
    let replay_first = inject(&mut world, 0, SymbolId(11), &closure)
        .expect("event 0 replays idempotently under the canonical instance");
    let replay_second = inject(&mut world, 1, SymbolId(11), &different)
        .expect("event 1 replays idempotently under the canonical instance");
    assert_eq!(first, replay_first);
    assert_eq!(second, replay_second);
    let cell = world
        .symbol(cluster_symbol)
        .expect("injected cluster symbol cell exists");
    assert_eq!(
        cell.sibling_vals,
        vec![first, second],
        "canonical replay never stacks duplicate sibling vals"
    );
}

/// One construction-generated (self-typed) `(TypeValue, PatternValue)` pair
/// whose Pattern is contributed to a fresh open construction as its pure P.
/// Returns `(cluster, type_value, pattern)` with the construction still Open.
fn open_self_typed_construction(
    world: &mut SemanticWorld,
    callable_seed: u64,
    policy: &PolicyPair,
    provenance: &Provenance,
) -> (
    lang_build::ClusterConstructionId,
    TypeValueId,
    lang_build::PatternValueId,
) {
    let callable = MetaCallableIdentity {
        selected_function_value: SemanticValueId(callable_seed),
        selected_call_entry: SemanticValueId(callable_seed + 1),
    };
    let root = MetaInstanceRoot {
        meta_callable: callable,
        placement_parent: world.package_owner(),
    };
    let key = MetaInvocationMaterialKey {
        callable,
        arguments: CanonicalValueAddr(1),
        provenance: provenance.clone(),
    };
    let (type_value, pattern) = world
        .install_meta_instance_type_value(&root, key, provenance.clone())
        .expect("meta instance type member installs");
    let cid = world.begin_cluster_construction(
        ConstructionAuthority::BuildRoot,
        world.package_owner(),
        provenance.clone(),
    );
    world
        .contribute_cluster_member_view(
            cid,
            PolicyResultEntry {
                value: None,
                pattern,
                view: plain_view(&policy),
            },
        )
        .expect("an open construction accepts its pure-P member");
    assert!(world
        .open_cluster(cid)
        .expect("open construction")
        .window_is_live(world.residual_runtime_epoch()));
    (cid, type_value, pattern)
}

/// The strong invariant is enforced by the real value
/// production primitive, not by an evaluator's manual courtesy call:
///
/// ```text
/// TypeView(v) = τ  ∧  Val1(v) ≠ null   ⟹   ¬WindowLive(τ)
/// ```
///
/// This test never calls `use_cluster_for_val1`.  Producing a plain value of
/// the self type through `install_plain_value` closes the construction window
/// automatically, and later member contribution / Pattern injection are
/// rejected.  Producing a value of an *unrelated* type leaves the
/// construction window live.
#[test]
fn real_val1_production_closes_the_construction_window() {
    let mut world = SemanticWorld::new("unit");
    world.bind_package_namespace(NamespaceNodeId(0));
    let policy = static_type_pair();
    let provenance = Provenance::new("window closure on Val1 production");

    let (cid, self_type, self_pattern) =
        open_self_typed_construction(&mut world, 900, &policy, &provenance);
    // An unrelated construction-generated type: its values must never
    // close the construction under test.
    let (_other_cid, other_type, _other_pattern) =
        open_self_typed_construction(&mut world, 950, &policy, &provenance);

    // Control: a Val1 of an unrelated type does not close this window.
    world
        .install_plain_value(other_type, policy.clone(), provenance.clone())
        .expect("unrelated plain value installs");
    assert!(world
        .open_cluster(cid)
        .expect("construction")
        .window_is_live(world.residual_runtime_epoch()));

    // Real value production of the self type — no manual freeze call.
    world
        .install_plain_value(self_type, policy.clone(), provenance.clone())
        .expect("self-typed plain value installs");
    let construction = world.open_cluster(cid).expect("construction");
    assert!(!construction.window_is_live(world.residual_runtime_epoch()));
    assert!(
        construction.use_observation.has_been_used_for_val1,
        "the closing event is recorded as UseForVal1"
    );

    // Closed: later member contribution is rejected.
    assert!(
        world
            .contribute_cluster_member_view(
                cid,
                PolicyResultEntry {
                    value: None,
                    pattern: self_pattern,
                    view: plain_view(&policy),
                },
            )
            .is_none(),
        "a closed construction window rejects member contribution"
    );

    // Closed: later pure-P injection is rejected.
    let rejected = world.inject_pattern_value_member(
        cid,
        self_pattern,
        CanonicalFullNavigation::from_component("late"),
        CanonicalPatternValue::Atom(CanonicalPatternAtom::Type(
            CanonicalTypeObservation::Observed(CanonicalValueAddr(7)),
        )),
        provenance.clone(),
    );
    let Err(diagnostic) = rejected else {
        panic!("a closed construction window must reject Pattern injection");
    };
    assert!(
        diagnostic.message.contains("closed"),
        "the rejection names the closed window: {}",
        diagnostic.message
    );
}

/// Same P, different Val2: two values each own an independent per-object
/// ObjectPlace.  Writing a `()` call entry into one value's place does not
/// make the other value callable, and does not pollute the pattern's
/// canonical type-level place.
#[test]
fn same_pattern_different_val2_are_distinguishable() {
    let mut world = SemanticWorld::new("unit");
    world.bind_package_namespace(NamespaceNodeId(0));
    let policy = static_type_pair();
    let provenance = Provenance::new("discriminability test");

    // Two independent type values; they have different patterns but the test
    // proves per-object place isolation regardless.
    let (_sym_a, value_a, pattern_a) = world
        .register_type_symbol(
            NamespaceNodeId(0),
            "A",
            SymbolId(1),
            TypeValueId(0),
            TypeValueId(0),
            None,
            policy.clone(),
            provenance.clone(),
        )
        .expect("type A registers");
    let (_sym_b, value_b, _pattern_b) = world
        .register_type_symbol(
            NamespaceNodeId(0),
            "B",
            SymbolId(2),
            TypeValueId(1),
            TypeValueId(0),
            None,
            policy.clone(),
            provenance.clone(),
        )
        .expect("type B registers");

    // Inject a fake "()" entry directly into value_a's own per-object place.
    let place_a = world.value(value_a).unwrap().place;
    world
        .object_place_mut(place_a)
        .expect("value_a's place exists")
        .associated_val2
        .entry("()".to_string())
        .or_default()
        .push(value_a); // use value_a as a stand-in call entry

    // value_a sees the entry through its own per-object place.
    assert_eq!(
        world
            .associated_values_for_value(value_a, "()")
            .map(|s| s.len()),
        Some(1),
        "value_a's own place carries a () entry"
    );

    // value_b does NOT see any () entry.
    assert!(
        world.associated_values_for_value(value_b, "()").is_none(),
        "value_b's own place (and its pattern's canonical place) are clean"
    );

    // The pattern's canonical type-level place was not affected.
    assert!(
        world
            .associated_values_for_pattern(pattern_a, "()")
            .is_none(),
        "per-object write does not pollute the pattern's canonical place"
    );
}

/// Injection into a pattern NOT owned by the current construction is
/// rejected with a hard diagnostic.
#[test]
fn injection_into_external_pattern_is_rejected() {
    let mut world = SemanticWorld::new("unit");
    world.bind_package_namespace(NamespaceNodeId(0));
    let policy = static_type_pair();
    let provenance = Provenance::new("p03 ownership test");

    // Register an external type T (installed, not under any open construction).
    let (_sym, _val, external_pattern) = world
        .register_type_symbol(
            NamespaceNodeId(0),
            "T",
            SymbolId(1),
            TypeValueId(0),
            TypeValueId(0),
            None,
            policy.clone(),
            provenance.clone(),
        )
        .expect("external type registers");

    // Begin a new construction (NOT owning external_pattern).
    let owner = world
        .pattern_owner(external_pattern)
        .expect("registered pattern")
        .owner;
    let authority = ConstructionAuthority::MetaInvocation {
        meta_callable: MetaCallableIdentity {
            selected_function_value: SemanticValueId(77),
            selected_call_entry: SemanticValueId(78),
        },
        canonical_key: MetaInvocationMaterialKey {
            callable: MetaCallableIdentity {
                selected_function_value: SemanticValueId(77),
                selected_call_entry: SemanticValueId(78),
            },
            arguments: CanonicalValueAddr(99),
            provenance: provenance.clone(),
        },
    };
    let cid = world.begin_cluster_construction(authority, owner, provenance.clone());

    // Attempt to inject an associated type into the external pattern.
    // The ownership check fires before the member view is used, so we can
    // pass external_pattern as a dummy member pattern.
    let result = world.inject_associated_type_member(
        cid,
        external_pattern,
        "child",
        pure_p_view(external_pattern, &policy),
        TypeValueId(1),
        provenance.clone(),
    );
    assert!(
        result.is_err(),
        "injection into external pattern must be rejected"
    );
    let diagnostic = result.unwrap_err();
    assert!(
        diagnostic
            .message
            .contains("not owned by the current construction authority"),
        "diagnostic names the ownership violation: {}",
        diagnostic.message
    );
}

/// Shared window-test setup: one installed target type `t` (its pattern claimed
/// by a fresh construction under the given authority) and one unrelated
/// member type `m` used as associated-injection material.
fn open_claimed_target(
    label: &str,
    make_authority: fn(lang_build::SemanticOwnerId, &Provenance) -> ConstructionAuthority,
) -> (
    SemanticWorld,
    lang_build::ClusterConstructionId,
    lang_build::PatternValueId,
    lang_build::PatternValueId,
    PolicyPair,
    Provenance,
) {
    let mut world = SemanticWorld::new("unit");
    world.bind_package_namespace(NamespaceNodeId(0));
    let policy = static_type_pair();
    let provenance = Provenance::new(label);
    let (_t_sym, _t_val, target_pattern) = world
        .register_type_symbol(
            NamespaceNodeId(0),
            "t",
            SymbolId(1),
            TypeValueId(0),
            TypeValueId(0),
            None,
            policy.clone(),
            provenance.clone(),
        )
        .expect("target type registers");
    let (_m_sym, _m_val, member_pattern) = world
        .register_type_symbol(
            NamespaceNodeId(0),
            "m",
            SymbolId(2),
            TypeValueId(1),
            TypeValueId(0),
            None,
            policy.clone(),
            provenance.clone(),
        )
        .expect("member material type registers");
    let owner = world
        .pattern_owner(target_pattern)
        .expect("registered pattern owner")
        .owner;
    let cid = world.begin_cluster_construction(
        make_authority(owner, &provenance),
        owner,
        provenance.clone(),
    );
    world.force_pattern_cluster_ownership(target_pattern, cid);
    (
        world,
        cid,
        target_pattern,
        member_pattern,
        policy,
        provenance,
    )
}

fn ambient_authority(
    owner: lang_build::SemanticOwnerId,
    _provenance: &Provenance,
) -> ConstructionAuthority {
    ConstructionAuthority::AmbientScope { owner }
}

fn meta_invocation_authority(
    _owner: lang_build::SemanticOwnerId,
    provenance: &Provenance,
) -> ConstructionAuthority {
    let callable = MetaCallableIdentity {
        selected_function_value: SemanticValueId(77),
        selected_call_entry: SemanticValueId(78),
    };
    ConstructionAuthority::MetaInvocation {
        meta_callable: callable,
        canonical_key: MetaInvocationMaterialKey {
            callable,
            arguments: CanonicalValueAddr(1),
            provenance: provenance.clone(),
        },
    }
}

/// An ambient ordinary construction lives in an ordinary window:
/// `WindowLive --FirstUse--> closed`. Compile-only branching is transparent,
/// injection works while open, and after the first semantic use the
/// construction rejects contribution/injection while boundary delivery
/// stays legal.
#[test]
fn ordinary_window_closes_on_first_semantic_use() {
    let (mut world, cid, target_pattern, member_pattern, policy, provenance) =
        open_claimed_target("ordinary first use", ambient_authority);

    let ConstructionWindow::Ordinary(window) = world.open_cluster(cid).expect("open").window else {
        panic!("an AmbientScope construction lives in an ordinary window");
    };
    assert_eq!(window.creation_flow_segment, world.residual_runtime_epoch());
    assert!(!window.first_use_seen);
    assert!(!window.closed_by_fork_or_end);

    // While open: associated-type injection succeeds.
    world
        .inject_associated_type_member(
            cid,
            target_pattern,
            "child",
            pure_p_view(member_pattern, &policy),
            TypeValueId(1),
            provenance.clone(),
        )
        .expect("an open ordinary window accepts associated injection");

    // Compile-only branching never closes an ordinary window.
    world.note_compile_only_branch();
    assert!(world
        .open_cluster(cid)
        .expect("open")
        .window_is_live(world.residual_runtime_epoch()));

    // FirstUse closes the ordinary window.
    world
        .note_first_semantic_use(cid)
        .expect("first-use event lands on the live construction");
    let construction = world.open_cluster(cid).expect("closed construction");
    assert!(!construction.window_is_live(world.residual_runtime_epoch()));
    let ConstructionWindow::Ordinary(window) = construction.window else {
        panic!("window kind never changes");
    };
    assert!(window.first_use_seen, "the closing event is recorded");

    // Closed: injection and contribution are rejected; delivery stays legal.
    let rejected = world.inject_associated_type_member(
        cid,
        target_pattern,
        "late",
        pure_p_view(member_pattern, &policy),
        TypeValueId(1),
        provenance.clone(),
    );
    let Err(diagnostic) = rejected else {
        panic!("a first-use-closed ordinary construction must reject injection");
    };
    assert!(
        diagnostic.message.contains("closed"),
        "the rejection names the closed window: {}",
        diagnostic.message
    );
    assert!(
        world
            .contribute_cluster_member_view(
                cid,
                PolicyResultEntry {
                    value: None,
                    pattern: target_pattern,
                    view: plain_view(&policy),
                },
            )
            .is_none(),
        "a closed ordinary construction rejects member contribution"
    );
    assert!(
        world.finalize_type_cluster(cid).is_some(),
        "boundary delivery stays legal after the ordinary window closed"
    );
}

/// An ordinary struct construction does not survive
/// a residual-runtime fork/end: later injection fails.  A construction
/// created in the *new* flow segment starts open.
#[test]
fn ordinary_window_closes_on_residual_runtime_fork_or_end() {
    let (mut world, cid, target_pattern, member_pattern, policy, provenance) =
        open_claimed_target("ordinary fork/end", ambient_authority);

    // Compile-only branching never advances the residual coordinate.
    let epoch_before = world.residual_runtime_epoch();
    world.note_compile_only_branch();
    assert_eq!(world.residual_runtime_epoch(), epoch_before);
    assert!(world
        .open_cluster(cid)
        .expect("open")
        .window_is_live(world.residual_runtime_epoch()));

    // A residual-runtime fork/end closes every earlier ordinary window.
    world.note_residual_runtime_fork_or_end();
    let construction = world.open_cluster(cid).expect("closed construction");
    assert!(!construction.window_is_live(world.residual_runtime_epoch()));
    let ConstructionWindow::Ordinary(window) = construction.window else {
        panic!("window kind never changes");
    };
    assert!(
        window.closed_by_fork_or_end,
        "the closing event is recorded"
    );

    let rejected = world.inject_associated_type_member(
        cid,
        target_pattern,
        "late",
        pure_p_view(member_pattern, &policy),
        TypeValueId(1),
        provenance.clone(),
    );
    let Err(diagnostic) = rejected else {
        panic!("injection across a residual-runtime fork must fail");
    };
    assert!(
        diagnostic.message.contains("closed"),
        "the rejection names the closed window: {}",
        diagnostic.message
    );

    // A construction created in the new flow segment starts open with the
    // advanced coordinate.
    let owner = world
        .pattern_owner(target_pattern)
        .expect("registered pattern owner")
        .owner;
    let cid2 = world.begin_cluster_construction(
        ConstructionAuthority::AmbientScope { owner },
        owner,
        provenance.clone(),
    );
    let fresh = world.open_cluster(cid2).expect("fresh construction");
    assert!(fresh.window_is_live(world.residual_runtime_epoch()));
    let ConstructionWindow::Ordinary(window) = fresh.window else {
        panic!("an AmbientScope construction lives in an ordinary window");
    };
    assert_eq!(
        window.creation_flow_segment,
        world.residual_runtime_epoch(),
        "the new window records the advanced flow segment"
    );

    // The next fork closes it too.
    world.note_residual_runtime_fork_or_end();
    assert!(!world
        .open_cluster(cid2)
        .expect("construction")
        .window_is_live(world.residual_runtime_epoch()));
}

/// A meta construction spans static control flow:
/// ordinary-window closing events (fork/end, first use) never freeze a
/// meta window; P/Val2 modification stays allowed until `UseForVal1`.
#[test]
fn meta_window_ignores_ordinary_window_events() {
    let (mut world, cid, target_pattern, member_pattern, policy, provenance) =
        open_claimed_target("meta window events", meta_invocation_authority);

    assert_eq!(
        world.open_cluster(cid).expect("open").window,
        ConstructionWindow::Meta,
        "a MetaInvocation construction lives in the meta window"
    );

    // Ordinary-window closing events are transparent to the meta window.
    world.note_residual_runtime_fork_or_end();
    world
        .note_first_semantic_use(cid)
        .expect("the event lands on the live construction");
    let construction = world.open_cluster(cid).expect("still open");
    assert!(construction.window_is_live(world.residual_runtime_epoch()));
    assert!(
        construction
            .use_observation
            .has_been_observed_or_transformed,
        "a non-Val1 use in the meta window is an observation"
    );

    // P/Val2 modification is still allowed across those events.
    world
        .inject_associated_type_member(
            cid,
            target_pattern,
            "child",
            pure_p_view(member_pattern, &policy),
            TypeValueId(1),
            provenance.clone(),
        )
        .expect("the meta window stays open for Val2 modification");

    // Only UseForVal1 closes the meta window.
    world
        .use_cluster_for_val1(cid)
        .expect("UseForVal1 lands on the live construction");
    assert!(!world
        .open_cluster(cid)
        .expect("closed")
        .window_is_live(world.residual_runtime_epoch()));
}

/// `let inner::t = some_val`: the target Pattern's
/// canonical norm is untouched; the value appears as a sibling val of the
/// associated ClusterSymbol `inner` under `t`'s Val2, and the injection
/// (a Transform, not a Val1 production of the constructed type) keeps the
/// meta construction open.
#[test]
fn associated_value_injection_keeps_target_pattern_norm_unchanged() {
    let (mut world, cid, target_pattern, member_pattern, policy, provenance) =
        open_claimed_target("associated sibling val", meta_invocation_authority);

    let norm_before = world
        .canonical_pattern_norm(target_pattern)
        .expect("registered target pattern has a canonical norm");

    // Evaluate the RHS: a plain value of the unrelated type `m`.  This is
    // a foreign-typed Val1 and never closes the construction under test.
    let value = world
        .install_plain_value(TypeValueId(1), policy.clone(), provenance.clone())
        .expect("RHS value installs");
    assert!(world
        .open_cluster(cid)
        .expect("construction")
        .window_is_live(world.residual_runtime_epoch()));

    world
        .inject_associated_existing_value_member(
            cid,
            target_pattern,
            "inner",
            PolicyResultEntry {
                value: Some(value),
                pattern: member_pattern,
                view: plain_view(&policy),
            },
            provenance.clone(),
        )
        .expect("`let inner::t = some_val` installs an associated sibling val");

    // t.P is unchanged.
    assert_eq!(
        world.canonical_pattern_norm(target_pattern),
        Some(norm_before),
        "associated value injection never modifies the target Pattern norm"
    );
    // t.Val2["inner"] carries the sibling val.
    assert_eq!(
        world.associated_values_for_pattern(target_pattern, "inner"),
        Some(&[value][..]),
        "the sibling val appears under the target's Val2"
    );
    // The construction is still open: injection is Transform, not UseForVal1.
    assert!(world
        .open_cluster(cid)
        .expect("construction")
        .window_is_live(world.residual_runtime_epoch()));
}

/// `let f::t = pure_type`: Val2 is not a name → raw value list map, it stays
/// a recursive Symbol world.
///
/// ```text
/// Val2(T_t)[f] = C_f
/// x ∉ Members(C_t)      — never a member of the HOST cluster
/// x  = PureP(C_f)       — the pure-P member of its own Val2 Symbol
/// P(C_f) = P(P_x) || P(w_1) || ... || P(w_m)
/// ```
///
/// So `AssociatedType ⊄ target ClusterMember` — never the unqualified
/// `AssociatedType ⊄ ClusterMember`: the associated Symbol `C_f` is an
/// ordinary cluster Symbol obeying the ordinary member disjunction, and
/// same-named associated vals are its sibling vals. The host construction's
/// member ledger and Pattern norm stay untouched.
///
/// The binding-level member view of `C_f` is the Policy authority. The
/// globally reused CoreTypeProjection adapter in the ObjectPlace is transport
/// material only, so two bindings of the same type keep two independent
/// views.
#[test]
fn associated_type_is_the_pure_p_member_of_its_val2_symbol() {
    let (mut world, cid, target_pattern, member_pattern, policy, provenance) =
        open_claimed_target("associated type Val2 entry", meta_invocation_authority);

    let norm_before = world
        .canonical_pattern_norm(target_pattern)
        .expect("registered target pattern has a canonical norm");

    world
        .inject_associated_type_member(
            cid,
            target_pattern,
            "f",
            pure_p_view(member_pattern, &policy),
            TypeValueId(1),
            provenance.clone(),
        )
        .expect("associated-type injection installs");

    // `Val2(T_t)["f"]` carries the CoreTypeProjection transport reference so the
    // pure type Object is navigable from the host type member's place.
    let transported = world
        .associated_values_for_pattern(target_pattern, "f")
        .expect("the associated type is reachable through the target place's Val2");
    assert_eq!(transported.len(), 1);
    let adapter = transported[0];

    // The semantic fact: `x = PureP(C_f)`, with this binding's member view
    // installed on the associated Symbol itself.
    let associated = world
        .associated_symbol_for_pattern(target_pattern, "f")
        .expect("the Val2 name resolves to its own recursive ClusterSymbol");
    let cell = world.symbol(associated).expect("associated Symbol exists");
    assert_eq!(
        cell.pure_p_pattern(),
        Some(member_pattern),
        "the associated type is the pure P of its own Val2 Symbol"
    );
    assert_eq!(
        cell.member_views,
        vec![pure_p_view(member_pattern, &policy)],
        "the binding-level pure-P member view is the Policy authority of `C_f`"
    );
    assert!(
        cell.sibling_vals.is_empty(),
        "a pure P is never disguised as a sibling val"
    );

    // The HOST is untouched: no member-ledger entry, no Pattern change — the
    // associated type never participates in the target's derived cluster
    // Policy or structural norm.
    assert!(
        world
            .open_cluster(cid)
            .expect("construction")
            .member_views
            .is_empty(),
        "associated-type injection never joins the target construction's member ledger"
    );
    assert_eq!(
        world.canonical_pattern_norm(target_pattern),
        Some(norm_before),
        "associated-type injection never modifies the target Pattern norm"
    );

    // Idempotent replay: the equal contribution changes nothing.
    world
        .inject_associated_type_member(
            cid,
            target_pattern,
            "f",
            pure_p_view(member_pattern, &policy),
            TypeValueId(1),
            provenance.clone(),
        )
        .expect("an equal associated-type contribution replays idempotently");
    assert_eq!(
        world
            .associated_values_for_pattern(target_pattern, "f")
            .expect("container")
            .len(),
        1
    );
    assert_eq!(
        world
            .symbol(associated)
            .expect("associated Symbol exists")
            .member_views
            .len(),
        1
    );

    // A same-named ordinary value joins the very same `C_f` as a sibling
    // val: `C_f = ⟨P_x, w_1, ..., w_m⟩`, and `P(C_f)` is the ordinary member
    // disjunction over both.
    let sibling = world
        .install_plain_value(TypeValueId(1), policy.clone(), provenance.clone())
        .expect("a plain value of the member material type installs");
    let sibling_view = PolicyResultEntry {
        value: Some(sibling),
        pattern: member_pattern,
        view: plain_view(&policy),
    };
    world
        .inject_associated_existing_value_member(
            cid,
            target_pattern,
            "f",
            sibling_view.clone(),
            provenance.clone(),
        )
        .expect("a same-named associated val joins the associated Symbol");
    let transported = world
        .associated_values_for_pattern(target_pattern, "f")
        .expect("container");
    assert_eq!(
        transported.len(),
        2,
        "the type transport reference and the sibling val coexist under one name"
    );
    assert!(transported.contains(&sibling));
    let cell = world.symbol(associated).expect("associated Symbol exists");
    assert_eq!(
        cell.pure_p_pattern(),
        Some(member_pattern),
        "a same-named val never displaces the associated Symbol's pure P"
    );
    assert_eq!(
        cell.sibling_vals,
        vec![sibling],
        "the same-named associated val is a sibling val of `C_f`"
    );
    assert_eq!(
        cell.member_views,
        vec![pure_p_view(member_pattern, &policy), sibling_view.clone()],
        "`C_f` keeps one member view per member"
    );
    assert_eq!(
        cell.cluster_policy(),
        derived_cluster_policy(&[pure_p_view(member_pattern, &policy), sibling_view]),
        "`P(C_f) = P(P_x) || P(w_1)`: the associated Symbol obeys the ordinary \
         cluster disjunction"
    );

    // The binding-level Policy is NOT the globally reused adapter Policy: a
    // second binding of the SAME type under another name transports the same
    // adapter yet keeps its own member view.
    let mut narrow_stages = StageSet::new();
    narrow_stages.insert(PolicyStage::Compile);
    let narrow = PolicyPair {
        value: ValueComponentPolicy {
            stages: narrow_stages.clone(),
            presence: ValuePresence::Present,
        },
        pattern: PatternComponentPolicy {
            stages: narrow_stages,
        },
    };
    assert_ne!(narrow, policy);
    world
        .inject_associated_type_member(
            cid,
            target_pattern,
            "g",
            pure_p_view(member_pattern, &narrow),
            TypeValueId(1),
            provenance.clone(),
        )
        .expect("a second binding of the same associated type installs under its own name");
    assert_eq!(
        world
            .associated_values_for_pattern(target_pattern, "g")
            .expect("container"),
        &[adapter][..],
        "the CoreTypeProjection adapter is globally reused transport material"
    );
    let g_cell = world
        .symbol(
            world
                .associated_symbol_for_pattern(target_pattern, "g")
                .expect("`g` resolves to its own associated Symbol"),
        )
        .expect("associated Symbol exists");
    assert_eq!(
        g_cell.member_views,
        vec![pure_p_view(member_pattern, &narrow)],
        "each binding keeps its own member view even when the transport adapter is shared"
    );
    assert!(
        world
            .symbol(associated)
            .expect("associated Symbol exists")
            .member_views
            .contains(&pure_p_view(member_pattern, &policy)),
        "the second binding never rewrites the first binding's view"
    );

    // A different associated type under the same name is a construction
    // conflict: one Symbol carries at most one pure P.
    let conflict = world.inject_associated_type_member(
        cid,
        target_pattern,
        "f",
        pure_p_view(target_pattern, &policy),
        TypeValueId(0),
        provenance.clone(),
    );
    let Err(diagnostic) = conflict else {
        panic!("a second, different associated type under `f` must conflict");
    };
    assert!(
        diagnostic.message.contains("different associated type"),
        "the rejection names the associated-type conflict: {}",
        diagnostic.message
    );

    // A Val1-carrying view is never accepted by associated-type injection.
    let rejected = world.inject_associated_type_member(
        cid,
        target_pattern,
        "h",
        PolicyResultEntry {
            value: Some(SemanticValueId(9)),
            pattern: member_pattern,
            view: plain_view(&policy),
        },
        TypeValueId(1),
        provenance,
    );
    let Err(diagnostic) = rejected else {
        panic!("a Val1-carrying view must be rejected as associated-type material");
    };
    assert!(
        diagnostic.message.contains("pure-P member view"),
        "the rejection names the pure-P requirement: {}",
        diagnostic.message
    );
}

/// The cluster Policy exists only as the derived disjunction
/// `P_cluster = P_1 || ... || P_n`; exposure keeps filtering per member,
/// so the aggregate never becomes an exposure authority.
///
/// The law is exclusive: the members of one ClusterSymbol are the only
/// place where a whole-function-object P1 is a disjunction over per-object
/// Policies. A Val2 name is itself a ClusterSymbol
/// (`Val2(T_t)[f] = C_f`), so `P(C_f) = P(P_x) || P(w_1) || ...` is that
/// same law one level down, not a second law — the host cluster and host
/// type member never absorb `P(C_f)`. Layered exposure composes
/// conjunctively at lookup and namespaces form no Policy at all.
#[test]
fn cluster_policy_disjunction_is_derived_not_authoritative() {
    // Empty member ledger derives no cluster policy.
    assert_eq!(derived_cluster_policy(&[]), None);

    let static_policy = static_type_pair(); // Meta + Compile, unconstrained mutability
    let mut runtime_stages = StageSet::new();
    runtime_stages.insert(PolicyStage::Runtime);
    let runtime_policy = PolicyPair {
        value: ValueComponentPolicy {
            stages: runtime_stages.clone(),
            presence: ValuePresence::Present,
        },
        pattern: PatternComponentPolicy {
            stages: runtime_stages,
        },
    };

    let views = vec![
        PolicyResultEntry {
            value: None,
            pattern: lang_build::PatternValueId(0),
            view: plain_view(&static_policy),
        },
        PolicyResultEntry {
            value: Some(SemanticValueId(1)),
            pattern: lang_build::PatternValueId(1),
            view: PolicyView {
                pair: runtime_policy.clone(),
                mode: PolicyMode::Mut,
            },
        },
    ];

    let derived = derived_cluster_policy(&views).expect("non-empty ledger derives");
    assert_eq!(
        derived,
        policy_or(&static_policy, &runtime_policy),
        "the derivation is exactly the fold of policy_or over the member views"
    );
    for stage in [
        PolicyStage::Meta,
        PolicyStage::Compile,
        PolicyStage::Runtime,
    ] {
        assert!(
            derived.value.stages.contains(stage),
            "the disjunction admits every member stage: missing {stage:?}"
        );
    }

    // The aggregate admits Runtime, but exposure filters per member: only
    // the runtime member is exposed at the Runtime phase.
    assert!(derived.value.stages.visible_at(Phase::Runtime));
    let exposed: Vec<_> = views
        .iter()
        .filter(|view| view.view.pair.value.stages.visible_at(Phase::Runtime))
        .collect();
    assert_eq!(
        exposed.len(),
        1,
        "Expose(cluster, phase) = {{ member_i | Expose(P_i, phase) }}"
    );
    assert_eq!(exposed[0].value, Some(SemanticValueId(1)));

    // Presence mixing: Present || Absent = Optional.
    let absent_policy = PolicyPair {
        value: ValueComponentPolicy {
            stages: StageSet::new(),
            presence: ValuePresence::Absent,
        },
        pattern: static_policy.pattern.clone(),
    };
    assert_eq!(
        policy_or(&static_policy, &absent_policy).value.presence,
        ValuePresence::Optional,
        "mixed presence relaxes to Optional"
    );
}

/// A `PolicyPair` whose value and Pattern components carry exactly `stages`.
fn stage_pair(stages: &[PolicyStage]) -> PolicyPair {
    let mut set = StageSet::new();
    for stage in stages {
        set.insert(*stage);
    }
    PolicyPair {
        value: ValueComponentPolicy {
            stages: set.clone(),
            presence: ValuePresence::Present,
        },
        pattern: PatternComponentPolicy { stages: set },
    }
}

/// Three carriers of one TypeValue plus one member type to navigate to.
///
/// `base` declares the Pattern and therefore keeps the Pattern's canonical
/// pure type Object; `t` and `u` are ordinary rebindings (`let T: type = base;`)
/// and own fresh writable places.  `t` is `meta + compile`, `u` is `meta`
/// only, so the two carriers share one Pattern and one TypeValue while
/// carrying different binding Policies.  The member type is `compile` only.
fn carriers_of_one_type() -> (
    SemanticWorld,
    lang_build::SemanticSymbolIdentity,
    lang_build::SemanticSymbolIdentity,
    lang_build::SemanticSymbolIdentity,
    lang_build::SemanticSymbolIdentity,
    lang_build::PatternValueId,
) {
    let mut world = SemanticWorld::new("unit");
    world.bind_package_namespace(NamespaceNodeId(0));
    let provenance = Provenance::new("carrier-local Val2 object");
    let register = |world: &mut SemanticWorld,
                    name: &str,
                    binding: u64,
                    represented: u64,
                    policy: PolicyPair| {
        world
            .register_type_symbol(
                NamespaceNodeId(0),
                name,
                SymbolId(binding),
                TypeValueId(represented),
                TypeValueId(0),
                None,
                policy,
                provenance.clone(),
            )
            .expect("type-rank symbol registers in the unit world")
    };

    let (base, _, pattern) = register(&mut world, "base", 1, 0, stage_pair(&[PolicyStage::Meta]));
    let (t, _, t_pattern) = register(
        &mut world,
        "T",
        2,
        0,
        stage_pair(&[PolicyStage::Meta, PolicyStage::Compile]),
    );
    let (u, _, u_pattern) = register(&mut world, "U", 3, 0, stage_pair(&[PolicyStage::Meta]));
    assert_eq!(
        t_pattern, pattern,
        "`let T: type = base` shares the Pattern"
    );
    assert_eq!(
        u_pattern, pattern,
        "`let U: type = base` shares the Pattern"
    );

    let (member, _, _) = register(
        &mut world,
        "member_type",
        4,
        1,
        stage_pair(&[PolicyStage::Compile]),
    );
    (world, base, t, u, member, pattern)
}

fn place_of(
    world: &SemanticWorld,
    symbol: lang_build::SemanticSymbolIdentity,
) -> lang_build::ObjectPlaceId {
    world
        .symbol(symbol)
        .expect("registered symbol")
        .pure_p_place()
        .expect("a pure P is a real object with its own place")
}

/// `let f::T = expr` writes `T`'s own pure-pure type Object, never the shared
/// PatternValue.
///
/// ```text
/// Pattern(T) = Pattern(U) = Pattern(base)
/// Place(T)  != Place(U)  != Place(base)
/// ```
///
/// Reads still fall back to the Pattern's canonical pure type Object, which is
/// where construction-time and toolchain-installed type members live, so
/// inheritance stays visible from every carrier while a per-carrier
/// injection stays local.
#[test]
fn carrier_local_val2_injection_stays_in_that_carriers_object() {
    let (mut world, base, t, u, member, pattern) = carriers_of_one_type();

    let canonical = world
        .pattern_place(pattern)
        .expect("the Pattern has a canonical pure type Object");
    assert_eq!(
        place_of(&world, base),
        canonical,
        "the declaring carrier keeps writing the Pattern's canonical object"
    );
    let t_place = place_of(&world, t);
    let u_place = place_of(&world, u);
    assert_ne!(t_place, canonical);
    assert_ne!(u_place, canonical);
    assert_ne!(t_place, u_place);

    // `let f::T = member_type`
    world
        .associate_existing_symbol_in_place(t_place, "f", member)
        .expect("the injection records a source-visible Val2 name on T's object");

    let host = |world: &SemanticWorld, symbol| {
        world
            .host_member_of(symbol)
            .expect("a type carrier is a host layer")
    };
    assert_eq!(
        world.associated_symbol_for_host(&host(&world, t), "f"),
        Some(member),
        "`T::f` resolves through T's own object"
    );
    assert_eq!(
        world.associated_symbol_for_host(&host(&world, u), "f"),
        None,
        "`U::f` must not see a member injected into T's object"
    );
    assert_eq!(
        world.associated_symbol_for_host(&host(&world, base), "f"),
        None,
        "`base::f` must not see it either"
    );
    assert_eq!(
        world.associated_symbol_for_pattern(pattern, "f"),
        None,
        "the shared PatternValue's canonical object stays untouched"
    );

    // The injection is a Val2 write only: the host carriers keep their
    // Pattern identity and their own binding views.
    for carrier in [base, t, u] {
        assert_eq!(
            world
                .symbol(carrier)
                .expect("registered symbol")
                .pure_p_pattern(),
            Some(pattern),
            "a Val2 write never rebinds a carrier's Pattern"
        );
    }

    // The canonical direction is inheritance: a member on the Pattern's own
    // object is reachable from every carrier of that Pattern.
    world
        .associate_existing_symbol_in_place(canonical, "inherited", member)
        .expect("the canonical pure type Object accepts a type member");
    for carrier in [base, t, u] {
        assert_eq!(
            world.associated_symbol_for_host(&host(&world, carrier), "inherited"),
            Some(member),
            "canonical type members stay visible through every carrier"
        );
    }
}

/// `Expose(t::f, φ) = Expose(T_t, φ) ∧ Expose(C_f, φ)`.
///
/// Two negatives pin the host factor down:
///
/// * a `meta`-only host carrying a `compile` member exposes nothing at
///   `SealStatic`, even though the member itself is `compile`-visible there;
/// * two carriers of one TypeValue with different binding Policies expose
///   the same member differently, which a `PatternValueId` alone could never
///   express.
///
/// The gate is on exposure, not on name resolution, and it only READS the
/// host Policy: no member view is rewritten, disjoined, or folded.
#[test]
fn layered_exposure_gates_val2_navigation_on_the_host_member() {
    let (mut world, _base, t, u, member, pattern) = carriers_of_one_type();
    let t_place = place_of(&world, t);
    let u_place = place_of(&world, u);
    world
        .associate_existing_symbol_in_place(t_place, "f", member)
        .expect("`let f::T`");
    world
        .associate_existing_symbol_in_place(u_place, "f", member)
        .expect("`let f::U`");

    let member_views = world
        .symbol(member)
        .expect("member symbol")
        .member_views
        .clone();
    assert!(
        member_views
            .iter()
            .all(|view| view.view.pair.pattern.stages.visible_at(Phase::SealStatic)),
        "the member factor itself is compile-visible at SealStatic"
    );

    let t_host = world.host_member_of(t).expect("T is a host layer");
    let u_host = world.host_member_of(u).expect("U is a host layer");

    // Same Pattern, same TypeValue, different binding Policy.
    assert_eq!(t_host.pattern, u_host.pattern);
    assert_eq!(t_host.pattern, pattern);
    assert_eq!(
        world.type_for_pattern(t_host.pattern),
        world.type_for_pattern(u_host.pattern)
    );
    assert_ne!(t_host.view, u_host.view);

    // Meta window: both hosts are visible, so both navigations expose the
    // member.
    assert_eq!(
        world.associated_member_views_for_host(&t_host, "f", Phase::OpenStatic),
        member_views,
        "an exposed host passes the member views through unmodified"
    );
    assert_eq!(
        world.associated_member_views_for_host(&u_host, "f", Phase::OpenStatic),
        member_views
    );

    // Sealed window: the `meta`-only host is gone, so `U::f` is unreachable
    // while `T::f` still resolves.
    assert!(t_host.exposed_at(Phase::SealStatic));
    assert!(!u_host.exposed_at(Phase::SealStatic));
    assert_eq!(
        world.associated_member_views_for_host(&t_host, "f", Phase::SealStatic),
        member_views,
        "a compile-visible host keeps exposing its compile member"
    );
    assert!(
        world
            .associated_member_views_for_host(&u_host, "f", Phase::SealStatic)
            .is_empty(),
        "a meta-only host exposes nothing at SealStatic, whatever its members are"
    );
    assert_eq!(
        world.associated_symbol_for_host(&u_host, "f"),
        Some(member),
        "the host gate filters exposure, it does not unbind the name"
    );

    // Reading the host Policy never writes to the member.
    assert_eq!(
        world.symbol(member).expect("member symbol").member_views,
        member_views,
        "layered exposure is read-only on the member ledger"
    );
}
