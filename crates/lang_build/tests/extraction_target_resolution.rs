//! Real extraction-target resolution.
//!
//! `expand_extraction_navigation` is not a standalone helper any more: these
//! tests drive `SemanticWorld::resolve_extraction_target`, which runs the
//! full chain against a real fixture world:
//!
//! ```text
//! extraction tree position
//! -> expand_extraction_navigation (nearest explicit anchor / implicit
//!    global top)
//! -> completed full Symbol path
//! -> exact resolution from the global namespace root
//! -> the Symbol's PatternValue
//! -> canonical pattern norm
//! ```
//!
//! The counter-examples pinned here:
//! * a top pattern with omitted navigation means exact `::` — a symbol that
//!   is visible through the bare-name scope chain is NOT a hit;
//! * the nearest explicit navigation anchor wins in real resolution;
//! * an intermediate `Absent` layer keeps extending the path outward;
//! * an anchorless parent chain is a hard diagnostic.

mod support;

use lang_build::{
    CanonicalFullNavigation, ExtractionPatternParent, PatternOwnNavigation, Provenance,
};

use support::build_single_fixture_world;

fn provenance() -> Provenance {
    Provenance::new("extraction-target resolution test")
}

fn nav(components: &[&str]) -> CanonicalFullNavigation {
    CanonicalFullNavigation::new(components.iter().copied())
}

// ---------------------------------------------------------------------------
// Top omitted navigation means exact `::`, never bare-name fallback.
// ---------------------------------------------------------------------------

#[test]
fn top_omitted_navigation_is_exact_global_never_bare_name() {
    let world = build_single_fixture_world("s10_type_binding", "app");
    let semantic = world.semantic_world();

    // Control fact: `uint8` IS visible from the package root through the
    // ordinary bare-name scope chain (core is a default mount).
    semantic
        .resolve_symbol_path(
            &["uint8".to_string()],
            world.package_root_node(),
            &[],
            &[world.core_node()],
        )
        .expect("bare-name lookup sees core uint8 from the package root");

    // Extraction fact: the same bare subject with an implicit-global top
    // resolves exactly from the global root, where no `uint8` symbol lives.
    let error = semantic
        .resolve_extraction_target(&nav(&["uint8"]), None, &[], provenance())
        .expect_err("implicit-global extraction must not reuse bare-name visibility");
    assert!(
        error
            .message
            .contains("never falls back to bare-name lookup"),
        "diagnostic names the exact-resolution rule: {}",
        error.message
    );

    // The full global path is the only extraction spelling that hits.
    let uint8 = semantic
        .symbol_in_namespace(world.core_node(), "uint8")
        .expect("core uint8 symbol");
    let target = semantic
        .resolve_extraction_target(&nav(&["uint8", "core"]), None, &[], provenance())
        .expect("exact global path resolves");
    assert_eq!(target.symbol, uint8.identity, "resolved the core symbol");
    assert_eq!(
        Some(target.pattern),
        uint8.pure_p_pattern(),
        "the extraction target is the symbol's PatternValue"
    );
    assert_eq!(
        Some(target.norm.clone()),
        semantic.canonical_pattern_norm(target.pattern),
        "the resolved norm is the canonical pattern norm"
    );
}

// ---------------------------------------------------------------------------
// Nearest explicit anchor wins in real resolution.
// ---------------------------------------------------------------------------

#[test]
fn nearest_explicit_anchor_wins_in_real_resolution() {
    let world = build_single_fixture_world("s10_type_binding", "app");
    let semantic = world.semantic_world();
    let uint8 = semantic
        .symbol_in_namespace(world.core_node(), "uint8")
        .expect("core uint8 symbol");

    // Nearest parent anchors at `core`; a farther parent anchors at the
    // package root. The nearest anchor terminates inheritance, so the
    // completed path is `uint8::core` and resolution hits the core symbol.
    let parents = [
        ExtractionPatternParent::new(
            nav(&["ignored-local"]),
            PatternOwnNavigation::Explicit(nav(&["core"])),
        ),
        ExtractionPatternParent::new(
            nav(&["ignored-outer"]),
            PatternOwnNavigation::Explicit(nav(&["app"])),
        ),
    ];
    let target = semantic
        .resolve_extraction_target(&nav(&["uint8"]), None, &parents, provenance())
        .expect("nearest explicit anchor resolves");
    assert_eq!(target.symbol, uint8.identity);

    // Swapping the anchors changes the real resolution outcome: the nearest
    // anchor now completes `uint8::app`, and the package root holds no
    // `uint8` symbol — the farther `core` anchor must NOT rescue the path.
    let swapped = [
        ExtractionPatternParent::new(
            nav(&["ignored-local"]),
            PatternOwnNavigation::Explicit(nav(&["app"])),
        ),
        ExtractionPatternParent::new(
            nav(&["ignored-outer"]),
            PatternOwnNavigation::Explicit(nav(&["core"])),
        ),
    ];
    let error = semantic
        .resolve_extraction_target(&nav(&["uint8"]), None, &swapped, provenance())
        .expect_err("the nearest anchor is authoritative even when it misses");
    assert!(
        error
            .message
            .contains("unresolved extraction path `uint8::app`"),
        "resolution used the nearest anchor's path: {}",
        error.message
    );
}

// ---------------------------------------------------------------------------
// An intermediate Absent layer keeps extending the path outward.
// ---------------------------------------------------------------------------

#[test]
fn intermediate_absent_layer_extends_the_completed_path() {
    let world = build_single_fixture_world("s10_type_binding", "app");
    let semantic = world.semantic_world();

    // subject `exists`, an Absent layer named `verify`, then an explicit
    // `core` anchor: the completed path is `exists::verify::core`, which
    // resolves to the real core verification primitive — a symbol without
    // a pure-P PatternValue, so the failure is the pattern-shape check,
    // NOT an unresolved path.
    let parents = [
        ExtractionPatternParent::new(nav(&["verify"]), PatternOwnNavigation::Absent),
        ExtractionPatternParent::new(
            nav(&["ignored-outer"]),
            PatternOwnNavigation::Explicit(nav(&["core"])),
        ),
    ];
    let error = semantic
        .resolve_extraction_target(&nav(&["exists"]), None, &parents, provenance())
        .expect_err("the verification primitive is not a Pattern");
    assert!(
        error.message.contains("without a pure-P"),
        "the Absent layer extended the path to a real non-pattern symbol: {}",
        error.message
    );
}

// ---------------------------------------------------------------------------
// An anchorless parent chain is a hard diagnostic.
// ---------------------------------------------------------------------------

#[test]
fn anchorless_parent_chain_is_a_hard_diagnostic() {
    let world = build_single_fixture_world("s10_type_binding", "app");
    let semantic = world.semantic_world();

    let parents = [
        ExtractionPatternParent::new(nav(&["mid"]), PatternOwnNavigation::Absent),
        ExtractionPatternParent::new(nav(&["outer"]), PatternOwnNavigation::Absent),
    ];
    let error = semantic
        .resolve_extraction_target(&nav(&["uint8"]), None, &parents, provenance())
        .expect_err("no enclosing layer carries an anchor");
    assert!(
        error.message.contains("no anchor"),
        "diagnostic names the missing anchor: {}",
        error.message
    );
}
