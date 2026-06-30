# PR86 Corrections Plan

## Step 1: Single Diagnostic Code

Replace `StatementAfterReturnEvent` and `StatementAfterBlockTailValue` with `StatementAfterTerminalBlockForm`.

**File:** `crates/lang_syntax/src/diagnostic.rs`
```rust
// Before:
    StatementAfterReturnEvent,
    StatementAfterBlockTailValue,

// After:
    StatementAfterTerminalBlockForm,
```

**File:** `crates/lang_syntax/src/dump.rs` — update `diagnostic_code_label` match to replace both old codes with the new one.

**File:** `crates/lang_syntax/src/parser/closure.rs` — update all uses of old codes to `StatementAfterTerminalBlockForm`. Messages distinguish:
- `"statement after block tail value"` for forms after an Expr terminal
- `"statement after return event"` for forms after a ReturnEvent terminal

**File:** `tests/cases/parser/return_after_return.diag` — update diagnostic code name in expected output.

---

## Step 2: Bare Tail Expr Terminality

**File:** `crates/lang_syntax/src/parser/closure.rs` — `parse_body_block`

Current: `seen_terminal = true` only for `ReturnEvent`.
Change: Also set `seen_terminal = true` for bare `FormAst::Expr`.

```rust
let form = parser.parse_form();
if matches!(&form, FormAst::ReturnEvent(_) | FormAst::Expr(_)) {
    seen_terminal = true;
}
forms.push(form);
```

Also remove the post-loop comment block that says "last form is implicitly a tail value" since terminality is now enforced during parsing.

**Semicolon behavior:** A `;` after a terminal form is consumed silently — it does NOT diagnose. Only actual FORMS after terminal are diagnosed. The existing semicolon-handling code in the loop already does this correctly (it just continues without checking `seen_terminal`). No change needed for semicolons.

Wait — the current code has:
```rust
if parser.cursor.consume_symbol(Symbol::Semicolon).is_some() {
    if seen_terminal {
        parser.error(DiagnosticCode::StatementAfterBlockTailValue, ...);
    }
    continue;
}
```

This diagnoses semicolons after a terminal. Per the new rules, semicolons should NOT diagnose. So REMOVE the `if seen_terminal` block from the semicolon handling.

Updated semicolon handling:
```rust
if parser.cursor.consume_symbol(Symbol::Semicolon).is_some() {
    continue;
}
```

**Golden test updates (3 tests):**

| Test | Change |
|------|--------|
| `closure_body_multi_form` | `false`→`true`, new `.diag`, update `.ast` to show Error form for `y` |
| `closure_body_semicolon_two_forms` | `false`→`true`, new `.diag`, update `.ast` to show Error form for `y` |
| `closure_inplace_body` | `false`→`true`, new `.diag`, update `.ast` to show Error form for `y` |

Expected `.diag` format:
```
error StatementAfterTerminalBlockForm "statement after block tail value" @ 1:11 [10..11]
```

Expected `.ast` for `{ x; y }`:
```
Program
  ExprForm
    Expr
      Pipe
        Segment has_incoming=false
          OperatorExpr
            Atom
              Closure InPlace
                BodyBlock
                  forms:
                    ExprForm          ← x is terminal (tail value)
                      Expr
                        Pipe
                          Segment has_incoming=false
                            OperatorExpr
                              Atom
                                Name x
                    Error "statement after block tail value"  ← y after terminal
```

**Also check `return_after_return` test:** The diag needs updating from `StatementAfterReturnEvent` to `StatementAfterTerminalBlockForm`.

---

## Step 3: Remove Self-Only Restriction for Explicit Return Targets

**File:** `crates/lang_syntax/src/parser/form.rs` — `extract_return_target_from_operator`

Remove lines 258-260:
```rust
// REMOVE:
if !element_is_name(first_elem, "Self") {
    return None;
}
```

Any expression as target is now accepted. `element_to_expr(first_elem)` handles arbitrary shapes.

---

## Step 4: Add `x (expr return)` Spelling (No `|>`)

**File:** `crates/lang_syntax/src/parser/form.rs` — `try_extract_explicit_return`

Current function only handles the `|>` pipe form (2 segments). Add a second path for the single-segment adjacency form:

```rust
fn try_extract_explicit_return(expr: &ExprAst) -> Option<ReturnEventAst> {
    let ExprKind::Pipe(pipe) = &expr.kind else { return None; };

    // Path 1: x |> (expr return)  — 2 segments
    if pipe.segments.len() == 2 {
        let lhs_seg = &pipe.segments[0];
        let rhs_seg = &pipe.segments[1];
        if rhs_seg.elements.len() != 1 { return None; }
        let rhs_elem = &rhs_seg.elements[0];
        if let Some((target, span)) = extract_return_target_from_segment_element(rhs_elem) {
            let value = segment_to_expr(lhs_seg);
            let span = value.span.join(span);
            return Some(ReturnEventAst { value: Box::new(value), target, span });
        }
        return None;
    }

    // Path 2: x (expr return)  — 1 segment with exactly 2 elements
    if pipe.segments.len() == 1 {
        let seg = &pipe.segments[0];
        if seg.elements.len() == 2 {
            let first = &seg.elements[0];   // value expression
            let second = &seg.elements[1];  // group containing return target
            if let Some((target, span)) = extract_return_target_from_segment_element(second) {
                let value = element_to_expr(first);
                let span = value.span.join(span);
                return Some(ReturnEventAst { value: Box::new(value), target, span });
            }
        }
        return None;
    }

    None
}
```

**New positive golden tests:**

| Test | Source | Expected |
|------|--------|----------|
| `return_explicit_adjacent` | `{ x (Self return); }` | `ReturnEvent Explicit(Self)` |
| `return_explicit_path` | `{ x \|> (Outer::Self return); }` | `ReturnEvent Explicit(Nav["Outer","Self"])` |

**Golden AST for `{ x (Self return); }`:**
```
Program
  ExprForm
    Expr
      Pipe
        Segment has_incoming=false
          OperatorExpr
            Atom
              Closure InPlace
                BodyBlock
                  forms:
                    ReturnEvent
                      value
                        Expr
                          Pipe
                            Segment has_incoming=false
                              OperatorExpr
                                Atom
                                  Name x
                      target
                        Explicit
                          Expr
                            Pipe
                              Segment has_incoming=false
                                OperatorExpr
                                  Atom
                                    Name Self
```

---

## Step 5: Context-Wide Return Embedding Rejection

Make `expression_contains_name` and its helpers visible outside `form.rs`.

**File:** `crates/lang_syntax/src/parser/form.rs`
- Add `pub(crate)` to: `expression_contains_name`, `segment_element_contains_name`, `operator_expr_contains_name`

Actually, a cleaner approach: move them to `expr.rs` since they operate on expression types.

**File:** `crates/lang_syntax/src/parser/expr.rs` — add:
```rust
pub fn expr_contains_name(expr: &ExprAst, name: &str) -> bool { ... }
fn segment_element_contains_name(el: &SegmentElementAst, name: &str) -> bool { ... }
fn operator_expr_contains_name(op: &OperatorExprAst, name: &str) -> bool { ... }
```

These are verbatim copies of the functions from `form.rs` (with appropriate imports).

**File:** `crates/lang_syntax/src/parser/form.rs` — replace the private helpers with calls to `super::expr::expr_contains_name(...)`.

**Call sites to add the check:**

| File | Function | Approx Line | After what expression |
|------|----------|-------------|----------------------|
| `let_stmt.rs` | `parse_let_value` | ~490 | After `parse_expr_until` for initializer |
| `let_stmt.rs` | `parse_binding_annotation` | ~448 | After right side of `_: expr` |
| `let_stmt.rs` | `parse_binding_annotation` | ~457 | After left side of `expr: expr` |
| `let_stmt.rs` | `parse_binding_annotation` | ~462 | After right side of `expr: expr` |
| `closure.rs` | `parse_delete_body` | ~232 | After delete message expr |
| `closure.rs` | `parse_fn_head_prefix` | ~304 | After fn item trait expr |
| `closure.rs` | `parse_head_clauses` | ~442 | After head clause body expr |
| `closure.rs` | `parse_capture_clause` | ~471 | After capture expr |
| `deduce.rs` | `parse_annotation_in_deduce` | ~130 | After deduce annotation expr |

At each site, add after the expression is parsed:
```rust
if expr_contains_name(&expr, "return") {
    parser.error(
        DiagnosticCode::ReturnExpressionNotAllowed,
        "return is only allowed as a block terminal form",
        expr.span,
    );
}
```

**New negative golden tests (8 tests):**

| Test | Source | Diag |
|------|--------|------|
| `return_in_let_rhs_bare` | `let y = (x return);` | ReturnExpressionNotAllowed |
| `return_in_let_rhs_pipe` | `let y = x \|> (Self return);` | ReturnExpressionNotAllowed |
| `return_in_let_rhs_adjacent` | `let y = x (Self return);` | ReturnExpressionNotAllowed |
| `return_call_arg_bare` | `{ f(x return); }` | ReturnExpressionNotAllowed |
| `return_call_arg_adjacent` | `{ f(x (Self return)); }` | ReturnExpressionNotAllowed |
| `return_in_group_bare` | `(x return)` | ReturnExpressionNotAllowed (already exists) |
| `return_in_group_adjacent` | `(x (Self return))` | ReturnExpressionNotAllowed |
| `return_in_operator` | `{ (x return) + y; }` | ReturnExpressionNotAllowed |
| `return_in_annotation` | `{ let y: (x return) = z; }` | ReturnExpressionNotAllowed |

**Note:** The `return_in_group_bare` test already exists as `return_in_group`. Update it or add the adjacent variant as a new test.

**Files to create per test (positive):**
- `.lang` — source file
- `.ast` — expected AST (placeholder, then capture actual)

**Files to create per test (negative):**
- `.lang` — source file
- `.ast` — expected AST (usually just wrapping the expression, with diag emitted)
- `.diag` — expected diagnostic output

**Register in `tests/parser_golden.rs`:** Add `assert_parser_case("test_name", true/false)` for each.

---

## Step 6: Full Change Summary

| Step | File(s) | Changes |
|------|---------|---------|
| 1 | `diagnostic.rs`, `dump.rs`, `closure.rs`, `return_after_return.diag` | Replace 2 codes with 1 |
| 2 | `closure.rs` | Set terminal for Expr, fix semicolon behavior |
| 2 | 3 `.ast` + 3 `.diag` + 3 test regs | Update existing tests |
| 3 | `form.rs` lines 258-260 | Remove Self-only check |
| 4 | `form.rs` `try_extract_explicit_return` | Add single-segment path |
| 4 | 2 `.lang` + 2 `.ast` + 2 test regs | New positive tests |
| 5 | `expr.rs` | Move `expr_contains_name` helpers |
| 5 | `form.rs` | Use moved helpers |
| 5 | `let_stmt.rs`, `closure.rs`, `deduce.rs` | Add checks at 8 call sites |
| 5 | 8 `.lang` + 8 `.ast` + 8 `.diag` + test regs | New negative tests |
