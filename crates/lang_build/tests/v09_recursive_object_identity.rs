//! Type-object identity is the RECURSIVE object normal form, and every
//! use context shares ONE Val2 Symbol navigator.
//!
//! A type object is `null × P × Val2`, so its normal form must carry both
//! components:
//!
//! ```text
//! Norm_type(x)    = ⟨ Norm_P(P_x), Norm_Val2(Val2_x) ⟩
//! Norm_Val2(V)    = Map_name( Norm_Cluster(V[name]) )
//! Norm_Cluster(C) = ⟨ Norm_pureP(C.pureP)?, Multiset{ Norm_val(v) } ⟩
//! Norm_pureP(x)   = ⟨ Norm_P(P_x), Norm_Val2(Val2_x) ⟩
//! ```
//!
//! `ObjectPlaceId ∉ Norm_type`: a place is only the observation coordinate
//! `place(x) ↦ Val2_x`, so
//!
//! ```text
//! P_x = P_y ∧ Norm_Val2(Val2_x) = Norm_Val2(Val2_y) ⇒ Norm_type(x) = Norm_type(y)
//! P_x = P_y ∧ Norm_Val2(Val2_x) ≠ Norm_Val2(Val2_y) ⇒ Norm_type(x) ≠ Norm_type(y)
//! ```
//!
//! even when `place(x) ≠ place(y)`.  The recursion is well-founded finite
//! recursion: it descends into each associated cluster Symbol and bottoms out
//! at leaves with no vertically traversable object children
//! (`Children_V(x) = ∅`, e.g. `Val2(()) = ∅`).  Re-entering an object still
//! on the ACTIVE recursion stack proves an illegal cyclic Val2 and is a hard
//! semantic error — a cycle has no normal form — while shared acyclic
//! subtrees (a diamond) reuse their FINISHED normal forms.  No
//! `SemanticValueId`, `ObjectPlaceId`, or memo node number reaches the
//! normal form.
//!
//! Navigation is Symbol-first and context-independent:
//!
//! ```text
//! Path -> Symbol -> ContextDirectedProjection
//! ```
//!
//! Which Symbol a path denotes is NOT decided by whether the result is later
//! used as a call target, a type, a value, or an injection target; only the
//! final facet projection differs.

mod support;

use lang_build::{
    classify_type_arguments_env_with_report, compute_meta_invocation_material_key,
    extract_single_call_site, invoke_host_member_symbol_ordinary, CanonicalValueAddr,
    MetaCallableIdentity, NamespaceNodeId, NonValueArgKind, ObjectPlaceId,
    OrdinaryInvocationContext, OrdinaryInvocationFailure, PatternComponentPolicy, PatternValueId,
    Phase, PolicyMode, PolicyPair, PolicyStage, ProductAtom, ProductMaterialRole, Provenance,
    RawArgShape, RawArgValueClass, ResolverContext, ReturnShape, SemanticSymbolIdentity,
    SemanticTypeEnv, SemanticValueId, SemanticWorld, StageSet, StructMaterializationState,
    SymbolId, TypeMemberFacet, TypeResolutionEnv, TypeValueId, ValueComponentPolicy, ValuePresence,
};
use support::initializer_from_source;

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

struct Carriers {
    world: SemanticWorld,
    /// Declares the shared Pattern, so it keeps the Pattern's canonical place.
    base: SemanticSymbolIdentity,
    /// `let T: type = base;` — same Pattern and TypeValue, own fresh place.
    t: SemanticSymbolIdentity,
    /// `let U: type = base;` — likewise.
    u: SemanticSymbolIdentity,
    /// `let V: type = base;` — likewise.
    v: SemanticSymbolIdentity,
    /// A second type, used as the injected Val2 member.
    member: SemanticSymbolIdentity,
    /// A third type, used to make one carrier's Val2 differ by content.
    other: SemanticSymbolIdentity,
    pattern: PatternValueId,
    type_value: TypeValueId,
}

/// Three carriers of one TypeValue plus two member types to inject.
fn carriers() -> Carriers {
    let mut world = SemanticWorld::new("unit");
    world.bind_package_namespace(NamespaceNodeId(0));
    let provenance = Provenance::new("recursive type identity");
    let register = |world: &mut SemanticWorld, name: &str, binding: u64, represented: u64| {
        world
            .register_type_symbol(
                NamespaceNodeId(0),
                name,
                SymbolId(binding),
                TypeValueId(represented),
                TypeValueId(0),
                None,
                stage_pair(&[PolicyStage::Meta, PolicyStage::Compile]),
                provenance.clone(),
            )
            .expect("type-rank symbol registers in the unit world")
    };

    let (base, _, pattern) = register(&mut world, "base", 1, 0);
    let (t, _, t_pattern) = register(&mut world, "T", 2, 0);
    let (u, _, u_pattern) = register(&mut world, "U", 3, 0);
    let (v, _, v_pattern) = register(&mut world, "V", 6, 0);
    assert_eq!(
        t_pattern, pattern,
        "`let T: type = base` shares the Pattern"
    );
    assert_eq!(
        u_pattern, pattern,
        "`let U: type = base` shares the Pattern"
    );
    assert_eq!(
        v_pattern, pattern,
        "`let V: type = base` shares the Pattern"
    );
    let (member, _, _) = register(&mut world, "member_type", 4, 1);
    let (other, _, _) = register(&mut world, "other_type", 5, 2);

    Carriers {
        world,
        base,
        t,
        u,
        v,
        member,
        other,
        pattern,
        type_value: TypeValueId(0),
    }
}

fn place_of(world: &SemanticWorld, symbol: SemanticSymbolIdentity) -> ObjectPlaceId {
    world
        .symbol(symbol)
        .expect("registered symbol")
        .pure_p_place()
        .expect("a pure P is a real object with its own place")
}

/// One type argument as the call pipeline would hand it over: the resolved
/// TypeValue plus the resolving carrier's own observation place.
fn type_arg(type_value: TypeValueId, place: Option<ObjectPlaceId>) -> (RawArgShape, ProductAtom) {
    let atom = ProductAtom::Unsupported {
        summary: "type argument under test".to_string(),
        provenance: Provenance::new("recursive type identity"),
    };
    let raw = RawArgShape::from_product_atom(0, &atom).as_type_object_named(
        "T".to_string(),
        type_value,
        None,
        None,
        place,
        None,
    );
    (raw, atom)
}

/// `Norm_type` of the type object observed from `place`.
fn type_addr(
    world: &mut SemanticWorld,
    type_value: TypeValueId,
    place: Option<ObjectPlaceId>,
) -> CanonicalValueAddr {
    try_type_addr(world, type_value, place).expect("acyclic Val2 normalizes")
}

/// `Norm_type`, keeping the cyclic-Val2 rejection observable.
fn try_type_addr(
    world: &mut SemanticWorld,
    type_value: TypeValueId,
    place: Option<ObjectPlaceId>,
) -> Result<CanonicalValueAddr, lang_build::Diagnostic> {
    let (raw, atom) = type_arg(type_value, place);
    world.canonical_argument_address(&raw, &atom)
}

/// Same `P`, different places, equal recursive Val2 ⇒ ONE normal form;
/// unequal recursive Val2 ⇒ different normal forms.
///
/// This is the whole `place ∉ identity` claim in one test: the addresses
/// separate exactly when the observed Val2 content separates, and they merge
/// again as soon as the content matches — across three carriers that never
/// share a place.
#[test]
fn type_identity_ignores_place_but_follows_recursive_val2() {
    let Carriers {
        mut world,
        base,
        t,
        u,
        v,
        member,
        other,
        pattern,
        type_value,
    } = carriers();

    let canonical = world
        .pattern_place(pattern)
        .expect("the Pattern has a canonical type object");
    let base_place = place_of(&world, base);
    let t_place = place_of(&world, t);
    let u_place = place_of(&world, u);
    let v_place = place_of(&world, v);
    assert_eq!(base_place, canonical, "the declaring carrier owns it");
    assert_ne!(t_place, canonical);
    assert_ne!(u_place, canonical);
    assert_ne!(t_place, u_place);
    assert_ne!(t_place, v_place);

    // Empty Val2 everywhere: three distinct places, one normal form.
    let base_addr = type_addr(&mut world, type_value, Some(base_place));
    let t_addr = type_addr(&mut world, type_value, Some(t_place));
    let u_addr = type_addr(&mut world, type_value, Some(u_place));
    assert_eq!(base_addr, t_addr, "Place(base) != Place(T) is not identity");
    assert_eq!(t_addr, u_addr, "Place(T) != Place(U) is not identity");

    // `let f::T = member_type` — T's observed Val2 now differs.
    world
        .associate_existing_symbol_in_place(t_place, "f", member)
        .expect("the injection records a Val2 name on T's own object");
    let t_with_f = type_addr(&mut world, type_value, Some(t_place));
    assert_ne!(
        t_with_f, t_addr,
        "a new Val2 member changes the type object's normal form"
    );
    assert_eq!(
        type_addr(&mut world, type_value, Some(u_place)),
        u_addr,
        "U's own object never saw the injection"
    );

    // `let f::U = other_type` — same NAME, different member content.
    world
        .associate_existing_symbol_in_place(u_place, "f", other)
        .expect("the injection records a Val2 name on U's own object");
    assert_ne!(
        type_addr(&mut world, type_value, Some(u_place)),
        t_with_f,
        "Norm_Val2 descends into the member Symbol: one shared name is not one Val2"
    );

    // `let f::V = member_type` — a third place with T's exact Val2 content:
    // the normal forms collapse back onto one address.
    world
        .associate_existing_symbol_in_place(v_place, "f", member)
        .expect("the injection records a Val2 name on V's own object");
    assert_eq!(
        type_addr(&mut world, type_value, Some(v_place)),
        t_with_f,
        "equal P and equal recursive Val2 is equal identity, whatever the place"
    );
}

/// Lookup-visible fallback is horizontal navigation, not owned Val2.  A
/// carrier freezes physically shared members when it is formed; later writes
/// to the Pattern's canonical place remain navigable but cannot rewrite that
/// existing Object's normal form.
#[test]
fn later_pattern_fallback_does_not_rewrite_existing_owned_val2() {
    let Carriers {
        mut world,
        base,
        t,
        member,
        type_value,
        ..
    } = carriers();
    let canonical = place_of(&world, base);
    let t_place = place_of(&world, t);
    let t_host = world.host_member_of(t).expect("T is a pure-P host");
    let before = type_addr(&mut world, type_value, Some(t_place));

    world
        .associate_existing_symbol_in_place(canonical, "late", member)
        .expect("the canonical Pattern object receives a later member");
    assert_eq!(
        world.associated_symbol_for_host(&t_host, "late"),
        Some(member),
        "navigation may still expose the Pattern fallback"
    );
    assert_eq!(
        type_addr(&mut world, type_value, Some(t_place)),
        before,
        "lookup fallback is not SemanticVal2Snapshot(T)"
    );
    assert_ne!(
        type_addr(&mut world, type_value, Some(canonical)),
        before,
        "the canonical object's own snapshot did change"
    );
}

/// The complete type callspace snapshot and an Object's owned Val2 snapshot
/// are distinct semantic observations.  A new direct TypeMember produces a
/// successor tau without rewriting the existing Object's Val2 definition.
#[test]
fn successor_vtau_does_not_redefine_object_val2() {
    let Carriers {
        mut world,
        t,
        pattern,
        type_value,
        ..
    } = carriers();
    let t_place = place_of(&world, t);
    let object_before = world
        .canonical_type_core_observation_address(type_value, Some(t_place))
        .expect("initial Object core observes");
    let tau_before = world
        .observe_complete_type(type_value, Some(t_place))
        .expect("initial complete tau observes")
        .whole;
    let builtin_member = world
        .type_object_value(TypeValueId(1))
        .expect("member type has a transport value");

    world
        .admit_direct_type_member(
            pattern,
            pattern,
            "vtau_only",
            TypeMemberFacet::Value,
            builtin_member,
        )
        .expect("a fresh direct TypeMember is admitted");
    let tau_after = world
        .observe_complete_type(type_value, Some(t_place))
        .expect("successor complete tau observes")
        .whole;

    assert_ne!(
        tau_before, tau_after,
        "V_tau changed the whole tau snapshot"
    );
    assert_eq!(
        object_before,
        world
            .canonical_type_core_observation_address(type_value, Some(t_place))
            .expect("Object core re-observes"),
        "V_tau is not SemanticVal2Snapshot(x)"
    );
}

/// One open type object observed BEFORE and AFTER a Val2 injection produces
/// two normal forms and therefore two meta instance keys.
///
/// ```text
/// t_1 = ⟨P_t, {f}⟩         MetaKey(meta_fn, t_1)
/// t_2 = ⟨P_t, {f, g}⟩      MetaKey(meta_fn, t_2)   ≠
/// ```
///
/// The observation coordinate is unchanged across both observations, so the
/// only thing that can separate the keys is the recursive Val2 content.
#[test]
fn open_type_object_observed_before_and_after_injection_changes_its_meta_key() {
    let Carriers {
        mut world,
        t,
        member,
        other,
        type_value,
        ..
    } = carriers();
    let t_place = place_of(&world, t);
    let meta_fn = MetaCallableIdentity {
        selected_function_value: SemanticValueId(7),
        selected_call_entry: SemanticValueId(70),
    };
    let provenance = Provenance::new("open construction meta key");

    let key_of = |world: &mut SemanticWorld| {
        let (raw, atom) = type_arg(type_value, Some(t_place));
        let args = world
            .canonical_arguments_product_address(&[raw], &[atom])
            .expect("acyclic Val2 normalizes");
        compute_meta_invocation_material_key(meta_fn, args, provenance.clone())
    };

    // let f::t = X;  let A = t |> meta_fn;
    world
        .associate_existing_symbol_in_place(t_place, "f", member)
        .expect("`let f::t = X`");
    let first_addr = type_addr(&mut world, type_value, Some(t_place));
    let first_key = key_of(&mut world);

    // Observing the same open object twice without any write is stable.
    assert_eq!(
        type_addr(&mut world, type_value, Some(t_place)),
        first_addr,
        "normalization is a pure read of the current object"
    );
    assert_eq!(key_of(&mut world), first_key);

    // let g::t = Y;  let B = t |> meta_fn;  — SAME callable, so only Val2
    // can separate the keys.
    world
        .associate_existing_symbol_in_place(t_place, "g", other)
        .expect("`let g::t = Y`");
    let second_addr = type_addr(&mut world, type_value, Some(t_place));
    assert_ne!(
        first_addr, second_addr,
        "⟨P_t, {{f}}⟩ and ⟨P_t, {{f, g}}⟩ are different type objects"
    );
    assert_ne!(
        first_key,
        key_of(&mut world),
        "MetaKey(meta_fn, t_1) != MetaKey(meta_fn, t_2)"
    );
}

/// `()` is the standard leaf of the Val2 recursion: `Val2(()) = ∅`, so
/// `Norm(()) = ⟨Norm_P(P_FunctionItem), ∅⟩` terminates without descending
/// further — the call entry is terminal, not a hop toward another callable.
#[test]
fn unit_is_terminal_leaf() {
    let Carriers { mut world, .. } = carriers();
    let provenance = Provenance::new("unit leaf");
    let initializer =
        initializer_from_source("let f = (self): compile -> let out: uint8 => { out; };");
    let lang_syntax::NormExpr::Closure(closure) = initializer else {
        panic!("callable fixture initializer is a closure");
    };
    let registered = world
        .register_source_callable(
            NamespaceNodeId(0),
            "f",
            SymbolId(90),
            &closure,
            None,
            lang_build::PolicyView {
                pair: stage_pair(&[PolicyStage::Meta, PolicyStage::Compile]),
                mode: PolicyMode::Plain,
            },
            lang_build::PolicyView {
                pair: stage_pair(&[PolicyStage::Meta, PolicyStage::Compile]),
                mode: PolicyMode::Plain,
            },
            None,
            ReturnShape::SingleVal(lang_build::PatternConstraint::Constrained),
            provenance.clone(),
        )
        .expect("source callable registers in the unit world");
    let (entry_pattern, entry_type) = {
        let entry = world
            .value(registered.call_entry)
            .expect("the callable owns a terminal call entry");
        (entry.pattern, entry.type_value)
    };
    assert!(
        world
            .associated_values_for_pattern(entry_pattern, "()")
            .is_none(),
        "Val2(()) = ∅: the call entry has no vertically traversable children"
    );

    let atom = ProductAtom::SemanticValue {
        value: registered.call_entry,
        type_value: entry_type,
        mode: PolicyMode::Const,
        provenance: provenance.clone(),
    };
    let raw = RawArgShape::from_product_atom(0, &atom);
    let addr = world
        .canonical_argument_address(&raw, &atom)
        .expect("`()` is a leaf: the recursion terminates here");
    let replay = world
        .canonical_argument_address(&raw, &atom)
        .expect("a terminal leaf replays");
    assert_eq!(
        addr, replay,
        "Norm(()) = ⟨Norm_P(P_FunctionItem), ∅⟩ is a stable normal form"
    );
}

/// `let loop::U = U;` — the walk re-enters the object it is still
/// normalizing.  Val2 normalization is well-founded finite recursion, so
/// this is NOT a normal form (no back edge): it is a hard semantic error.
#[test]
fn self_cycle_val2_is_rejected() {
    let Carriers {
        mut world,
        u,
        type_value,
        ..
    } = carriers();
    let u_place = place_of(&world, u);

    // Before the cycle exists, the same object normalizes.
    let acyclic = try_type_addr(&mut world, type_value, Some(u_place))
        .expect("an empty-Val2 object normalizes");

    world
        .associate_existing_symbol_in_place(u_place, "loop", u)
        .expect("`let loop::U = U`");
    let rejected = try_type_addr(&mut world, type_value, Some(u_place))
        .expect_err("an active-stack re-entry is an illegal cyclic Val2");
    assert!(
        rejected.message.contains("cyclic Val2"),
        "the rejection names the cyclic Val2: {rejected:?}"
    );
    let _ = acyclic;
}

/// `A → B → A` through two DISTINCT objects is still an active-stack
/// re-entry, observed from either end of the cycle.
#[test]
fn mutual_cycle_val2_is_rejected() {
    let Carriers {
        mut world,
        t,
        u,
        type_value,
        ..
    } = carriers();
    let t_place = place_of(&world, t);
    let u_place = place_of(&world, u);

    // `let a::T = U;` and `let b::U = T;` — no object refers to itself, but
    // the traversal T → U → T re-enters T while T is still open.
    world
        .associate_existing_symbol_in_place(t_place, "a", u)
        .expect("`let a::T = U`");
    world
        .associate_existing_symbol_in_place(u_place, "b", t)
        .expect("`let b::U = T`");

    for place in [t_place, u_place] {
        let rejected = try_type_addr(&mut world, type_value, Some(place))
            .expect_err("a mutual cycle has no normal form from either entry point");
        assert!(
            rejected.message.contains("cyclic Val2"),
            "the rejection names the cyclic Val2: {rejected:?}"
        );
    }
}

/// A diamond `A → {B, C}`, `B → D`, `C → D` shares the leaf D without any
/// cycle: the second visit reuses D's FINISHED normal form instead of being
/// mistaken for an active-stack re-entry.
#[test]
fn shared_acyclic_subtree_is_allowed() {
    let Carriers {
        mut world,
        t,
        u,
        v,
        member,
        type_value,
        ..
    } = carriers();
    let t_place = place_of(&world, t);
    let u_place = place_of(&world, u);
    let v_place = place_of(&world, v);
    let member_place = place_of(&world, member);

    // A = T, B = U, C = V, D = member_type.
    world
        .associate_existing_symbol_in_place(t_place, "b", u)
        .expect("`let b::T = U`");
    world
        .associate_existing_symbol_in_place(t_place, "c", v)
        .expect("`let c::T = V`");
    world
        .associate_existing_symbol_in_place(u_place, "d", member)
        .expect("`let d::U = member_type`");
    world
        .associate_existing_symbol_in_place(v_place, "d", member)
        .expect("`let d::V = member_type`");

    // ONE walk visits D twice (via B and via C) and still succeeds: sharing
    // is DAG reuse, not a cycle.
    let diamond = try_type_addr(&mut world, type_value, Some(t_place))
        .expect("a shared acyclic subtree is not a cycle");
    assert_eq!(
        type_addr(&mut world, type_value, Some(t_place)),
        diamond,
        "the diamond normal form is stable across walks"
    );

    // The leaf D itself terminates with `Val2 = ∅` and is not the diamond.
    let leaf = type_addr(&mut world, TypeValueId(1), Some(member_place));
    assert_ne!(diamond, leaf, "the diamond is not collapsed onto its leaf");
}

/// `f::T` denotes ONE terminal Symbol, and every use context only projects a
/// different facet of it afterwards.
///
/// Covered contexts: call target / injection RHS (`(…) |> f::T`,
/// `let g::U = f::T`), extraction completion (`g::f::T`), the type context
/// (`let A: type = f::T`), and the meta-argument context
/// (`let B = (f::T) meta_fn`) — where the classified argument must also
/// normalize through the terminal Symbol's OWN place, tying this back to
/// `Norm_type`.
#[test]
fn navigated_path_reaches_one_terminal_symbol_in_every_context() {
    let Carriers {
        mut world,
        t,
        u,
        member,
        pattern,
        ..
    } = carriers();
    let t_place = place_of(&world, t);
    let member_place = place_of(&world, member);
    let member_pattern = world
        .symbol(member)
        .expect("member symbol")
        .pure_p_pattern()
        .expect("the member is a pure P");
    let member_type = world
        .type_for_pattern(member_pattern)
        .expect("the member type has a TypeValue");
    world
        .associate_existing_symbol_in_place(t_place, "f", member)
        .expect("`let f::T = member_type`");

    let path = |components: &[&str]| -> Vec<String> {
        components.iter().map(|c| (*c).to_string()).collect()
    };
    let f_t = path(&["f", "T"]);

    // The shared navigator: `f::T` means `Val2(T)[f]`, stepping through T's
    // own object place.
    let navigation = world
        .navigate_semantic_path(&f_t, NamespaceNodeId(0), &[], &[])
        .expect("`f::T` navigates T's Val2");
    assert_eq!(
        navigation.terminal_symbol, member,
        "the path resolves to the associated cluster Symbol"
    );
    assert_eq!(
        navigation
            .terminal_host()
            .expect("`f::T` stepped through a host")
            .pattern,
        pattern,
        "the host layer is T's pure-P type object"
    );

    // Call target and injection RHS: `(…) |> f::T`, `let g::U = f::T`.
    assert_eq!(
        world
            .resolve_symbol_path(&f_t, NamespaceNodeId(0), &[], &[])
            .expect("call/injection path resolution"),
        member,
    );
    // Extraction completion: `g::f::T` resolves its `f::T` prefix the same way.
    assert_eq!(
        world.resolve_symbol_path_exact(&f_t, NamespaceNodeId(0)),
        Some(member),
    );

    // Type context: `let A: type = f::T` projects the pure-P facet of the
    // same terminal Symbol, carrying that Symbol's own place.
    let context = ResolverContext::new(NamespaceNodeId(0));
    let resolution = SemanticTypeEnv::new(&world)
        .resolve_type_path(&f_t, &context)
        .expect("the type context resolves `f::T`");
    assert_eq!(resolution.represented_type, member_type);
    assert_eq!(
        resolution.carrier_place,
        Some(member_place),
        "the type facet is observed from the terminal Symbol's own object"
    );

    // Meta-argument context: `let B = (f::T) meta_fn`.
    let initializer = initializer_from_source("let B = (f::T) meta_fn;");
    let call_site = extract_single_call_site(&initializer).expect("normalized call");
    let shape =
        call_site.to_arg_product_shape(ProductMaterialRole::MetaConstructionArgumentProduct);
    let report =
        classify_type_arguments_env_with_report(&shape, &SemanticTypeEnv::new(&world), &context);
    assert!(
        report.unresolved_names.is_empty(),
        "a navigated type argument is not an unresolved name: {:?}",
        report.unresolved_names
    );
    let classified = report
        .classified_shape
        .raw_args
        .iter()
        .find(|raw| {
            matches!(
                raw.value_class,
                RawArgValueClass::NonValue(NonValueArgKind::CoreTypeProjection)
            )
        })
        .expect("`f::T` classifies as a type object argument")
        .clone();
    assert_eq!(classified.known_first_order_type_value, Some(member_type));
    assert_eq!(
        classified.known_type_carrier_place,
        Some(member_place),
        "the meta-argument context carries the terminal Symbol's own place"
    );
    let atom = report
        .classified_shape
        .flattened
        .atoms
        .get(classified.index)
        .expect("the classified argument keeps its atom")
        .clone();
    let via_call = world
        .canonical_argument_address(&classified, &atom)
        .expect("acyclic Val2 normalizes");
    let via_symbol = type_addr(&mut world, member_type, Some(member_place));
    assert_eq!(
        via_call, via_symbol,
        "the navigated argument normalizes through Norm_type of the very object it named"
    );

    // Negative symmetry: `f::U` is unresolved in EVERY context, because the
    // Symbol step fails once, not once per context.
    let f_u = path(&["f", "U"]);
    assert!(world
        .navigate_semantic_path(&f_u, NamespaceNodeId(0), &[], &[])
        .is_err());
    assert!(world
        .resolve_symbol_path(&f_u, NamespaceNodeId(0), &[], &[])
        .is_err());
    assert_eq!(
        world.resolve_symbol_path_exact(&f_u, NamespaceNodeId(0)),
        None
    );
    assert!(SemanticTypeEnv::new(&world)
        .resolve_type_path(&f_u, &context)
        .is_none());
    let _ = u;
}

/// The recursive Symbol navigator returns the WHOLE host chain of a
/// multi-layer path, and an ordinary call must compose the exposure
/// conjunction over EVERY host it stepped through — not just the innermost
/// one.
///
/// ```text
/// Expose(g::f::T, φ) = Expose(T, φ) ∧ Expose(f, φ) ∧ Expose(g_member, φ)
/// ```
///
/// `g::f::T` navigates `[T, f]` as its host chain with `g` as the terminal
/// Symbol.  A `meta`-only outer host `T` hides the whole call at
/// `SealStatic`, even though the middle host `f` and the terminal `g` are
/// both `compile`-visible there.  Collapsing the chain to the innermost host
/// alone (the pre-fix single-host call target) would silently drop `T`'s
/// navigability constraint and wrongly reach `g`.
#[test]
fn multi_layer_navigation_gates_ordinary_call_on_every_host_in_the_chain() {
    let mut world = SemanticWorld::new("unit");
    world.bind_package_namespace(NamespaceNodeId(0));
    let provenance = Provenance::new("multi-layer host chain call gate");
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
    let visible = stage_pair(&[PolicyStage::Meta, PolicyStage::Compile]);
    // Seed the `type` rank (TypeValueId(0)) before any carrier of it.
    let _ = register(&mut world, "type_root", 0, 0, visible.clone());
    // T is the outer host and is meta-only, so it is hidden at SealStatic.
    // f is the middle host and g the terminal, both compile-visible there.
    let (t, _, _) = register(&mut world, "T", 1, 1, stage_pair(&[PolicyStage::Meta]));
    let (f, _, _) = register(&mut world, "f", 2, 2, visible.clone());
    let (g, _, _) = register(&mut world, "g", 3, 3, visible);
    let t_place = place_of(&world, t);
    let f_place = place_of(&world, f);
    // `let f::T = f;` then `let g::f = g;` — a real two-layer Val2 chain.
    world
        .associate_existing_symbol_in_place(t_place, "f", f)
        .expect("`let f::T = f`");
    world
        .associate_existing_symbol_in_place(f_place, "g", g)
        .expect("`let g::f = g`");

    // The navigator produces the COMPLETE host chain of `g::f::T`.
    let path: Vec<String> = ["g", "f", "T"].iter().map(|c| (*c).to_string()).collect();
    let navigation = world
        .navigate_semantic_path(&path, NamespaceNodeId(0), &[], &[])
        .expect("`g::f::T` navigates two host layers");
    assert_eq!(navigation.terminal_symbol, g, "the terminal Symbol is `g`");
    assert_eq!(
        navigation.host_chain.len(),
        2,
        "`g::f::T` steps through both `T` and `f`, not just the innermost host"
    );
    assert_eq!(navigation.host_chain[0].symbol, Some(t));
    assert_eq!(navigation.host_chain[1].symbol, Some(f));

    // Exposure facts: only the outer host `T` is hidden at SealStatic.
    assert!(
        !navigation.host_chain[0].exposed_at(Phase::SealStatic),
        "the meta-only outer host T is hidden at SealStatic"
    );
    assert!(
        navigation.host_chain[1].exposed_at(Phase::SealStatic),
        "the middle host f is compile-visible at SealStatic"
    );
    assert!(navigation
        .host_chain
        .iter()
        .all(|host| host.exposed_at(Phase::OpenStatic)));

    let initializer = initializer_from_source("let probe = (0) g;");
    let call_site = extract_single_call_site(&initializer).expect("normalized call site");
    let resolver = ResolverContext::new(NamespaceNodeId(0));
    let mut materialization = StructMaterializationState::default();

    let mut sealed = OrdinaryInvocationContext::open_static(&[]);
    sealed.phase = Phase::SealStatic;

    // The whole chain is gated: `T` is hidden, so `g::f::T(...)` is
    // unreachable at SealStatic. Resolution is already sealed, so the
    // projection reports `NoTargetValues` without any outward fallback.
    let blocked = invoke_host_member_symbol_ordinary(
        &mut world,
        &mut materialization,
        &navigation.host_chain,
        g,
        &call_site,
        &resolver,
        sealed.clone(),
        provenance.clone(),
    );
    assert!(
        matches!(
            blocked,
            Err(OrdinaryInvocationFailure::NoTargetValues { .. })
        ),
        "a hidden OUTER host hides the whole navigation at SealStatic: {blocked:?}"
    );

    // The exact regression: dropping the outer host (the pre-fix single-host
    // behavior) reaches member processing instead — a DIFFERENT failure — so
    // the outer host `T` was the only thing gating the call.
    let leaked = invoke_host_member_symbol_ordinary(
        &mut world,
        &mut materialization,
        &navigation.host_chain[1..],
        g,
        &call_site,
        &resolver,
        sealed,
        provenance.clone(),
    );
    assert!(
        !matches!(
            leaked,
            Err(OrdinaryInvocationFailure::NoTargetValues { .. })
        ),
        "the inner host f is visible; only the dropped outer host T was gating: {leaked:?}"
    );

    // With every host exposed the chain gate no longer blocks: the call falls
    // through to member processing (`g` carries no callable value here).
    let open = OrdinaryInvocationContext::open_static(&[]);
    let passed = invoke_host_member_symbol_ordinary(
        &mut world,
        &mut materialization,
        &navigation.host_chain,
        g,
        &call_site,
        &resolver,
        open,
        provenance,
    );
    assert!(
        !matches!(
            passed,
            Err(OrdinaryInvocationFailure::NoTargetValues { .. })
        ),
        "with every host exposed the chain gate passes: {passed:?}"
    );
}

/// Two DISTINCT associated cluster Symbols with identical pure-P and
/// sibling-val normal content produce ONE type normal form: an associated
/// Symbol's allocation identity `C_f^T ≠ C_f^U` never leaks into `Norm_type`.
///
/// The existing place-vs-identity test reuses ONE member Symbol under two
/// places; this pins the stronger claim that even two SEPARATE cluster
/// Symbols merge as long as their recursive content matches.
#[test]
fn distinct_associated_symbols_with_equal_content_share_one_type_normal_form() {
    let mut world = SemanticWorld::new("unit");
    world.bind_package_namespace(NamespaceNodeId(0));
    let provenance = Provenance::new("distinct associated symbol identity");
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
    let policy = stage_pair(&[PolicyStage::Meta, PolicyStage::Compile]);
    // A type-rank root, two carriers T and U of ONE Pattern, and two DISTINCT
    // member carriers c_t and c_u of one SECOND Pattern (equal content).
    let _ = register(&mut world, "type_root", 0, 0, policy.clone());
    let (t, _, t_pattern) = register(&mut world, "T", 1, 1, policy.clone());
    let (u, _, u_pattern) = register(&mut world, "U", 2, 1, policy.clone());
    assert_eq!(t_pattern, u_pattern, "T and U carry one Pattern");
    let (c_t, _, c_t_pattern) = register(&mut world, "c_t", 3, 2, policy.clone());
    let (c_u, _, c_u_pattern) = register(&mut world, "c_u", 4, 2, policy);
    assert_ne!(c_t, c_u, "two DISTINCT associated cluster Symbols");
    assert_eq!(
        c_t_pattern, c_u_pattern,
        "but they carry ONE Pattern, so their pure-P norms are equal"
    );

    let t_place = place_of(&world, t);
    let u_place = place_of(&world, u);
    // `let f::T = c_t;` and `let f::U = c_u;` — same name, DISTINCT Symbols,
    // equal recursive content (both empty-Val2 carriers of one Pattern).
    world
        .associate_existing_symbol_in_place(t_place, "f", c_t)
        .expect("`let f::T = c_t`");
    world
        .associate_existing_symbol_in_place(u_place, "f", c_u)
        .expect("`let f::U = c_u`");

    let carrier_type = TypeValueId(1);
    let t_addr = type_addr(&mut world, carrier_type, Some(t_place));
    let u_addr = type_addr(&mut world, carrier_type, Some(u_place));
    assert_eq!(
        t_addr, u_addr,
        "distinct associated Symbols with equal content do not separate Norm_type"
    );
}
