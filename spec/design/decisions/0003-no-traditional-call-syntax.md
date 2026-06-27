# ADR 0003: No Traditional Call Syntax

**Status:** Accepted

**Context:** Traditional `f(args)` call syntax conflicts with the language's
pipe-and-segment expression model. The project needed an explicit decision
for the parser.

**Decision:** `f(args)` is not a general function call. `(args)` is an
`ArgPack` with a segment-local role (`SourcePack`, `InsertPack`, or
`RightTargetSubsegment`). Traditional call syntax is not recognized by the
v0.1 parser.

**Consequences:**
- Expression parsing is based on `|>` segmentation and ArgPack role assignment,
  not on a precedence-based call grammar.
- Every `ArgPack` must appear within a `Segment`.
- Future semantic passes may interpret ArgPack roles as different call
  conventions (pipe, insert, method call, etc.).
