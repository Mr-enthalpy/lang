//! Parent-neutral meta invocation material key and fingerprint.
//!
//! `MetaInvocationMaterialKey = MetaCallableIdentity × CanonicalArgumentProductAddr`
//! the key STORES its structural coordinates and defines
//! equality/ordering directly on them.  The FNV fingerprint is a derived
//! digest for display/transport only — it never defines semantic equality.
//!
//! The `PreparedCallableCandidate` digest channel survives only as an opaque
//! compatibility-cache digest; it no longer produces a semantic root key.

use crate::{
    canonical_value::CanonicalValueAddr,
    fingerprint::Fnv1a64,
    identity::MetaCallableIdentity,
    meta_candidate::{
        CanonicalArgAtomKind, CanonicalArgProductShapeMaterial, PreparedCallableCandidate,
    },
    model::Provenance,
};

/// Deterministic canonical fingerprint prefixed with version marker.
///
/// A fingerprint is DERIVED material: display, transport, and legacy cache
/// digests only.  It never defines equality of any semantic identity.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalFingerprint {
    pub value: String,
}

impl CanonicalFingerprint {
    pub fn new(hex: String) -> Self {
        Self {
            value: format!("v08:{hex}"),
        }
    }
}

/// Parent-neutral structural key for replayable meta invocation material.
///
/// ## Equality and ordering
///
/// Equality and ordering are defined DIRECTLY on the structural coordinates
/// `(callable, arguments)` — never on a digest.
/// `provenance` is excluded: it is diagnostic context, not canonical
/// identity.  Graph declaration SymbolIds never enter the key: the
/// callable coordinate is the selected function object
/// VALUE identity plus its selected `()` call entry.
#[derive(Clone, Debug)]
pub struct MetaInvocationMaterialKey {
    /// Selected meta callable: function object value + selected call entry.
    pub callable: MetaCallableIdentity,
    /// Canonical address of the whole argument Product,
    /// `Addr(Product(a1..an))`.
    pub arguments: CanonicalValueAddr,
    pub provenance: Provenance,
}

impl MetaInvocationMaterialKey {
    /// Structural identity coordinates participating in Eq/Ord.
    fn coords(&self) -> (MetaCallableIdentity, CanonicalValueAddr) {
        (self.callable, self.arguments)
    }

    /// Derived display/transport fingerprint of the structural coordinates.
    ///
    /// This digest NEVER defines equality; it is recomputed from the stored
    /// structural coordinates on demand.
    pub fn fingerprint(&self) -> CanonicalFingerprint {
        let mut h = Fnv1a64::new();
        // Version marker for the normalized canonical meta key encoding.
        h.write_str_field("v09-source-meta-norm");
        // Selected function object value identity + selected call entry.
        h.write_field(&self.callable.selected_function_value.as_u64().to_le_bytes());
        h.write_field(&self.callable.selected_call_entry.as_u64().to_le_bytes());
        // The whole argument tuple as one interned Product address.
        h.write_field(&self.arguments.as_u64().to_le_bytes());
        CanonicalFingerprint::new(h.finish_hex())
    }
}

impl PartialEq for MetaInvocationMaterialKey {
    fn eq(&self, other: &Self) -> bool {
        self.coords() == other.coords()
    }
}

impl Eq for MetaInvocationMaterialKey {}

impl PartialOrd for MetaInvocationMaterialKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MetaInvocationMaterialKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.coords().cmp(&other.coords())
    }
}

/// Compatibility cache digest of a prepared meta candidate.
///
/// This is the surviving remnant of the pre-canonical key channel: an opaque
/// digest used ONLY by `MetaInstanceCache`. It is not a
/// `MetaInvocationMaterialKey` and defines no semantic identity.
///
/// The digest material is derived on demand from the candidate's argument
/// product shape and build-identity fragments — there is no stored second
/// "canonical key" definition.  The encoding is field-by-field with
/// length-prefixing so that concatenation of neighbouring fields cannot
/// produce false matches (e.g. `"ab" + "c"` must not collide with
/// `"a" + "bc"`).
pub fn compute_legacy_meta_instance_digest(
    candidate: &PreparedCallableCandidate,
) -> CanonicalFingerprint {
    let material =
        CanonicalArgProductShapeMaterial::from_arg_product_shape(&candidate.arg_product_shape);
    let mut h = Fnv1a64::new();

    // Version marker
    h.write_str_field("v08");

    // Callee identity
    h.write_field(&candidate.callee_symbol_id.0.to_le_bytes());

    // Argument arity
    h.write_field(&(material.arity as u64).to_le_bytes());

    // Unit positions
    h.write_field(&(material.unit_positions.len() as u64).to_le_bytes());
    for pos in &material.unit_positions {
        h.write_field(&(*pos as u64).to_le_bytes());
    }

    // Atom kinds
    h.write_field(&(material.atom_kinds.len() as u64).to_le_bytes());
    for kind in &material.atom_kinds {
        let discriminant = atom_kind_discriminant(kind);
        h.write_field(&[discriminant]);
    }

    // Known type values. Name/carrier navigation has already been evaluated
    // and must not affect canonical invocation identity.
    h.write_field(&(material.known_type_values.len() as u64).to_le_bytes());
    for type_value in &material.known_type_values {
        match type_value {
            None => h.write_field(&[0u8]),
            Some(type_value) => {
                h.write_field(&[1u8]);
                h.write_field(&type_value.0.to_le_bytes());
            }
        }
    }

    // Build/policy identity fragments
    write_opt_str(&mut h, &candidate.build_identity.package_identity_fragment);
    write_opt_str(&mut h, &candidate.build_identity.mount_identity_fragment);
    write_opt_str(
        &mut h,
        &candidate.build_identity.build_config_fingerprint_fragment,
    );
    write_opt_str(
        &mut h,
        &candidate.build_identity.policy_export_fingerprint_fragment,
    );

    CanonicalFingerprint::new(h.finish_hex())
}

/// Compute the parent-neutral material key of one meta invocation from the
/// selected meta callable identity and the canonical address of the whole
/// argument Product.
///
/// `MetaInvocationMaterialKey = MetaCallableIdentity × Addr(Product(a1..an))` — this
/// single key mechanism serves source-declared AND core meta callables.
/// The invocation parentheses are themselves a
/// Product value, so the arguments participate as one Product normal form
/// whose members are the per-position canonical addresses: top-level
/// argument equivalence is order-sensitive because Product identity is
/// positional, not because of any sequence encoding here.  Formal binder
/// names, source paths, body material, backing declaration SymbolIds, and
/// carrier Symbols never enter this key.  α-renaming a formal binder
/// cannot change the key; two distinct meta function values under one
/// carrier Symbol always produce distinct keys.
pub fn compute_meta_invocation_material_key(
    callable: MetaCallableIdentity,
    arguments_product_addr: CanonicalValueAddr,
    provenance: Provenance,
) -> MetaInvocationMaterialKey {
    MetaInvocationMaterialKey {
        callable,
        arguments: arguments_product_addr,
        provenance,
    }
}

pub(crate) fn atom_kind_discriminant(kind: &CanonicalArgAtomKind) -> u8 {
    match kind {
        CanonicalArgAtomKind::ExpressionBarrier => 0,
        CanonicalArgAtomKind::ResolvedValue => 1,
        CanonicalArgAtomKind::TypeObject => 2,
        CanonicalArgAtomKind::RankObject => 3,
        CanonicalArgAtomKind::NamespaceObject => 4,
        CanonicalArgAtomKind::MetaObject => 5,
        CanonicalArgAtomKind::PatternObject => 6,
        CanonicalArgAtomKind::ProductUnit => 7,
        CanonicalArgAtomKind::Unsupported => 8,
    }
}

fn write_opt_str(h: &mut Fnv1a64, opt: &Option<String>) {
    match opt {
        None => h.write_field(&[0u8]),
        Some(s) => {
            h.write_field(&[1u8]);
            h.write_str_field(s);
        }
    }
}
