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

The relational interface, Hole identity, DirectPatternChild distinction,
StructuralDefault family boundary and genericity-as-extraction are fixed.
Restricted Split/D, internal chain/target completion and separate residual escape
are closed. General Boolean difference, arbitrary custom residual protocols and
universal result joins are not implied.

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

Primitive PolicyMode={const,plain,mut}, demand formation before maxima,
three-point preference, capability realization, unique selection and no reopen
are fixed. Stage is one atom; the static trio is pairwise incomparable. P2 is
horizon, P1/Pout producer visibility. Pin admits explicit stage constraints and
holes; Pout.stage=P1.stage. InputAdmissible, migration and Ready are separate.
Runtime P2 defaults omitted P1 to runtime; seal defaults to seal. Bare let
supplies no override; written concrete mode and explicit hole are distinct.
Policy deduction uses ordinary operator Pattern relations, HoleBinderId and
require. The 3×3 tables are finite explanatory views of relational declarations,
not a separate inference primitive. Default completion is not a source constraint.

## Residual and serial evaluation

- What IR represents partially evaluated invocation frames across static and
  runtime phases?
- What concrete effect and overlay representation implements sequential file
  actions and unordered common-snapshot sibling composition?
- What ABI and storage representation carries residual continuations?
- How do capability and effect summaries compose with residual evaluation?
- What concrete representation carries `Done`, targeted return, and result
  Pattern delivery?

Runtime main and actual stack-relative MetaDom/SealDom, two-round R_vis then
ordinary C_sigma selection, no speculative candidate execution and retained
selected origin are closed. Concrete witness/cache representation is open.

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
  meta completion, plain closure and registered-structure closure events?
- What public spelling should the builtin associated-state callable A use?
- Which values beyond closure-expression-produced closures can prove a
  location-parametric ReinstantiationWitness? The initial domain is fixed;
  arbitrary owner-changing replication is not admitted.
- What concrete witness/template and alpha-renaming representation implements
  anchored replication while preserving captures and internal identity edges?

Typed structural NameExpr creation establishes an uninitialized Place;
qualified formation is value-side, using resolved structural root identity and
current type OpenHere without parent Writable or parent mut type ref. Explicit
borrowing is Place-side. Contextual meta qualification is closed for type/type
ref only; direct mut and explicit meta-to-mut confirmation are coherent ordinary
candidates, not an implicit chain. Both recheck OpenHere, target Writable and
ordinary capability/access/lifetime. Saved refs preserve their borrowed subject
and cannot evade Close. These are closed laws; consumer encoding remains pending.
Explicit borrowing and ordinary first write initialize the Place. Initialization
authority is independent of const/plain/mut and is consumed on the successful
first commit; later replacement has separate capability and resident
compatibility checks. Close requires
initialized retained members being published, not realization of every coordinate.
First named contribution uses one-shot formation
and initialization; later contributions extend an existing resident.
Named-contribution positions synthesize
V_tau, while ordinary lexical let does not aggregate by spelling. Type +=/-=
requires OpenHere and final anchored closure membership, changing only V_tau;
Pattern-registered structural changes use extend/inject; ordinary name writes
may independently change Val2(Core). Ordinary group mutation needs its own Writable,
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
committed success. Killable is instance-local, MoveEffect is fixed, and Movable
is frontier legality. A killing move ends and starts generations at one cut;
preserving moves require the narrow proof. Directed with placement, default NLL,
lexical empty-with, no duplicate destructor and same-K @ observations are closed;
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
- What source library/API expresses trait-like laws as ordinary meta results?
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

- Which carrier realizes ordinary struct Material_C into full tau_C, its c_C,
  A_C and () roles, and represents already determined dependency realizations?
- How are in-place closure embedding reads resolved without inventing captures?
- How should the NLL/control-flow graph be represented?
- How are `return`, effect, and sync operations integrated into the shared
  semantic continuation?

Operator tokens/fixity/precedence/parse associativity are grammar facts; semantic
naked use selects operator[op], dot .op selects op::adl and explicit paths stay
explicit. OG_s's ordinary string-to-type construction, spelling extraction,
explicit Forget_s and current-slot selection are closed. Call, registered
extraction and generative invocation use ordinary relations. Source wiring
remains roadmap work; structured Path and ordinary ADL are now closed by their
topic owners.

Every legal completed closure expression returns full tau_C through ordinary
struct. File implementation-layer let installs under the established package
root, while true lexical local let remains binding. Conservative contribution roles preserve
legal ordinary actions and never retry failed execution. These are closed laws,
not a pending choice based on RHS type or declaration count.

## Generic navigation and Product surface

- Should intrinsic Product ordinal selectors be exposed as a user-visible
  tuple namespace API or remain structural navigation only?
- How does source code reference or replace a derived compile companion?
- Can default companion generation ever be suppressed, and what equivalent
  compile Pattern/contract must replace it?
- What finer-grained identity, if any, is needed for grouped inferred-require
  atoms?

### Structured Path representation and remaining surface scope

[Structured Path](../design/symbol-world/structured-path-algebra-and-pattern-splice.md)
now defines internal composition, ordinary extractable linked nodes, endpoint
shape, external Read, late textual roots, anchored value/reference roots,
Path-specific # and general Pattern splice $. A string creates one name node;
it does not parse source or resolve a target. Ordinary `.field -> field::adl`
is closed. Concrete representations and source wiring remain pending.

The exact public ordinal API and general quotation beyond the defined Path
domain remain open. Neither gap permits implicit sorting of an unordered
Product or opaque-only Path semantics.

NameCoord exists independently of realization. Finite generative rules can
match legal requested coordinates without enumerating an infinite Val2 or using
a universal name quantifier; concrete heads outrank wildcard heads by ordinary
specificity. Unordered same-name contributions share the coordinate and merge
their contribution effects, so no first-creator identity choice remains open.
Generated occurrences supply ordinary Val2 residency only, never registration
evidence.

The former Close/realization question is closed: ordinary generated Val2 results
are permitted; neither V_tau nor Pattern registration can come from a generated
occurrence. Close freezes non-generative registered structure, not all future
Val2 realization. V_tau callable values need no val::path resident; classifier
home is distinct. Current observations, prior immutable snapshots and no-reopen
checks follow the [name owner](../design/symbol-world/names-and-overload-groups.md#71-generated-val2-after-registered-structure-is-closed).
Representation of these effects is consumer work, not a second navigation ontology.

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


## Closure dependency lifetime refinement

[The lifetime handoff](../design/lifetime/lifetime-policy-and-overload-boundary.md#8-closure-dependency-lifetime-refinement-handoff)
owns further integration of dependency region/generation persistence,
move/copy/preserve, return/store/escape/promotion, bounded runtime state with
stable descriptors, and possible restricted first-class in-place uses.
FormationLegal, LifetimeLegal, Pre/Post, MoveEffect/Movable, EscapeLegal and
owned-transfer/promotion checks remain required. This does not block closure
formation semantics and grants neither global lifetime to tau_C nor a blanket
ban on every local dependency. Already established non-meta type survival
facts are retained in their domain.
