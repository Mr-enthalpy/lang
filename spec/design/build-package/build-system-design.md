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

Source actions, including legal ordinary compile invocations, determine how Objects are formed, how dependencies arise,
how external libraries are acquired and how target machines are described.
There is no second program-meaning input through a manifest, dependency list,
mount table, target flag, feature flag, package graph, include path or library
path. Future optimization options configure planner search without changing E.

## 2. Neutral physical normalization

```text
PhysicalTree(Level) -> normalized source actions
NormalizeRoot(Level,p) = NormalizeBody(Level,p)
NormalizeBody(D,p) = Unordered{NormalizeFile(f_i,p), NormalizeDir(n_j,D_j,p), ...}
NormalizeFile(f,p) = Seq(ordinary source actions under NavigationContext(p))
NormalizeDir(n,D,p) = NormalizeBody(D,n::p)

NavigationContext(a) |- inner => inner::a
a/ { P let inner = rhs; } => P let inner::a = rhs;
```

The selected root and filenames add no name segment. Child directory names
supply inherited navigation, preserved in the same canonical name::path
material as written qualified declarations. Discovery and normalization create
no semantic names, Places, Objects, empty type residents or authority facts.

The ordinary let consumer handles each qualified declaration. Initializer-bearing
bindings infer the RHS type and perform ordinary complete binding/installation;
initializer-free typed names use the established name-formation, explicit
borrow and one-shot initialization rules. Neither form obtains authority from
physical containment. Required intermediate navigation and target identity are
checked by the ordinary Path/name consumer.

For a level containing main.lang, helpers.lang and math/vector.lang, both root
files retain the root navigation context and vector.lang inherits math. A let
inner=bool:: in vector.lang therefore targets inner::math with RHS bool:: at
the same NameExpr level. It performs ordinary name installation, not type
composition or structural expression evaluation.

Each file preserves source order. Sibling blocks start from one common snapshot,
evaluate independent overlays and join ordinary effects. Ordering filenames for
diagnostics supplies no execution priority and cannot expose sibling new writes.
An unavailable target, illegal access or conflicting realization diagnoses under
ordinary semantics rather than being repaired by physical namespace allocation.

Main has runtime P2; root identity and active body evaluation are independent.
CompileInstance formation remains compatible with seal P2. Every action checks
actual OpenHere, Writable, capability, access, lifetime and continuation facts
where required, and publishes only through the common semantic transaction.

## 3. Joining effects

File implementation-layer `let name = rhs` installs the evaluated material at
the established package structural root. A true lexical local let remains an
ordinary binding; filenames introduce no owner. Every legal completed closure
expression yields full tau_C through atomic structural type formation. Direct and legal
incremental construction use the same material algebra. The
[source composition owner](../symbol-world/symbol-construction-units-and-namespace-origin.md)
defines this installation boundary.


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
injection or a hidden package model. Its [root-acquisition effect law](../static-evaluation/host-capabilities-and-machine-objects.md)
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
CompileInstance identity rules. A cache hit does not grant construction authority.

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
