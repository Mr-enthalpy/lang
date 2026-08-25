//! Struct resident type atoms consume the type OBSERVATION
//! `Addr(Norm(Core(tau)))` at the resident place, never the bare `TypeValueId`
//! projection.
//!
//! The two boundary directions of the observation identity:
//!
//! ```text
//! same TypeValueId + same Pattern + different observed Val2
//!     → struct resident type atoms DIFFER      (no over-merge)
//! different place  + same P       + same recursive Val2
//!     → struct resident type atoms are EQUAL   (place ∉ identity)
//! ```
//!
//! The first direction is the open-type counterexample: one resident
//! TypeValue observed before and after a Val2 injection is two distinct
//! type-object observations, so two `struct` generations that consume the
//! two observations must not share one resident leaf.  The second direction
//! pins that a fresh carrier place alone never separates the leaf.

mod support;

use lang_build::{
    extract_single_call_site, BuildManifest, CanonicalFullNavigation, CanonicalPatternAtom,
    CanonicalPatternBuilder, CanonicalPatternValue, CanonicalTypeObservation, CompilationWorld,
    InvocationOutcome, OrdinaryInvocationContext, OrdinaryInvocationFailure,
    PatternNavigationInput, Provenance, SemanticTypeEnv, TypeResolutionEnv, ValueMutability,
};
use support::{build_single_fixture_world, initializer_from_source};

fn invoke_struct(
    world: &mut CompilationWorld,
    spelling: &str,
    provenance: &str,
) -> Result<InvocationOutcome, OrdinaryInvocationFailure> {
    let initializer = initializer_from_source(spelling);
    let call_site = extract_single_call_site(&initializer).expect("normalized struct call");
    world.invoke_ordinary_call(
        world.package_root_node(),
        &call_site,
        OrdinaryInvocationContext::open_static(&[ValueMutability::Const]),
        Provenance::new(provenance),
    )
}

fn generated_struct(
    world: &mut CompilationWorld,
    spelling: &str,
    provenance: &str,
) -> lang_build::GeneratedTypeDefinitionValue {
    let outcome = invoke_struct(world, spelling, provenance).expect("struct invocation succeeds");
    let InvocationOutcome::SingleMember(result) = outcome else {
        panic!("struct returns one complete type value");
    };
    let lang_build::OrdinaryReturnedValue::CompleteType(returned) = result.returned else {
        panic!("struct semantic result is complete tau");
    };
    returned
        .construction_material
        .expect("compatibility projection retains replayable struct material")
}

/// One resident pattern `((<field type> <binder>) <top>) |> struct` expressed
/// as the expected canonical map with an explicit resident leaf atom.
fn expected_single_field_pattern(
    top: &str,
    binder: &str,
    resident: CanonicalTypeObservation,
) -> CanonicalPatternValue {
    let mut builder =
        CanonicalPatternBuilder::named_root(CanonicalFullNavigation::from_component(top));
    builder
        .contribute_pattern_value(
            PatternNavigationInput::Explicit(CanonicalFullNavigation::new([binder, top])),
            CanonicalPatternValue::Atom(CanonicalPatternAtom::Type(resident)),
        )
        .expect("the explicit complete navigation is unique");
    builder.finish()
}

/// The over-merge counterexample: ONE resident TypeValue (`uint8`), ONE
/// Pattern, but a Val2 injection between two `struct` generations — the two
/// resident leaves consume two different observations and must differ.
///
/// ```text
/// t_1 = ⟨P_t, ∅⟩        ((t x) A) |> struct
/// let f::t = type;
/// t_2 = ⟨P_t, {f}⟩      ((t x) B) |> struct
/// Atom(t_1) ≠ Atom(t_2)  even though TypeValueId(t_1) = TypeValueId(t_2)
/// ```
#[test]
fn same_type_value_with_different_observed_val2_separates_struct_resident_atoms() {
    let mut world =
        CompilationWorld::from_manifest(&BuildManifest::new("app", vec!["app".to_string()]))
            .expect("core semantic world builds");
    let resident_type = world
        .resolve_type_value("uint8")
        .expect("core uint8 type resolves semantically");
    let uint8_symbol = world
        .semantic_world()
        .resolve_symbol_path(&["uint8".to_string()], world.core_node(), &[], &[])
        .expect("core uint8 carrier symbol resolves");
    let uint8_place = world
        .semantic_world()
        .symbol(uint8_symbol)
        .expect("registered core symbol")
        .pure_p_place()
        .expect("a pure P is a real object with its own place");
    let injected_member = world
        .semantic_world()
        .resolve_symbol_path(&["type".to_string()], world.core_node(), &[], &[])
        .expect("core `type` carrier symbol resolves");

    // First observation: the resident's Val2 is still empty.
    let before = world
        .canonical_type_core_observation_address(resident_type, Some(uint8_place))
        .expect("acyclic Val2 normalizes");
    let a = generated_struct(
        &mut world,
        "let A: type = ((uint8 x) A) |> struct;",
        "resident observation before injection",
    );
    assert_eq!(
        a.fields[0].type_observation,
        CanonicalTypeObservation::Observed(before),
        "the generated field consumes the observed Core(tau), not the bare TypeValueId"
    );
    assert_eq!(
        a.canonical_pattern_value(),
        expected_single_field_pattern("A", "x", CanonicalTypeObservation::Observed(before)),
        "the struct Pattern leaf is the observation read at THIS invocation"
    );

    // `let f::uint8 = type;` — the resident's observed Val2 changes.
    world
        .associate_existing_symbol_in_place(uint8_place, "f", injected_member)
        .expect("the injection records a Val2 name on the resident's own object");
    let after = world
        .canonical_type_core_observation_address(resident_type, Some(uint8_place))
        .expect("acyclic Val2 normalizes");
    assert_ne!(
        before, after,
        "⟨P, ∅⟩ and ⟨P, {{f}}⟩ are two different type-object observations"
    );

    let b = generated_struct(
        &mut world,
        "let B: type = ((uint8 x) B) |> struct;",
        "resident observation after injection",
    );
    assert_eq!(
        b.fields[0].type_observation,
        CanonicalTypeObservation::Observed(after),
        "the second generation consumes the second observation"
    );
    assert_eq!(
        b.canonical_pattern_value(),
        expected_single_field_pattern("B", "x", CanonicalTypeObservation::Observed(after)),
    );

    // The over-merge boundary itself: one TypeValueId, two distinct atoms.
    assert_eq!(
        a.fields[0].type_value, b.fields[0].type_value,
        "both residents project ONE TypeValueId"
    );
    assert_ne!(
        CanonicalPatternValue::Atom(CanonicalPatternAtom::Type(a.fields[0].type_observation)),
        CanonicalPatternValue::Atom(CanonicalPatternAtom::Type(b.fields[0].type_observation)),
        "same TypeValueId, different observed Val2 → the struct resident atoms differ"
    );
}

/// The reverse direction: two carriers `T` and `U` of ONE TypeValue at two
/// DIFFERENT places, with equal (empty) recursive Val2 — the two struct
/// resident leaves normalize onto one observation address, so the atoms are
/// equal.  A place is only the observation coordinate, never identity.
#[test]
fn different_place_with_equal_recursive_val2_shares_one_struct_resident_atom() {
    let mut world = build_single_fixture_world("struct_resident_observation", "app");
    let context = world.package_context();
    let (t_resolution, u_resolution) = {
        let env = SemanticTypeEnv::new(world.semantic_world());
        (
            env.resolve_type_path(&["T".to_string()], &context)
                .expect("`let T: type = uint8;` resolves"),
            env.resolve_type_path(&["U".to_string()], &context)
                .expect("`let U: type = uint8;` resolves"),
        )
    };
    assert_eq!(
        t_resolution.represented_type, u_resolution.represented_type,
        "T and U carry one TypeValue"
    );
    assert_ne!(
        t_resolution.carrier_place, u_resolution.carrier_place,
        "T and U observe it from two distinct places"
    );

    // Distinct field binders keep the two generations two distinct ambient
    // shapes: with an identical binder the two invocations already collide,
    // because the resident observations are equal — the merge under test.
    let a = generated_struct(
        &mut world,
        "let A: type = ((T x) A) |> struct;",
        "resident observed through T's place",
    );
    let b = generated_struct(
        &mut world,
        "let B: type = ((U y) B) |> struct;",
        "resident observed through U's place",
    );

    assert_eq!(a.fields[0].type_value, b.fields[0].type_value);
    assert!(
        matches!(
            a.fields[0].type_observation,
            CanonicalTypeObservation::Observed(_)
        ),
        "a world-connected struct invocation attaches a real observation"
    );
    assert_eq!(
        a.fields[0].type_observation, b.fields[0].type_observation,
        "different place, same P, same recursive Val2 → ONE observation address"
    );
    assert_eq!(
        CanonicalPatternValue::Atom(CanonicalPatternAtom::Type(a.fields[0].type_observation)),
        CanonicalPatternValue::Atom(CanonicalPatternAtom::Type(b.fields[0].type_observation)),
        "the struct resident atoms are equal across the two carrier places"
    );
}
