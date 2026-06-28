//! Canonical meta instance key and fingerprint.
//!
//! Produces deterministic fingerprints from `PreparedCallableCandidate` material.
//! Key computation uses an explicit field-by-field encoding, **not** `Debug`
//! formatting, source text, or normalized dumps.
//!
//! ## v0.8 placeholder
//!
//! Fingerprints are prefixed `v08:` to mark them as **not** cross-version-stable.
//! The final stable canonical key will use a different encoding scheme and key
//! derivation when cross-build type-value identity is implemented.

use crate::{
    fingerprint::Fnv1a64,
    meta_candidate::{
        CanonicalArgAtomKind, CanonicalMetaInstanceKeySeed, PreparedCallableCandidate,
    },
    model::{Provenance, SymbolId},
};

/// Deterministic canonical fingerprint prefixed with version marker.
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

/// Canonical meta instance key derived from a `PreparedCallableCandidate`.
///
/// The key captures everything that identifies a unique meta invocation:
/// callee identity, argument structure, type values, and build/policy context.
/// It does **not** capture binding names, provenance descriptions, or
/// declaration-level metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetaInstanceKey {
    pub fingerprint: CanonicalFingerprint,
    pub callee_symbol_id: SymbolId,
    pub provenance: Provenance,
}

// Manual Ord: use fingerprint + callee_symbol_id only. Provenance is not
// orderable and should not change key identity.
impl PartialOrd for MetaInstanceKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MetaInstanceKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.fingerprint
            .cmp(&other.fingerprint)
            .then(self.callee_symbol_id.cmp(&other.callee_symbol_id))
    }
}

/// Compute a `MetaInstanceKey` from a prepared candidate.
pub fn compute_meta_instance_key(candidate: &PreparedCallableCandidate) -> MetaInstanceKey {
    let hash = encode_canonical_meta_instance_key_seed(&candidate.canonical_key_seed);
    MetaInstanceKey {
        fingerprint: CanonicalFingerprint::new(hash),
        callee_symbol_id: candidate.callee_symbol_id,
        provenance: candidate.provenance.clone(),
    }
}

/// Encode the canonical key seed into a deterministic hex digest.
///
/// The encoding is field-by-field with length-prefixing so that concatenation
/// of neighbouring fields cannot produce false matches (e.g. `"ab" + "c"` must
/// not collide with `"a" + "bc"`).
fn encode_canonical_meta_instance_key_seed(seed: &CanonicalMetaInstanceKeySeed) -> String {
    let mut h = Fnv1a64::new();

    // Version marker
    h.write_str_field("v08");

    // Callee identity
    h.write_field(&seed.callee_function_symbol_id.0.to_le_bytes());

    // Argument arity
    h.write_field(&(seed.argument_arity as u64).to_le_bytes());

    // Unit positions
    h.write_field(&(seed.unit_positions.len() as u64).to_le_bytes());
    for pos in &seed.unit_positions {
        h.write_field(&(*pos as u64).to_le_bytes());
    }

    // Atom kinds
    h.write_field(&(seed.argument_product_shape_material.atom_kinds.len() as u64).to_le_bytes());
    for kind in &seed.argument_product_shape_material.atom_kinds {
        let discriminant = atom_kind_discriminant(kind);
        h.write_field(&[discriminant]);
    }

    // Known type values
    h.write_field(
        &(seed.argument_product_shape_material.known_type_values.len() as u64).to_le_bytes(),
    );
    for tv in &seed.argument_product_shape_material.known_type_values {
        match tv {
            None => h.write_field(&[0u8]),
            Some(tv) => {
                h.write_field(&[1u8]);
                h.write_field(&tv.0.to_le_bytes());
            }
        }
    }

    // Build/policy identity fragments
    write_opt_str(&mut h, &seed.package_identity_fragment);
    write_opt_str(&mut h, &seed.mount_identity_fragment);
    write_opt_str(&mut h, &seed.build_config_fingerprint_fragment);
    write_opt_str(&mut h, &seed.policy_export_fingerprint_fragment);

    h.finish_hex()
}

fn atom_kind_discriminant(kind: &CanonicalArgAtomKind) -> u8 {
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
