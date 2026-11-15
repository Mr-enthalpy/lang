#!/usr/bin/env python3
"""Reproduce this consolidation's document inventories and bounded evidence.

These checks do not prove source semantics, the F gates, or a machine backend.
The scope is explicit so the audit still works after the changes are committed.
Run with Python 3 from any directory; no downloaded handoff or Git state is needed.
"""

import hashlib
import json
import pathlib
import re
import subprocess
import sys
import urllib.parse


ROOT = pathlib.Path(__file__).resolve().parents[4]
MARKDOWN_FILES = [
    "AGENTS.md",
    "README.md",
    "spec/README.md",
    "spec/contracts/control-flow-end-events.md",
    "spec/contracts/e-host-architecture-and-incremental-revalidation.md",
    "spec/contracts/lifetime-program-consumers.md",
    "spec/contracts/policy-migration.md",
    "spec/contracts/raw-ast-contract.md",
    "spec/contracts/semantic-values-and-extraction-interfaces.md",
    "spec/design/README.md",
    "spec/design/control-flow/effect-steps-and-continuation-handoff.md",
    "spec/design/control-flow/heterogeneous-computation-containers.md",
    "spec/design/lifetime/README.md",
    "spec/design/lifetime/atomic-pattern-transitions-and-causality.md",
    "spec/design/lifetime/heap-stack-and-storage-responsibility.md",
    "spec/design/lifetime/lifetime-algebra-walkthroughs.md",
    "spec/design/lifetime/lifetime-policy-and-overload-boundary.md",
    "spec/design/lifetime/unsafe-semantic-admission.md",
    "spec/design/patterns-overload/pattern-relations-observation-and-extraction.md",
    "spec/design/patterns-overload/product-repetition-and-unpacking.md",
    "spec/design/patterns-overload/return-value-extraction-and-implicit-decomposition.md",
    "spec/design/patterns-overload/static-pattern-spaces-and-extraction-chains.md",
    "spec/design/semantic-spine.md",
    "spec/design/static-evaluation/evaluation-residual-and-optimization.md",
    "spec/design/static-evaluation/host-capabilities-and-machine-objects.md",
    "spec/design/symbol-world/dependency-observation-and-realization.md",
    "spec/design/symbol-world/function-object-call-model.md",
    "spec/design/symbol-world/policy-and-static-flow-projection.md",
    "spec/design/symbol-world/structural-type-formation-and-composition.md",
    "spec/design/symbol-world/type-associated-function-objects-and-access-trees.md",
    "spec/design/unified-static-name-and-structural-semantics.md",
    "spec/planning/canonical-semantic-conformance.md",
    "spec/planning/evidence/lifetime-concurrency/README.md",
    "spec/planning/heap-io-effect-migration.md",
    "spec/planning/lifetime-concurrency-acceptance.md",
    "spec/planning/lifetime-concurrency-consolidation.md",
    "spec/planning/open-questions.md",
    "spec/planning/roadmap.md",
    "spec/public/agent-interpretation-guide.md",
    "spec/public/normalized-surface-semantics.md",
    "spec/reference/glossary.md",
]
OWNERS = [
    "spec/design/lifetime/lifetime-policy-and-overload-boundary.md",
    "spec/design/lifetime/heap-stack-and-storage-responsibility.md",
    "spec/design/lifetime/atomic-pattern-transitions-and-causality.md",
    "spec/design/control-flow/heterogeneous-computation-containers.md",
    "spec/design/symbol-world/function-object-call-model.md",
    "spec/design/patterns-overload/pattern-relations-observation-and-extraction.md",
    "spec/design/patterns-overload/return-value-extraction-and-implicit-decomposition.md",
    "spec/design/unified-static-name-and-structural-semantics.md",
]
MODEL_HASHES = {
    "expressiveness_checks_v0_1.py": "ab3749e73d086dc8f069a94acafb1d0bb90c039d1715c05d99105770c427e5ab",
    "model_output.txt": "74f49cfac4d398afc2ceef483b50729f21d8f4abfc3c6455f299813ce4ff1aa1",
}


def read(relative):
    return (ROOT / relative).read_text(encoding="utf-8")


def prose(text):
    return re.sub(r"^```[^\n]*\n.*?^```\s*$", "", text, flags=re.M | re.S)


def anchors(path):
    seen, result = {}, set()
    for heading in re.findall(r"^#{1,6}\s+(.+?)\s*#*$", prose(path.read_text(encoding="utf-8")), re.M):
        heading = re.sub(r"`([^`]+)`", r"\1", heading).lower()
        slug = re.sub(r"[^\w\-\s]", "", heading).replace(" ", "-")
        count = seen.get(slug, 0)
        seen[slug] = count + 1
        result.add(slug + (f"-{count}" if count else ""))
    return result


def require(condition, message):
    if not condition:
        raise ValueError(message)


def main():
    bad_links, links = [], 0
    for name in MARKDOWN_FILES:
        path = ROOT / name
        for dest in re.findall(r"(?<!!)\[[^\]\n]+\]\(([^)\n]+)\)", prose(read(name))):
            if re.match(r"\w+://|mailto:", dest):
                continue
            dest = urllib.parse.unquote(dest.strip("<>"))
            target, _, anchor = dest.partition("#")
            resolved = (path.parent / target).resolve() if target else path
            links += 1
            if not resolved.exists():
                bad_links.append((name, dest, "missing file"))
            elif anchor and resolved.suffix == ".md" and anchor not in anchors(resolved):
                bad_links.append((name, dest, "missing anchor"))

    headings = "\n".join(read(p) for p in OWNERS)
    ids = re.findall(r"^### (N\d\d) —", headings, re.M)
    require(sorted(ids) == [f"N{i:02}" for i in range(1, 46)], "N owner headings must be unique and contiguous")
    unified = read(OWNERS[-1])
    require(re.findall(r"^## (U\d\d) —", unified, re.M) == [f"U{i:02}" for i in range(1, 13)], "U headings mismatch")
    ledger = read("spec/planning/lifetime-concurrency-consolidation.md")
    for prefix, count in (("M", 31), ("F", 18)):
        require(re.findall(rf"^\| ({prefix}\d\d) \|", ledger, re.M) ==
                [f"{prefix}{i:02}" for i in range(1, count + 1)], f"{prefix} inventory mismatch")
    acceptance = read("spec/planning/lifetime-concurrency-acceptance.md")
    cases = re.findall(r"^\| ((?:REG|CLR|HET|REP|JOIN|IDX|FORG|HEAP|ATOM|MEM|INT)\d\d) \|", acceptance, re.M)
    require(len(cases) == len(set(cases)) == 105, "105 unique paired acceptance cases required")
    require(acceptance.count("| ID | 规范条款 | 正向要求 | 必须拒绝或不得推导 | D | P | S | C | T | B |") == 11,
            "all 11 acceptance groups must expose D/P/S/C/T/B")

    evidence = pathlib.Path(__file__).resolve().parent
    for name, expected_hash in MODEL_HASHES.items():
        require(hashlib.sha256((evidence / name).read_bytes()).hexdigest() == expected_hash,
                f"preserved evidence hash mismatch: {name}")
    result = subprocess.run([sys.executable, str(evidence / "expressiveness_checks_v0_1.py")],
                            cwd=ROOT, capture_output=True, check=True)
    normalize = lambda data: data.replace(b"\r\n", b"\n")
    require(normalize(result.stdout) == normalize((evidence / "model_output.txt").read_bytes()),
            "bounded model stdout differs from supplied output")
    print(json.dumps({"markdown_files": len(MARKDOWN_FILES), "local_links": links,
                      "bad_links": bad_links, "unique_N_owners": len(ids), "U": 12,
                      "M": 31, "F": 18, "paired_cases": len(cases),
                      "model_hashes_verified": len(MODEL_HASHES), "model_exit_code": result.returncode,
                      "model_stdout_matches": True}, ensure_ascii=True))
    return bool(bad_links)


if __name__ == "__main__":
    sys.exit(main())
