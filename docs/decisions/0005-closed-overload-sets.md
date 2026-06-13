# ADR 0005: Closed Overload Sets

**Status:** Accepted

**Context:** Languages with open namespaces (C++ via ADL, dynamic languages via
monkey-patching) allow external users to inject candidates into already visible
lookup scopes. This makes overload resolution non-local, complicates separate
compilation, and introduces name pollution.

**Decision:** Externally visible names have closed overload sets.

**Rules:**

- Namespace nodes have explicit construction origins. Their candidate set is
  fully determined at the point the namespace is constructed.
- Meta-function injection is parent-to-child only: a parent namespace may
  inject candidates when constructing child namespaces, but not retroactively.
- External users cannot add candidates to an already visible namespace node.
- Unqualified lookup cannot expand overload sets beyond the namespace node's
  closed candidate set.
- Name-polymorphic candidates must be explicitly declared inside the same
  closed namespace candidate structure. They are part of the closed set, not
  an escape hatch.
- Concrete candidates shadow name-polymorphic candidates.
- Concrete call failure reports the failure immediately. It does **not** fall
  back to abstract name-polymorphic lookup as a second chance.

**Consequences:**

- Overload resolution is locally decidable from the namespace node's
  construction site alone.
- C++-style ADL-like candidate injection is impossible by design.
- Dynamic-language-style name pollution is impossible by design.
- Name-polymorphic lookup, if implemented, operates on a closed, statically
  known candidate list — never on an open-ended set.
