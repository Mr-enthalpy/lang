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
positions observe the complete immutable `tau`; lookup indices, name bindings,
Places, and whole snapshots are distinct identities. Named-type synthesis and
ordinary OverloadGroup aggregation are different algebras; a group has no
additional authority over the complete types it aggregates.

## Pattern representation and extraction IR

- What is the final canonical-space representation of a Pattern?
- What concrete derivation/residual IR should record `R_Gamma(P,c,rho)` proofs,
  extraction chains, and multiple valuations?
- How should custom `?` providers expose richer extraction interfaces?
- How should static Pattern spaces and `Done` be represented in later IR?
- Which source-level ownership/lookup consumer enforces the existing closed
  control-pattern non-additivity relation?

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
reopen are fixed. Formal PolicyPair inherits P2; its independent omitted
PolicyMode is plain. Return-position refinement keeps its separately specified
P_out rule. These are not competing defaults.

## Residual and serial evaluation

- What IR represents partially evaluated invocation frames across static and
  runtime phases?
- What concrete effect and overlay representation implements sequential file
  actions and unordered common-snapshot sibling composition?
- What ABI and storage representation carries residual continuations?
- How do capability and effect summaries compose with residual evaluation?
- What concrete representation carries `Done`, targeted return, and result
  Pattern delivery?

Runtime continuation may not re-resolve a name, namespace path, candidate
family, or sealed invocation. E saturation, projection synchronization and
rewrite revalidation are fixed. Implicit return selects the outermost enclosing
function layer; consumer alignment is roadmap work.

## Place, construction, and write algebra

- What concrete capability carrier records the Place's pending first-write
  authority and its consumption across aliases? Declaration-policy independence,
  const initialization, current Pre checks and no replacement power from saved
  initial references are closed laws, not representation choices.

- Which ordinary assignment candidates, if any, realize closure-to-type-ref
  assignment using existing construction/replication operations? Typed structural
  name creation and explicit borrowing do not themselves select such a candidate. Every candidate must preserve the existing assignment boundaries.

- What carrier/entry encoding realizes the closed group candidate domain and
  aggregation laws (with BucketEq(T1,T2) iff Norm_type(T1)=Norm_type(T2)) without erasing entries through an
  unrelated cache/value-identity quotient? This is not an open semantic codomain.
- What IR represents the already defined construction-window termination,
  meta completion, plain closure and externally visible name-set closure events?
- What public spelling should the builtin associated-state callable A use?
- Which values beyond closure-expression-produced closures can prove a
  location-parametric ReinstantiationWitness? The initial domain is fixed;
  arbitrary owner-changing replication is not admitted.
- What concrete witness/template and alpha-renaming representation implements
  anchored replication while preserving captures and internal identity edges?

Typed structural NameExpr creation establishes an uninitialized Place;
explicit borrowing and ordinary first write initialize it. Initialization
authority is independent of const/plain/mut and is consumed on the successful
first commit; later replacement has separate capability and resident
compatibility checks. Close requires
initialized resolvable members. First named contribution uses one-shot formation
and initialization; later contributions extend an existing resident.
Named-contribution positions synthesize
V_tau, while ordinary lexical let does not aggregate by spelling. Type +=/-=
requires OpenHere and final anchored closure membership, changing only V_tau;
Core changes use extend/inject. Ordinary group mutation needs its own Writable,
not OpenHere of its contained types. Meta invocation constructs ordinary result
instance names/types with dependency-derived openness. P1 meta retains that
qualification; plain let completes/closes the instance. Arbitrary Val2 payloads
use ordinary navigation and policy; V_tau and Pattern registration are independent.
Its input normalization retains semantically observed name/subject identities;
its registry/cache preserves instance/member Places and current state.
A is a derived instance, with its construction subject retained through Close and
input-carrier replacement. Generalization is settled by these meta invocation
laws; ordinary Core equality does not merge subjects or grant write authority.
Persistent dependency/source encoding remains open, not the propagation law.

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

## Owner, namespace, and infrastructure persistence

- How are semantic owner roots, namespace snapshots, MetaInstance result names,
  current type/member observations, construction status and dependency effects persisted across
  incremental evaluation without extending dependency lifetimes?
- What concrete IR represents invocation input identity observations and output
  access-dependency/opening-source summaries?
- What API expresses context-directed member projection after stable name
  resolution without turning consumer roles into name ontologies?
- Are escaped field names needed outside the existing syntax?
- What is the final form of the existing unique-trait design?
- What concrete host IO/FFI APIs expose ordinary Objects and policy views?
- How do traversal/index consumers preserve Core equality while retaining
  captured complete callspaces in whole-snapshot observations?

Physical normalization includes child-directory names desugaring to ordinary
fresh-name/type actions followed by their body under the resulting reference;
the selected root and filenames add no segments. This law, main.lang anchoring,
sibling overlays and post-hoc
DependencyGraph projection are fixed. Manifests, mounts, registry solvers and
package graphs are not alternative semantic inputs. Retrieval APIs may be
ordinary source/host work; their implementation does not create authority.

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
  to ordinary callable selection material?

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
semantic primitive justified by non-bootstrappability. A permanent host primitive requires the unavailable-host-capability
justification; existing representation/library mechanisms take priority. A's
source definition uses a general meta instance type with an ordinary Val2 group
Place. The singleton-Val2 compile helper may also begin as a builtin and later
use source enumeration and compile error expression; its public spelling and
concrete error representation remain to be selected. Its current bootstrap notation creates no independent map primitive.

Concrete source definitions for literal construction, construction/migration
families, capability entries, StructuralDefault providers, and lifecycle
algebra remain future work. Ordinary selection and the canonical relations
already determine their meaning.
