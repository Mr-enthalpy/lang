# Path Material at Strong Syntax Positions

**Status:** canonical Path and alias laws are closed; general consumers remain roadmap work.

Strong positions preserve [Path material](structured-path-algebra-and-interpretation-polarity.md).
Navigation runs in `name::path` direction with LHS/Pattern navigation inheritance.
An explicit `$` operand uses RHS expression interpretation before reinjection.
The frontend neither resolves a path nor fixes a terminal binding identity.

[Lexical alias](entity-alias-design.md) targets and `with` items use PathMaterialAst
and NormPathMaterial. These syntax carriers are neither language entities nor
identity proofs. Ordinary Path admissibility includes explicit roots, anchors
and dependencies. Other strong-context representation questions remain in planning;
they cannot restore an EntityRef-only alias grammar, terminal cache or second resolver.
