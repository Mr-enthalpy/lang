# Open questions

This document contains only unresolved design or representation questions.
Closed semantic laws live in their canonical topic owners; implementation
wiring that applies a closed law is tracked in `roadmap.md`, not here.

## Identity and canonical representation

- What persistent representation should encode the opaque Core lookup index
  currently named `TypeValueId`?
- How should persistent `SemanticOwnerId`, syntax-local keys, Pattern roots,
  and MetaInstance roots be serialized and restored while preserving parent
  homomorphism?
- What concrete representation should carry resident generations and
  ProjectionSlot identity?
- Which value payload families should gain content normalization after the
  safe identity-stable opaque Val1 boundary?
- How should generic complete-type expressions such as `(int Vec::std)`
  lower into complete type values?

The following facts are not open: ordinary Object normalization observes
Val1/Pattern/owned-Val2; ordinary type equality observes Core; whole-snapshot
positions observe the complete immutable `tau`; lookup indices, Symbols,
Places, and whole snapshots are distinct identities.

## Pattern representation and extraction IR

- What is the final canonical-space representation of a Pattern?
- What concrete derivation/residual IR should record `R_Gamma(P,c,rho)` proofs,
  extraction chains, and multiple valuations?
- How should custom `?` providers expose richer extraction interfaces?
- How should static Pattern spaces and `Done` be represented in later IR?
- How should closed control-pattern non-additivity be enforced through package
  ownership and lookup routing?

The relational interface, Hole identity, DirectPatternChild distinction,
StructuralDefault family boundary, and genericity-as-extraction are fixed.

## Literals and concrete type catalog

- What is the source spelling and exact-value model for character literals?
- Which additional exact-real spellings are supported?
- What is the concrete machine-Type catalog?
- Which source/context mechanism requests a concrete literal construction
  target when no explicit annotation supplies one?

Abstract integer/real/character values form first; concrete construction and
same-Type Policy migration are separate ordinary operations.

## Policy surface and overload extensions

- What surface spelling, if any, denotes the absent-value Policy pattern?
- How should a surface Policy annotation be diagnosed when whole-slot mode
  factorization leaves an empty pair side?
- Which named overload strategies beyond compiler-known must-select are
  available, and what monotone comparison law does each use?
- What is the final call-site candidate-family selector syntax?
- What are the remaining later B-filter interfaces and their proof carriers?
- How does any future policy stage compose with the fixed OpenStatic,
  SealStatic, and Runtime phases?
- Which effect/error/panic/resource capability dimensions are added to
  DynamicLegality?

Primitive `PolicyMode={const,plain,mut}`, total result demand before maxima,
three-point preference, 3×3 capability realization, unique selection, and no
reopen are fixed.

## Residual and serial evaluation

- What IR represents partially evaluated invocation frames across static and
  runtime phases?
- Where is the sequencing frontier for effectful expressions under progressive
  evaluation?
- What ABI and storage representation carries residual continuations?
- How do capability and effect summaries compose with residual evaluation?
- What concrete representation carries `Done`, targeted return, and result
  Pattern delivery?

Runtime continuation may not re-resolve a Symbol, namespace path, candidate
family, or sealed invocation.

## Place, construction, and write algebra

- What final cluster write algebra exposes and replaces individual facets of
  an existing member?
- Which IR event closes an ordinary construction window, and how is that event
  scheduled relative to graph seal?
- How can external objects intentionally expose extension points?
- What is the final source integration for an externally navigated call-entry
  extension?
- What final syntax exposes coordinated value/ref/share receiver families, if
  any?

Binding creation, member creation, member write, assignment, inject, rebind,
ref, and share remain distinct. `extend` is pure; `inject` is
read+extend+write; OpenHere, Writable, PolicyMode, and authority are independent.

## Lifetime, Color, and access

- What concrete IR represents LifeName, NameView, LifetimeValue, Region
  generations, and cleanup placement?
- What source/storage syntax names an open Color vocabulary?
- What algorithm constructs the access tree and performs escape validation?
- Which summary compression, diagnostics, and extended temporal logic are
  useful without changing the closed lifecycle relations?
- How should compile caches represent call-site Open-sensitive applicability:
  an uncached judgment or an explicit requirement summary?

Cleanup is fixed before observation; Pre precedes mutation; Post describes only
committed success; move ends and starts generations at one continuation cut;
Color relations are explicit directed rows and Color inheritance is monotone.

## Owner, namespace, and build persistence

- What is the manifest file format and version/registry solver?
- How are package roots, mount identities, namespace snapshots, and
  MetaInstance roots persisted across incremental builds?
- What API surface expresses resolver expected-role disambiguation?
- Are escaped field names needed outside existing object/subspace conflicts?
- What is the final form of `unique trait`?
- How do external namespace providers expose candidates while preserving
  package boundary and stable admission facts?
- How does type-associated namespace traversal interact with Core equality and
  whole-snapshot callspace capture?

## Closure, control flow, and ownership

- What carrier materializes a Closure AST as a callable object and lays out
  explicit/automatic captures?
- How are in-place closure embedding reads resolved without inventing captures?
- How should the NLL/control-flow graph be represented?
- What source-defined control-pattern family expresses D-reduction and
  `if`/`else`/unit absorption through ordinary calls?
- How are `return`, effect, and sync operations integrated into the shared
  semantic continuation?
- What complete operator-environment selector algebra maps spelling+fixity+arity
  to an ordinary Symbol?

## Generic navigation and Product surface

- Can a closed complete type carry one finite general navigation operation
  that extracts a requested binding-name key from known request material,
  without adding members or introducing a universal name quantifier?
- Should intrinsic Product ordinal selectors be exposed as a user-visible
  tuple namespace API or remain structural navigation only?
- How does source code reference or replace a derived compile companion?
- Can default companion generation ever be suppressed, and what equivalent
  compile Pattern/contract must replace it?
- What finer-grained identity, if any, is needed for grouped inferred-require
  atoms?

## Bootstrap boundary

For each compiler-provided operation, determine whether it is a bootstrap seed,
a source definition still to be connected, an intrinsic observation, or a
semantic primitive justified by non-bootstrappability. No current family has a
non-bootstrappability proof.

Concrete source definitions for literal construction, construction/migration
families, capability entries, StructuralDefault providers, and lifecycle
algebra remain future work. Ordinary selection and the canonical relations
already determine their meaning.
