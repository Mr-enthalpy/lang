# Bounded-model evidence

This directory preserves the handoff's Python model and expected stdout without
editing. The separately authored [audit script](audit_consolidation.py) checks
document links/anchors, N/M/U/F and case inventories, both original hashes and
the model's actual output. Run from a checkout with Python 3:

```text
python spec/planning/evidence/lifetime-concurrency/audit_consolidation.py
```

The audit has an explicit document scope and needs neither an uncommitted diff
nor the original downloaded ZIP. It is auxiliary evidence, not lang syntax,
a source consumer, an ISA
model or normative ontology. Its old explanatory implementation-language words
(including atomic enum) define no lang Pattern or declaration. Execution and
SHA-256 checks are recorded in [the ledger](../../lifetime-concurrency-consolidation.md).
The model covers at most three ARC tickets/two actors and simplified MP/SB.
It does not prove full CAS resource semantics, Region/Color, repair, HeapStack,
no-thin-air, overflow/Weak/reclamation, fence/ISA or compiler preservation.
