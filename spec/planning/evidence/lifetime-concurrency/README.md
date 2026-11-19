# Bounded-model evidence

This directory preserves the handoff's Python model and expected stdout without
editing. The separately authored [audit script](audit_consolidation.py) checks
all tracked current Markdown for links and table/fence structure. It checks
shared owner links, the definition and consumer status ledger, stable N/M/U/F
inventories, 131 paired cases, AS01–AS14 and BT01–BT05,
and the preserved model hashes and actual output. Run from a checkout with Python 3:

```text
python spec/planning/evidence/lifetime-concurrency/audit_consolidation.py
```

The [CI workflow](../../../../.github/workflows/ci.yml) runs this same command in
the independent `Specification and evidence audit` job on pull requests, pushes
to main and manual dispatch. It installs Python 3.13 and needs no third-party
Python packages. A link/inventory, preserved-hash or model-output mismatch fails
the job. This structural check does not establish that every normative sentence
is consistent, execute the 131 source targets or discharge semantic proof gates;
those still require separate review and their actual consumers.

Git's tracked Markdown inventory defines the scope. The owner-link and
status checks enforce traceability, and semantic agreement is reviewed
independently. It is auxiliary evidence, not lang syntax,
a source consumer, an ISA
model or normative ontology. Its explanatory implementation-language words
(including atomic enum) define no lang Pattern or declaration. Execution and
SHA-256 checks are recorded in [the ledger](../../lifetime-concurrency-consolidation.md).
The model covers at most three ARC tickets/two actors and simplified MP/SB.
It does not prove full CAS resource semantics, Region/Color, repair, HeapStack,
no-thin-air, overflow/Weak/reclamation, fence/ISA or compiler preservation.

The recursive Pattern, arrow, Atomic custody and Heap handoff acceptance
targets in AS01–AS14 remain normative. Rust frontend golden tests
exercise syntax. The preserved finite model and the inventory audit do
not discharge the revised transaction, staged identity, release or
backend proofs.
