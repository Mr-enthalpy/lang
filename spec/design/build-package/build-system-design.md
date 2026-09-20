# Compilation Roots and Physical Source Normalization

Status: canonical semantics; the current build substrate is pending alignment.

## 1. Compilation input

    L = Normalize(PhysicalTree(Level))
    K_entry = EntryContinuation(L, F_main)
    main.P2 = runtime
    omitted ordinary main.P1.stage = runtime
    Compile(Level) = Materialize(Pi_machine(Residual(E(K_entry))))

Before planner parameters are added, the compiler's semantic invocation selects
one compilation level. The compiler finds main.lang there as an explicit
compilation-root anchor. It is an ordinary sibling file after that selection,
not an execution-order authority.

Source actions, including legal ordinary meta invocations, determine how Objects are formed, how dependencies arise,
how external libraries are acquired and how target machines are described.
There is no second program-meaning input through a manifest, dependency list,
mount table, target flag, feature flag, package graph, include path or library
path. Future optimization options configure planner search without changing E.

## 2. Neutral physical normalization

    PhysicalTree(Level) -> normalized source actions
    NormalizeRoot(Level, b_root) = NormalizeBody(Level, b_root)
    NormalizeBody(D, b) = Unordered{
      NormalizeFile(f_i, b), NormalizeDir(n_j, D_j, b), ...
    }
    NormalizeFile(f, b) = Seq(decoded ordinary source actions of f under b)
    NormalizeDir(n, D, b) = Seq(
      n_expr := ordinary typed name formation let n::b:type;
      r_n := explicit ref of n_expr;
      r_n = ordinary one-shot empty directory type formation at n's coordinate;
      NormalizeBody(D, n_expr)
    )

Here b_root is the stable structural root supplied by bootstrap or an
independently legal ordinary meta formation. Its identity is not an active
meta frame around the entire compilation. The
[entry law](../meta-invocation/meta-object-invocation-and-policy-reduction.md#21-compilation-entry-and-root-formation)
separates root identity, actual stack dominance, main's runtime P2 and
construction authority.

Level selects physical input only. Normalization creates neither the root
nor OpenHere/Writable evidence. Name formation observes the current resident
type and resolved structural root identity, not Norm equality. Explicit
borrowing and actual writes separately require their ordinary authority.
Each meta/seal frame imposes dominance during its activity and restores its
enclosing context on return.

The selected level adds no extra name segment. Each child-directory basename n
becomes the selector of a generated ordinary structural let action. That action
uses typed NameExpr creation, explicit borrowing and ordinary initialization.
The directory's initial resident is formed by ordinary one-shot construction;
creation itself installs no dummy type. Only after initialization can its body
navigate and extend that resident. Omitted policy supplies no override; the
ordinary inherited/contextual policy and applicable default completion apply.
See [name realization](../symbol-world/names-and-overload-groups.md).

The generated action realizes NameCoord(StructuralRootIdentity(b),n); it does not manufacture that
coordinate's identity. Coordinates alone provide no Place or construction
permission. This normalization-generated syntax is not a generative meta-head
occurrence: the latter's Val2-only registration rule is a semantic occurrence
distinction, not a test of a frontend Generated provenance tag.

Evaluation of the directory body follows its creation in that directory's serial
wrapper; its child blocks share the post-initialization snapshot and initialized
name n_expr. This ordering
is internal to the wrapper and grants no priority over its parent's siblings.
Directory contents retain ordinary binding/action semantics. Only established
structural contribution roles synthesize the named type; explicit extend/inject
keeps its ordinary checks.
The initial ref r_n loses its initialization capability at commit. Later writes
require a separately applicable ordinary ref/write candidate, including direct
mut or explicit meta-to-mut confirmation for an initialized type. Physical
containment grants no write privilege.

Discovery emits these syntax-directed actions, not preinstalled namespace
nodes. A generated let must pass the same value-side OpenHere, selector,
non-retention and access/path/type checks as a source-written action. Child
borrowing and initialization independently check the existing one-shot Place
authority. A conflicting existing
name is an ordinary creation/write conflict; normalization cannot overwrite it,
choose a different target or merge it by directory privilege. Selector spelling
must be representable under ordinary name rules; otherwise normalization reports
a diagnostic rather than inventing a naming policy.

For a level containing main.lang, helpers.lang and math/vector.lang, both root
files target b_root, while the math wrapper realizes math under b_root and runs
vector.lang under the initialized math name. Neither filename adds a segment. The directory edge
has meaning only through this ordinary generated name action; the physical path
otherwise supplies provenance, not an additional owner or permission.

Each sibling block starts from the common input snapshot and evaluates into its
own overlay. The unordered join uses ordinary state-update algebra. A serial
scheduler is valid when it preserves this semantics; lexical sorting is useful
for diagnostics but cannot expose one sibling's writes to another sibling.

If A needs a value newly written by sibling B in the same block, sorting B
first does not make that read legal. Such a dependency would need a separately
specified explicit composition mechanism; the current unordered model does not
provide one.

## 3. Joining effects

Named-contribution positions synthesize the same named type's V_tau.
Associative/commutative contributions from different files can join; physical
provenance neither makes them exclusive nor merges distinct entries by value
equality. Conflicting replacements report an unordered-block write conflict.
Subtraction and other updates commute only where their ordinary algebra says so.

Sibling actions already established as contributions address the same
NameCoord(root,name). Equal coordinates do not turn ordinary actions into
contributions. If unretained
in the common snapshot, their accepted joined material forms the first resident
once by the ordinary one-shot rule. There is no first file that creates its
semantic identity. Explicit exclusive realization effects can still conflict.

Invocation-generated result Places follow these same rules, including the
associated-state member n_A(t). Stable invocation identity/cache reuse grants
no global ordering exception or permanent mutability. Current resident reads,
dependency-derived opening sources and ordinary write Pre remain observable.

## 4. Dependency projection

    DependencyGraph = Projection(EvaluationEffects)

A source host call that acquires an external Object creates the corresponding
observed dependency. The graph is useful afterward for diagnostics, cache
validation and scheduling known work; it does not choose namespace visibility,
available source or program meaning before evaluation.

link is a future host-backed ordinary callable. Its result is bound by ordinary
language actions. Neither link nor an engineering linker owns namespace
injection or a hidden package model. Its [root-acquisition effect law](../meta-invocation/host-capabilities-and-machine-objects.md)
uses E's compilation-wide LinkRegistry: active-root re-entry is a cycle,
already completed acquisition is a duplicate. Sibling overlays cannot hide
duplicate root-acquisition effects by joining them idempotently.

## 5. Engineering responsibilities

Infrastructure can discover files, decode sources, schedule evaluation, cache
results, report diagnostics and persist artifacts. It may implement semantic
actions but may not introduce semantic facts.

Stable paths, source hashes and content fingerprints support provenance and
cache validation. Cache reuse preserves resolved identities, effects, entry
multiplicity, host observations and required contextual checks. Cached execution
material can be parent-neutral; semantic roots still use the language's
MetaInstance identity rules. A cache hit does not grant construction authority.

Atomic storage supports an enclosing semantic transaction when one exists.
File boundaries and name-creation/initialization sequences do not independently create
transactions or rollback promises. Failed action Pre leaves the prior state
unchanged under the ordinary evaluator contract.

## 6. Frontend and implementation boundary

The weak lexer and syntax-directed parser/normalizer preserve source shapes;
they do not resolve packages, names, overloads or target machines. No
import/use/include/module syntax follows from physical normalization.

Current discovery and package/workspace carriers remain implementation material.
The connected build path still consumes configured roots and a package graph
and commits declarations into a shared world; it does not yet implement the
Level/main.lang anchor and common-snapshot sibling overlays specified here.
The [roadmap](../../planning/roadmap.md) records this migration rather than
granting those carriers normative status.
