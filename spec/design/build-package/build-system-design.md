# Compilation Roots and Physical Source Normalization

Status: canonical semantics; the current build substrate is pending alignment.

## 1. Compilation input

    Compile(Level)
      = Materialize(Residual(MetaEval(Normalize(PhysicalTree(Level)))))

Before planner parameters are added, the compiler's semantic invocation selects
one compilation level. The compiler finds main.lang there as an explicit
compilation-root anchor. It is an ordinary sibling file after that selection,
not an execution-order authority.

Source meta actions determine how Objects are formed, how dependencies arise,
how external libraries are acquired and how target machines are described.
There is no second program-meaning input through a manifest, dependency list,
mount table, target flag, feature flag, package graph, include path or library
path. Future optimization options configure planner search without changing E.

## 2. Neutral physical normalization

    PhysicalTree(Level) -> MetaProgram
    DirBlock = Unordered{FileBlock_i, ChildDir_j, ...}
    FileBlock_i = Seq(action_i1; action_i2; ...)

Directory/file discovery and source decoding establish physical blocks and
provenance. Source-level meta actions uniquely determine the resulting language
Objects. A physical path is not a construction permission or a namespace owner.
Implementation filenames do not become semantic identities by discovery.

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

Associated compile state A[t] follows exactly these same rules. Global
addressability introduces no global ordering exception and no permanent
mutability: its existing OpenHere/Writable premises govern each write.

## 4. Dependency projection

    DependencyGraph = Projection(EvaluationEffects)

A source host call that acquires an external Object creates the corresponding
observed dependency. The graph is useful afterward for diagnostics, cache
validation and scheduling known work; it does not choose namespace visibility,
available source or program meaning before evaluation.

link is a future host-backed ordinary callable. Its result is bound by ordinary
language actions. Neither link nor an engineering linker owns namespace
injection or a hidden package model.

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
File boundaries and structural let assignment do not independently create
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
