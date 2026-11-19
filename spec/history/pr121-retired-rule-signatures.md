# PR #121 historical rule signatures

This archive records the comparison signatures formerly used by the audit
at `20c3379a3d975550310c3f5986f5bc66c7624df5`. It has no current semantic
authority and the current checker does not read or import it.

The early Pattern draft used a dedicated direct-arm association. It also
restricted masked matching to resource-preserving updates and described a
retirement-time transfer to a Forget receiver. The forward Heap draft moved
lazy release to allocation. The archive preserves the distinct signatures
used to detect remnants of those revisions, along with terminology used
before Pattern-local lifetime defaults were fixed.

```python
RETIRED_RULES = {
    "association token or node": r"(?<!~)~~(?!~)|\bAssocArm\b",
    "association heading": r"(?:direct[ -])?sum[ -]arm[ -](?:structural[ -])?association",
    "all replacements change arm": r"\bresource replacement requires a (?:real|true) arm change\b",
    "partial comparison only on Keep": r"\bpartial (?:Status )?match(?:ing)? (?:is )?(?:only|confined to) same-arm Keep\b",
    "escape at allocation": r"\bescape (?:fails|is rejected) at (?:production(?:/allocation)?|allocation) Pre\b",
    "read exit transfers ownership to Forget": (
        r"\bread(?: source)? Exit(?:Slot)? transfers? (?:the )?(?:sole|old) ownership to (?:lawful )?Forget\b"
    ),
    "retirement-time Drop registration": (
        r"\blogical end (?:must |may )?(?:forms?|transfers?) "
        r"(?:a (?:real )?)?(?:deferred )?Drop continuation\b"
        r"|逻辑结束动作交出值材料和析构责任，形成普通 drop continuation"
    ),
    "provisional lifetime default": r"\bprovisional\w*\s+`?lifetime default|`lifetime default E` provisionally",
}
```

## Earlier package manifest

The preserved package manifest describes its own 45-N/18-F/105-case evidence
scope; it is not the inventory or authority of the current specification.
Its 01_SPEC.md SHA-256 is
`f1e0a5d2dca964b1ddbe966643db74113592caf9ce6d96753d988238da835ab9`;
02_LOCAL_AGENT_TASK.md is
`3fd0c0b8cf31023dafa476eb25d22c6f12e9563af4329d451fc7e60803887cff`.
Current clause/case inventories are computed by the checker, independently of
that manifest. External instructions are not another specification owner.
