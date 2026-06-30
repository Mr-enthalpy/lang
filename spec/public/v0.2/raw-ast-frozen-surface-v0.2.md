# Raw AST Frozen Surface Inventory v0.2

## Purpose

This document is a structured inventory of the completed v0.2 Raw AST frozen
surface. It records every Raw AST construct family, its implemented shape,
frozen guarantee, non-semantic boundary, v0.3 normalization obligation, and
forbidden assumption. It is an enumerated handoff specification for `v0.3
Normalized AST Specification`.

This document is an inventory, not a design essay. It does not design the
Normalized AST node set. It does not introduce new syntax, parser behavior,
diagnostic rules, or golden snapshots.

## 1. Source normalization and span basis

| Field | Value |
|---|---|
| Raw construct family | Source normalization and span basis |
| AST / token shape | `Span { byte_start, byte_end, line, column }`; normalized LF source text |
| Implementation file(s) | `source.rs`, `span.rs`, `lexer.rs` |
| Spec source | `frontend-v0.1.md`; `raw-ast-contract-v0.1.md` |
| Frozen guarantee | All spans refer to the CRLF/LF-normalized source text. Line numbers and byte offsets are deterministic in the normalized representation. |
| Non-semantic boundary | Source normalization is mechanical (CRLF→LF, CR→LF). No source mapping, byte-to-character transcoding, or encoding-layer intelligence. |
| v0.3 obligation | Normalization may rely on span validity in the normalized source text. |
| Forbidden assumption | v0.3 must not assume spans map to original (pre-normalization) source, to character positions in a non-ASCII encoding, or to semantic entities. |

## 2. Lexer token categories

| Raw construct family | AST / token shape | Implementation file(s) | Spec source | Frozen guarantee | Non-semantic boundary | v0.3 obligation | Forbidden assumption |
|---|---|---|---|---|---|---|---|
| Name | `TokenKind::Name` | `token.rs`, `lexer.rs` | `ast-construction-v0.1.md` §2 | All identifiers, including semantic words (`return`, `else`, `match`, `fn`, `type`, etc.), are `Name` tokens. | The lexer does not classify any name as a keyword. | v0.3 may consume `Name` text as-is. | v0.3 must not assume name resolution or declaration binding from `Name` tokens alone. |
| IntLiteral | `TokenKind::IntLiteral`, `AtomKind::IntLiteral(String)` | `token.rs`, `lexer.rs`, `ast.rs` | `ast-construction-v0.1.md` §2, §8.1 | Exact source text preserved. Covers decimal, binary (`0b`/`0B`), octal (`0o`/`0O`), hex (`0x`/`0X`) with optional `'` separators. | The lexer preserves text but does not interpret the integer value. | v0.3 may extract source text. | v0.3 must not evaluate the integer value, perform overflow checking, or assign a type. |
| FloatLiteral | `TokenKind::FloatLiteral`, `AtomKind::FloatLiteral(String)` | `token.rs`, `lexer.rs`, `ast.rs` | `ast-construction-v0.1.md` §2, §8.1 | Exact source text preserved. Covers decimal scientific notation, leading/trailing-dot, and hex floats (`0x1p+4`). | The lexer preserves text but does not interpret the float value. | v0.3 may extract source text. | v0.3 must not evaluate the float value, normalize precision, or assign a type. |
| StringLiteral | `TokenKind::StringLiteral`, `AtomKind::StringLiteral(String)` | `token.rs`, `lexer.rs`, `ast.rs` | `ast-construction-v0.1.md` §2, §8.1 | Exact source text preserved including opening backslash boundary, body, and closing boundary. Ranked quote-boundary strings (`\"...\"`, `\\"...\\"`, etc.). | The lexer preserves text but does not decode escape sequences or interpret string contents. | v0.3 may extract source text. | v0.3 must not decode escape sequences, interpret character values, or concatenate adjacent string literals. |
| Symbol | `TokenKind::Symbol(Symbol)` (19 variants: `LParen`, `RParen`, `LBracket`, `RBracket`, `LBrace`, `RBrace`, `Comma`, `Colon`, `Equal`, `Dot`, `DotDot`, `ColonColon`, `PipeGreater`, `FatArrow`, `ThinArrow`, `Less`, `Greater`, `Semicolon`, `TripleEqual`) | `token.rs`, `lexer.rs` | `ast-construction-v0.1.md` §2 | Each structural symbol is a single token with maximal-munch priority. `TripleEqual` is a structural delimiter for alias-let at the lexer/form boundary. | The lexer distinguishes structural symbols from lexer-produced operator tokens. `Less`, `Greater`, and `TripleEqual` are bare symbols that the parser may reinterpret as operator spellings in expression contexts. | v0.3 may rely on symbol identity for structural parsing. | v0.3 must not treat parser-reinterpreted `TripleEqual` as semantic equality, alias resolution, or forwarding. |
| Operator | `TokenKind::Operator(OperatorSpelling)` (34 variants: 32 lexer-produced + contextual `TripleEqual` and `BracketCall`) | `token.rs`, `lexer.rs` | `operator-design.md` | 32 lexer-produced operator spellings recognized via maximal-munch. `TripleEqual` (`===`) is contextually reinterpreted from `Symbol::TripleEqual` in expression context. `BracketCall` (`[]`) is a contextual paired operator name (never produced by the lexer as a single token). | Operator spellings are syntax-level names. They do not imply built-in arithmetic, comparison, mutation, or lookup. | v0.3 may preserve operator spellings through normalization. | v0.3 must not perform operator lookup, overload resolution, ADL, or type-directed operator selection. |
| Trivia | `TokenKind::Trivia(TriviaKind)` (3 variants: `Whitespace`, `LineComment`, `BlockComment`) | `token.rs`, `lexer.rs` | `ast-construction-v0.1.md` §2 | Trivia tokens carry spans but are skipped by the parser. Block comments nest. | Trivia spans are preserved for diagnostic positioning. The parser discards trivia after consumption. | v0.3 may discard trivia. | v0.3 must not rely on trivia for semantic meaning, formatting reconstruction, or AST structure. |
| Invalid | `TokenKind::Invalid` | `token.rs`, `lexer.rs` | `ast-construction-v0.1.md` §2 | Produced for unrecognized byte sequences. Carries span and source text. | The lexer treats invalid bytes as opaque content. | v0.3 may preserve or discard `Invalid` tokens according to normalization rules. | v0.3 must not treat `Invalid` as a recoverable syntax node. |
| Eof | `TokenKind::Eof` | `token.rs`, `lexer.rs` | `ast-construction-v0.1.md` §2 | Always the final token emitted by the lexer. | End-of-input sentinel. | v0.3 may rely on `Eof` as the end of the token stream. | v0.3 must not consume tokens past `Eof`. |

## 3. Weak lexer rule

| Field | Value |
|---|---|
| Raw construct family | Weak lexer rule |
| AST / token shape | All lexer token categories (see §2) |
| Implementation file(s) | `lexer.rs` |
| Spec source | `AGENTS.md`; `ast-construction-v0.1.md` §2 |
| Frozen guarantee | The lexer does not classify any name as a keyword. Semantic words (`return`, `else`, `match`, `drop`, `move`, `sync`, `effect`, `fn`, `type`, `meta`, `runtime`, `compile`, `namespace`, `struct`, `guard`, `acquire`) are ordinary `Name` tokens. Parser contexts may recognize selected names structurally only where explicitly specified. |
| Non-semantic boundary | The lexer assigns no semantic roles. Semantic interpretation is exclusively a parser or later-stage responsibility. |
| v0.3 obligation | v0.3 may receive the token stream with all names as `Name`. |
| Forbidden assumption | v0.3 must not assume that any `Name` token has been pre-classified as a keyword, declaration, statement, or control-flow construct. |

## 4. Operator spellings

| Field | Value |
|---|---|
| Raw construct family | Operator spellings |
| AST / token shape | `TokenKind::Operator(OperatorSpelling)` (34 variants) |
| Implementation file(s) | `token.rs`, `lexer.rs` |
| Spec source | `operator-design.md` |
| Frozen guarantee | 32 lexer-produced operator spellings recognized via longest-match. `TripleEqual` (`===`) is `Symbol::TripleEqual` at the lexer/form boundary and may be reinterpreted as an expression operator spelling. `BracketCall` (`[]`) is a contextual paired operator name, never produced by the lexer as a single token, used as the operator identity in bracket-call sugar and bindable/aliasable in operator-name positions. |
| Non-semantic boundary | Operator spellings are syntax-level names. They do not imply built-in arithmetic, comparison, mutation, assignment, ADL, or type-directed lookup. |
| v0.3 obligation | v0.3 may preserve operator spellings through normalization. Operator sugar must be desugared to named operator calls during normalization. Prefix-negative (`-x`) must be normalized to typed-zero binary subtraction, not resolved as a prefix operator declaration. |
| Forbidden assumption | v0.3 must not perform operator lookup, overload resolution, or ADL. v0.3 must not treat parser-reinterpreted `TripleEqual` as semantic equality, alias resolution, or forwarding. |

## 5. Program / form boundary

| Field | Value |
|---|---|
| Raw construct family | Program and form boundary |
| AST / token shape | `ProgramAst { forms: Vec<FormAst> }`; hard-only form boundaries: `;`, `}`, EOF |
| Implementation file(s) | `parser/form.rs`, `parser/cursor.rs` |
| Spec source | `ast-construction-v0.1.md` §3 |
| Frozen guarantee | Forms are separated by hard boundaries only: `;`, `}`, EOF. A line break (`\n`) is trivia and is never promoted to a form separator. `ProgramAst.forms` preserves source order. |
| Non-semantic boundary | The parser does not reorder forms and does not decide whether a form is a declaration, statement, or expression at a semantic level. |
| v0.3 obligation | v0.3 may rely on form order and hard boundary separation. |
| Forbidden assumption | v0.3 must not assume line breaks imply form boundaries. v0.3 must not assume every form is semantically independent (forms may be syntactically adjacent in the same scope). |

## 6. FormAst family

| Raw construct family | AST / token shape | Implementation file(s) | Spec source | Frozen guarantee | Non-semantic boundary | v0.3 obligation | Forbidden assumption |
|---|---|---|---|---|---|---|---|
| FormAst | `FormAst::Let(LetAst)`, `FormAst::AliasLet(LetAliasLet)`, `FormAst::Expr(ExprAst)`, `FormAst::ReturnEvent(ReturnEventAst)`, `FormAst::Error(ErrorAst)` | `ast.rs`, `parser/form.rs`, `parser/let_stmt.rs` | `ast-construction-v0.1.md` §3 | Five form variants: `Let` (ordinary/extract bindings), `AliasLet` (alias bindings), `Expr` (expression forms), `ReturnEvent` (return terminal events — structurally preserved, target unresolved, recognized contextually by parser in return terminal form positions), `Error` (recovery markers). | The parser preserves form shape but does not decide semantic meaning. | v0.3 must unify `Let` and `AliasLet` into a common normalized declaration form. | v0.3 must not assume forms have been validated for scope, visibility, or declaration semantics. |

## 7. Binding slot family

| Field | Value |
|---|---|
| Raw construct family | Binding slot |
| AST / token shape | `BindingSlotAst { policy: Option<ExprAst>, has_let: bool, deduce: Option<DeduceListAst>, pattern: BindingPatternAst, annotation: Option<BindingAnnotationAst>, with_clause: Option<WithClauseAst>, initializer: Option<ExprAst>, span: Span }` |
| Implementation file(s) | `ast.rs`, `parser/let_stmt.rs`, `parser/closure.rs`, `parser/deduce.rs`, `parser/canonical.rs` |
| Spec source | `ast-construction-v0.1.md` §4 |
| Frozen guarantee | The binding slot shape is reused across let bindings, parameter slots, and return slots. Optional `policy` is recognized only by the `Expr let` syntactic shape; `policy = None` means unwritten (implicit/inferred later). Initializer is required only in let-binding context. `with { ... }` is rejected in return slots. |
| Non-semantic boundary | The parser preserves the slot shape but performs no policy validation, annotation type-checking, or `with` dependency/lifetime analysis. |
| v0.3 obligation | v0.3 must normalize binding slots into a unified declaration/pattern shape. Optional `with` clauses must be preserved. |
| Forbidden assumption | v0.3 must not assume the policy expression denotes a valid accessibility/visibility/capability condition. v0.3 must not assume the annotation denotes a valid type, rank, or classifier. |

## 8. Binding pattern family

| Field | Value |
|---|---|
| Raw construct family | Binding pattern |
| AST / token shape | `BindingPatternAst::Binder(BinderNameAst)`, `BindingPatternAst::Product(ProductExtractAst)`, `BindingPatternAst::Skeleton(CanonicalSkeletonAst)`, `BindingPatternAst::Error(ErrorAst)` |
| Implementation file(s) | `ast.rs`, `parser/let_stmt.rs`, `parser/canonical.rs` |
| Spec source | `ast-construction-v0.1.md` §4 |
| Frozen guarantee | Four pattern variants: `Binder` (simple name or operator binder), `Product` (product extraction in binding context), `Skeleton` (canonical skeleton pattern), `Error` (recovery). |
| Non-semantic boundary | The parser preserves pattern shape but performs no matching, extractability validation, or semantic pattern analysis. |
| v0.3 obligation | v0.3 must normalize all patterns into a unified normalized pattern form. |
| Forbidden assumption | v0.3 must not assume patterns have been validated for admissibility, extractability, or binder uniqueness. |

## 9. With clause

| Field | Value |
|---|---|
| Raw construct family | With clause |
| AST / token shape | `WithClauseAst { kind: WithClauseKind, span: Span }`; `WithClauseKind::Empty`, `WithClauseKind::Items { items: Vec<NameAst> }`, `WithClauseKind::Error(ErrorAst)` |
| Implementation file(s) | `ast.rs`, `parser/let_stmt.rs` |
| Spec source | `ast-construction-v0.1.md` §4.2 |
| Frozen guarantee | `with {}` is explicit empty. Non-empty `with { ... }` accepts only comma-separated `Name` items. No expressions, paths, operator names, or EntityRef syntax. Trailing commas are rejected. Absence of a with clause is distinct from `with {}`. Malformed `with` must not produce `WithClauseKind::Empty`. |
| Non-semantic boundary | The parser does not resolve `with` names or run dependency/lifetime/ownership analysis. |
| v0.3 obligation | v0.3 must preserve the `with` payload through normalization. |
| Forbidden assumption | v0.3 must not assume `with` names denote valid same-level bindings, dependencies, lifetimes, or ordering constraints. |

## 10. Product expression and product extraction

| Field | Value |
|---|---|
| Raw construct family | Product (expression) and product extraction (binding) |
| AST / token shape | `ProductExprAst { elements: Vec<ProductElementAst>, span }`; `ProductExtractAst { elements: Vec<ProductExtractElementAst>, span }`; `ProductElementAst::Expr | Unit`; `ProductExtractElementAst::Slot | Unit` |
| Implementation file(s) | `ast.rs`, `parser/product.rs`, `parser/let_stmt.rs`, `parser/closure.rs` |
| Spec source | `ast-construction-v0.1.md` §9 |
| Frozen guarantee | Parenthesized forms with top-level commas are product forms. In expression context: product construction (`ProductExprAst`). In binding/extraction context: product extraction (`ProductExtractAst`). Leading/doubled/trailing commas produce explicit `Unit` elements (not omitted, not wildcards). No `ArgPack`, `SourcePack`, `InsertPack`, or `RightTargetSubsegment` role enum exists. |
| Non-semantic boundary | The parser does not assign call/application roles to product elements and does not decide whether a product is constructible, destructible, layout-compatible, or callable. |
| v0.3 obligation | v0.3 must normalize product placement into a unified call structure. Unit elements must be preserved, not discarded. |
| Forbidden assumption | v0.3 must not assume product elements have source/insert/right-target roles. v0.3 must not assume unit elements are implicit discards. |

## 11. Pipe expression / segment architecture

| Field | Value |
|---|---|
| Raw construct family | Pipe expression and segment |
| AST / token shape | `PipeExprAst { segments: Vec<SegmentAst>, span }`; `SegmentAst { elements: Vec<SegmentElementAst>, has_incoming: bool, span }`; `SegmentElementAst::OperatorExpr | Product` |
| Implementation file(s) | `ast.rs`, `parser/pipe.rs`, `parser/product.rs`, `parser/operator.rs` |
| Spec source | `ast-construction-v0.1.md` §7 |
| Frozen guarantee | `|>` is the outer expression skeleton. Each segment contains operator expressions and product elements. `has_incoming` indicates whether a prior segment exists. Operators bind tighter than whitespace auto-pipe and `|>`. Operator precedence is segment-local. |
| Non-semantic boundary | The parser preserves pipe/segment structure as Raw AST. It does not interpret segments as function-application chains. |
| v0.3 obligation | v0.3 must flatten pipe segments into a unified normalized call form. |
| Forbidden assumption | v0.3 must not assume pipe segments denote function application, monadic bind, or control-flow sequencing. v0.3 must not assume operator expressions within segments have been desugared. |

## 12. Pipe branch-name shorthand

| Field | Value |
|---|---|
| Raw construct family | Pipe branch-name shorthand |
| AST / token shape | Same as explicit form: incoming segment with two-element product head (`_`, `name`) + in-place closure body |
| Implementation file(s) | `parser/pipe.rs` |
| Spec source | `ast-construction-v0.1.md` §7.1.1 |
| Frozen guarantee | `|> name { ... }` is accepted only as a mechanical shorthand for `|> (_ name) { ... }`. Recognizes only the exact local prefix `|> name { ... }`. The branch-name token may be `_`. No wildcard, unit, or pattern semantics attach to either `_` or `name`. Following tokens remain ordinary segment material. Not a precedent for branch-arm sugar families. |
| Non-semantic boundary | The parser performs no matching, lookup, closure materialization, or semantic validation on the shorthand. |
| v0.3 obligation | v0.3 must receive the explicit-form Raw AST shape. No special-case handling is needed. |
| Forbidden assumption | v0.3 must not generalize the shorthand into a branch-arm sugar system. v0.3 must not treat `_` or `name` as carrying pattern/semantic meaning from the shorthand alone. |

## 13. OperatorExpr

| Field | Value |
|---|---|
| Raw construct family | Operator expression |
| AST / token shape | `OperatorExprAst { kind: OperatorExprKind, span }`; `OperatorExprKind::Atom | Product | OperatorSugar | NavPath | MemberSugar | DoubleDotSugar | BracketCallSugar | Error` |
| Implementation file(s) | `ast.rs`, `parser/operator.rs`, `parser/atom.rs` |
| Spec source | `ast-construction-v0.1.md` §7.3, §8.4a; `operator-design.md` |
| Frozen guarantee | Operator sugar is preserved as Raw AST surface markers. Prefix-negative (`-x`) is the sole `Prefix` fixity shape and is not an overloadable operator declaration. Postfix and binary operators are preserved with `Postfix`/`Binary` fixity. Comparison, equality, and equals-suffixed chains are non-associative and produce `ChainedNonAssociativeOperator`. |
| Non-semantic boundary | The parser does not lower operator sugar to ordinary calls. It does not perform operator lookup, type-directed resolution, overload resolution, or semantics assignment. |
| v0.3 obligation | v0.3 must desugar all operator sugar to named operator calls. Prefix-negative must be normalized to typed-zero binary subtraction; exact normalized representation is specified in v0.3. |
| Forbidden assumption | v0.3 must not treat `Prefix` fixity as an overloadable operator fixity. v0.3 must not resolve operator identities to declarations. |

## 14. Atom suffixes

| Raw construct family | AST / token shape | Implementation file(s) | Spec source | Frozen guarantee | Non-semantic boundary | v0.3 obligation | Forbidden assumption |
|---|---|---|---|---|---|---|---|
| `::` navigation | `NavPath { components: Vec<NavComponentAst> }` (atom layer and operator-expr layer) | `ast.rs`, `parser/atom.rs`, `parser/operator.rs` | `ast-construction-v0.1.md` §8.4 | Inner-to-outer source-order navigation. Innermost component must be `Name` or `OperatorName`. Outer components may be `Name` or `Group(ExprAst)`. Operator names are valid only as innermost components. Grouped expressions are valid only as outer components. | The parser preserves navigation components in source order and performs no lookup. | v0.3 must preserve navigation order through normalization. | v0.3 must not resolve navigation components to declarations, scopes, or namespaces. |
| `.` member sugar | `MemberSugar { object, selector: SelectorAst }` | `ast.rs`, `parser/atom.rs`, `parser/operator.rs` | `ast-construction-v0.1.md` §8.5 | Selector is `Text(NameAst)` only. Numeric selectors removed. Invalid selectors produce `ExpectedNameAfterDot`. | The parser preserves member shape but performs no field lookup, access semantics, or offset calculation. | v0.3 must desugar member sugar into a normalized member-access form. | v0.3 must not assume the selector resolves to a field, method, or accessor. |
| `..` double-dot sugar | `DoubleDotSugar { object, selector: SelectorAst, args: ProductExprAst }` | `ast.rs`, `parser/atom.rs`, `parser/operator.rs` | `ast-construction-v0.1.md` §8.6 | Requires selector (`Name`) followed by product form. Selector is `Text(NameAst)` only. Invalid selectors produce `ExpectedNameAfterDoubleDot`. Missing product produces `ExpectedProductAfterDoubleDotName`. | The parser preserves double-dot shape but performs no method resolution or dispatch. | v0.3 must desugar double-dot sugar into a normalized method-call form. | v0.3 must not assume the selector resolves to a method or that the product form denotes arguments. |
| `obj[args]` bracket-call sugar | `BracketCallSugar { object, operator: OperatorNameAst(spelling="[]"), args: ProductExprAst }` | `ast.rs`, `parser/atom.rs`, `parser/operator.rs`, `parser/product.rs` | `ast-construction-v0.1.md` §8.7 | Source-preserving. Left-associative. `obj[]` is valid (empty args). Operator spelling `[]`. No indexing/slicing/container semantics. | The parser preserves bracket-call shape. `[]` is a contextual paired operator name, bindable/aliasable in operator-name positions. | v0.3 must desugar bracket-call sugar into a named `[]` operator call: `(object, args...) |> []`. | v0.3 must not assume bracket-call implies indexing, slicing, bounds checking, or container access. |

## 15. Closure AST family

| Field | Value |
|---|---|
| Raw construct family | Closure AST |
| AST / token shape | `ClosureAst::InPlace(InPlaceClosureAst)`, `ClosureAst::Explicit(ExplicitClosureAst)` |
| Implementation file(s) | `ast.rs`, `parser/closure.rs` |
| Spec source | `ast-construction-v0.1.md` §10 |
| Frozen guarantee | `InPlace`: bare `{ ... }` in atom position. No capture clause, no parameter clause, no return clause, no head clauses. It is not a normal block expression. Having no extraction head means no extracted input, including no implicit unit input. `Explicit`: `FnHeadPrefix => ClosureBodyAst`. `ClosureBodyAst` is either `Block(BodyBlockAst)` or `Delete(DeleteBodyAst)`. `DeleteBodyAst` is `(message_expr) delete` after `=>`. Headed closures without `=>` are rejected (`InvalidClosureHead`). Closure literals produce AST, not callable objects. |
| Frozen guarantee (v0.8 amendment) | `ExplicitClosureAst.body` changed from `BodyBlockAst` to `ClosureBodyAst` to accommodate the `=> (message) delete` form. `DeleteBodyAst` is accepted only as `(message_expr) delete` after `FatArrow`. The `( ... )` group is required — bare `=> "msg" delete` is rejected. `InPlaceClosureAst` is unaffected. |
| Non-semantic boundary | The parser produces closure AST only. Closure materialization into callable objects is a future semantic pass. |
| v0.3 obligation | v0.3 must preserve closure AST structure through normalization. The headless/headed distinction must be preserved. |
| Forbidden assumption | v0.3 must not materialize closures into callable objects. v0.3 must not assume a headless in-place closure implicitly accepts unit input. |

## 16. Closure head clauses

| Field | Value |
|---|---|
| Raw construct family | Closure head |
| AST / token shape | `FnHeadPrefixAst { deduce: Option<DeduceListAst>, captures: Option<CaptureClauseAst>, params: Option<ParamClauseAst>, fn_item_trait: Option<ExprAst>, returns: Option<ReturnClauseAst>, clauses: Vec<HeadClauseAst>, span }` |
| Implementation file(s) | `ast.rs`, `parser/closure.rs`, `parser/deduce.rs`, `parser/canonical.rs` |
| Spec source | `ast-construction-v0.1.md` §11 |
| Frozen guarantee | Fixed clause order: deduce list, capture clause, parameter clause, trait clause, return clause, head clause tail. `CaptureItemAst` stores full `ExprAst` (not name-only). `ParamClause` is one `ProductExtractAst`. `ReturnClause` is one `BindingSlotAst`. Active head clauses: `Require`, `Pre`, `Post`, `LifetimePre`, `LifetimePost` (each holds one `ExprAst`). `acquire` is an ordinary name. |
| Non-semantic boundary | The parser preserves clause shape but performs no capture validation (move/ref/copy), parameter type-checking, return-type checking, contract/lifetime/resource validation, or rank/type/predicate checking. |
| v0.3 obligation | v0.3 must preserve clause order and shape through normalization. |
| Forbidden assumption | v0.3 must not interpret head clauses as semantic contracts, lifetime conditions, resource conditions, type-level objects, rank-level objects, or predicates. |

## 17. Canonical skeletons and deduce lists

| Field | Value |
|---|---|
| Raw construct family | Canonical skeleton and deduce list |
| AST / token shape | `CanonicalSkeletonAst::Segment | ProductExtract | Wildcard | Name(role) | NavPath | Literal | Error`; `DeduceListAst { binders: Vec<BinderDeclAst> }`; `CanonicalNameRole::Hole | NodeName | Unknown` |
| Implementation file(s) | `ast.rs`, `parser/canonical.rs`, `parser/deduce.rs` |
| Spec source | `ast-construction-v0.1.md` §5, §6 |
| Frozen guarantee | Parser-preserved only. `Hole`/`NodeName`/`Unknown` are parse-time role markers. No matching, admissibility checking, type/value interpretation, or canonical evaluation is performed. All canonical skeleton golden tests are parser preservation tests. |
| Non-semantic boundary | The parser builds skeleton AST but does not execute matching. The `Hole`/`NodeName` distinction is syntactic, not semantic. |
| v0.3 obligation | v0.3 must desugar skeletons and deduce lists into normalized pattern forms. Hole/NodeName roles must be preserved. |
| Forbidden assumption | v0.3 must not assume skeletons are admissible, well-formed, or semantically meaningful. v0.3 must not resolve holes to types or values. |

## 18. Alias-let and EntityRef (alias RHS subset)

| Field | Value |
|---|---|
| Raw construct family | Alias binding and EntityRef |
| AST / token shape | `LetAliasAst { policy: Option<ExprAst>, binder: AliasBinderAst, target: EntityRefAst, span }`; `AliasBinderAst::Name | Operator | Error`; `EntityRefAst { components: Vec<NavComponentAst>, span }` |
| Implementation file(s) | `ast.rs`, `parser/let_stmt.rs`, `parser/form.rs` |
| Spec source | `ast-construction-v0.1.md` §16; `entity-alias-design.md`; `entity-ref-design.md` |
| Frozen guarantee | `let binder === EntityRef` is a form-level construct. `EntityRef` is parsed only inside alias-let RHS (not a general expression parser mode). Optional `policy` prefix preserved. Alias binding must be bounded by `;`, `}`, or EOF. `with { ... }` is not accepted in alias binding. |
| Non-semantic boundary | The parser preserves alias shape but performs no target resolution, name lookup, operator identity validation (`spelling + fixity + arity`), namespace resolution, or alias semantics. |
| v0.3 obligation | v0.3 must preserve `EntityRefAst` as an unresolved entity reference in normalized alias declarations. |
| Forbidden assumption | v0.3 must not resolve `EntityRef` to a semantic entity, namespace, or declaration. v0.3 must not validate operator alias identity. |

## 19. Diagnostics

| Raw construct family | AST / token shape | Implementation file(s) | Spec source | Frozen guarantee |
|---|---|---|---|---|
| Lexer diagnostics (4) | `DiagnosticCode::InvalidToken`, `UnclosedString`, `UnclosedComment`, `InvalidNumericLiteral` | `diagnostic.rs`, `lexer.rs` | `diagnostics-v0.1.md` §3.1 | Guaranteed: emitted when the lexer encounters invalid byte sequences, unclosed strings/comments, or malformed numeric literals. Every diagnostic carries a `Span`. |
| Parser diagnostics (17) | `DiagnosticCode::UnexpectedToken`, `ExpectedName`, `ExpectedColon`, `ExpectedBindingAnnotation`, `ExpectedEqual`, `EmptyPipeSegment`, `ExpectedNameAfterDot`, `ExpectedNameAfterDoubleDot`, `ExpectedProductAfterDoubleDotName`, `UnclosedParen`, `UnclosedBracket`, `UnclosedBrace`, `InvalidDeduceList`, `InvalidCanonicalSkeleton`, `InvalidClosureHead`, `TopLevelComma`, `UnusedClosureAst` | `diagnostic.rs`, `parser/*.rs` | `diagnostics-v0.1.md` §3.2 | 16 guaranteed-emitted + 1 optional: `UnusedClosureAst` is optional/not-guaranteed-emitted. |
| Return diagnostics (3) | `DiagnosticCode::ReturnRequiresValue`, `StatementAfterTerminalBlockForm`, `ReturnExpressionNotAllowed` | `diagnostic.rs`, `parser/form.rs` | `diagnostics-v0.1.md` §3.3 | Guaranteed: emitted for malformed return events — return without a value expression, statements placed after a terminal block form, and return expressions in non-return positions. |
| Operator diagnostics (3) | `DiagnosticCode::InvalidOperatorExpression`, `ChainedNonAssociativeOperator`, `InvalidNavComponent` | `diagnostic.rs`, `parser/operator.rs`, `parser/atom.rs` | `diagnostics-v0.1.md` §3.4 | Guaranteed: emitted for malformed operator expressions, non-associative chains, and invalid navigation components. |
| Alias diagnostics (5) | `DiagnosticCode::ExpectedAliasTarget`, `InvalidAliasBinder`, `InvalidAliasPosition`, `InvalidEntityRef`, `UnexpectedAliasRhsExpression` | `diagnostic.rs`, `parser/let_stmt.rs`, `parser/form.rs` | `diagnostics-v0.1.md` §3.5 | 4 guaranteed-emitted + 1 reserved: `InvalidAliasBinder` is reserved and not currently emitted by the parser. |

| Field | Value |
|---|---|
| Frozen guarantee (general) | 32 `DiagnosticCode` variants (Lexer: 4, Parser: 17, Return: 3, Operator: 3, Alias: 5, Total: 32). Every diagnostic carries a `DiagnosticCode`, `message`, and `Span`. The parser is error-tolerant: it inserts `ErrorAst` nodes alongside diagnostics and continues parsing. Three special statuses exist: (a) guaranteed emitted, (b) optional/not-guaranteed-emitted (`UnusedClosureAst`), (c) reserved/not-currently-emitted (`InvalidAliasBinder`). |
| Non-semantic boundary | Diagnostics cover lexer and parser errors only. No type errors, kind errors, lifetime errors, or semantic warnings. |
| v0.3 obligation | v0.3 must preserve or rewire diagnostic spans through normalization. `ErrorAst` nodes and `Diagnostic` entries carry sufficient information for rewiring. |
| Forbidden assumption | v0.3 must not mutate the frozen Raw AST diagnostic code set; any normalization-stage diagnostics require an explicit v0.3 diagnostic spec. v0.3 must not assume optional diagnostics are guaranteed. v0.3 must not activate reserved diagnostics. |

## 20. Dumps and golden snapshots

| Field | Value |
|---|---|
| Raw construct family | Token dump, AST dump, diagnostic dump; golden test snapshots |
| AST / token shape | Hand-written stable text formats (not Rust `Debug`) |
| Implementation file(s) | `dump.rs`; `tests/lexer_golden.rs`, `tests/parser_golden.rs`, `tests/diagnostics_golden.rs`; `tests/cases/` |
| Spec source | `frontend-v0.1.md`; `implementation-status-v0.1.md` |
| Frozen guarantee | Three stable, dumpable outputs: token dump, AST dump, diagnostic dump. 25 lexer golden cases, 299 parser golden cases, 43 diagnostic golden cases. Dump formats are hand-written and stable — not Rust `Debug` output. Golden snapshots define externally visible frontend behavior. |
| Non-semantic boundary | Dumps are diagnostic/visibility artifacts. They do not constitute a semantic output of the compiler. |
| v0.3 obligation | v0.3 may rely on dump formats for Normalized AST golden testing. Golden test infrastructure must remain compatible. |
| Forbidden assumption | v0.3 must not change existing dump outputs or golden snapshots without explicit documentation of the change. |

## Cross-references

- `spec/contracts/raw-ast-contract-v0.1.md` — Raw AST invariants for normalization
- `spec/contracts/raw-ast-contract-freeze-v0.2.md` — v0.2 freeze boundary and allowed/forbidden work
- `spec/implementation/v0.1/ast-construction-v0.1.md` — normative syntax rules
- `spec/history/v0.1/operator-design.md` — normative operator design
- `spec/implementation/v0.1/diagnostics-v0.1.md` — normative diagnostic catalog
- `spec/implementation/v0.1/implementation-status-v0.1.md` — authoritative factual inventory
- `spec/planning/roadmap.md` — stage model and scope boundaries
