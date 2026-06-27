# decisions

**Status: Accepted architecture decision records (ADRs) / historical decisions.
They constrain design direction but do not override current `spec/public/`
behavior. Where an ADR and a public document appear to conflict, the public
document defines current behavior.**

These ADRs were relocated here from `docs/decisions/`. `docs/` is no longer the
authoritative entry point for decisions.

## Records and block mapping

| ADR | Constrains |
|---|---|
| `0001-v0.1-frontend-scope.md` | The frontend / public / history boundary. |
| `0002-closure-ast-not-object.md` | The frontend surface / closure-AST boundary (closures are AST, not objects). |
| `0003-no-traditional-call-syntax.md` | The call-syntax boundary for `meta-invocation/` and `mechanical-lowering/` (no traditional call syntax). |
| `0004-match-is-meta-function-not-syntax.md` | The `match`-is-not-syntax boundary for `patterns-overload/` and `meta-invocation/`. |
| `0005-closed-overload-sets.md` | The closed-overload-set boundary for `patterns-overload/` and `symbol-world/`. |
| `0006-build-system-track-stays-in-monorepo.md` | The build-system track boundary for `build-package/`. |

## Relationship to other blocks

ADRs are cross-cutting constraints. The design blocks must remain consistent
with these decisions; the blocks elaborate design within the boundaries the ADRs
set.
