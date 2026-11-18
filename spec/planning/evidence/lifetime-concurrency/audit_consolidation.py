#!/usr/bin/env python3
"""Reproduce this consolidation's document inventories and bounded evidence.

These checks do not prove source semantics, the F gates, or a machine backend.
The audit reads every tracked Markdown file outside spec/history from the checkout.
Run with Python 3 from any directory; no downloaded handoff is needed.
"""

import hashlib
import json
import pathlib
import posixpath
import re
import subprocess
import sys
import urllib.parse


ROOT = pathlib.Path(__file__).resolve().parents[4]
CONFORMANCE = "spec/planning/canonical-semantic-conformance.md"
LEDGER = "spec/planning/lifetime-concurrency-consolidation.md"
# Clause identifiers remain stable when an obligation is consolidated.
N_IDS = [f"N{i:02}" for i in range(1, 51) if i != 30]

OWNER_REFS = {
    "Pattern": "spec/design/patterns-overload/pattern-relations-observation-and-extraction.md#external-typed-leaves",
    "Compile": "spec/design/static-evaluation/compile-instance-invocation-and-result-delivery.md#runtime-inputs-to-compile-producers",
    "Atomic": "spec/design/lifetime/atomic-pattern-transitions-and-causality.md#n46--普通结构真实类型包装与表示预算-定义",
    "Lazy": "spec/design/lifetime/heap-stack-and-storage-responsibility.md#n28--分配成功时注册-lazy-drop-不变量",
    "Submit": "spec/design/lifetime/heap-stack-and-storage-responsibility.md#n29--relocate-精确向直接父层提交且不执行-pending-不变量",
}
ENTRY_POINTS = (
    "spec/README.md",
    "spec/public/normalized-surface-semantics.md",
    "spec/public/agent-interpretation-guide.md",
    "spec/reference/glossary.md",
    CONFORMANCE,
    LEDGER,
)
RULE_STATUS = {
    "Typed wrapper": ("Defined", "Source consumer pending"),
    "Mixed-stage inputs": ("Defined", "Source consumer pending"),
    "Lazy registration": ("Defined", "Source consumer pending"),
    "Heap submission": ("Defined", "Source consumer pending"),
    "Extra memory order": ("Open", "Semantic choice pending"),
}
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


def table_cells(line):
    """Markdown pipes split cells even in code spans; an escaped pipe does not."""
    cells, start, slashes = [], 1, 0
    for i, char in enumerate(line[1:], 1):
        if char == "|" and slashes % 2 == 0:
            cells.append(line[start:i].strip())
            start = i + 1
        slashes = slashes + 1 if char == "\\" else 0
    return cells


def markdown_structure(name, text):
    fences = re.findall(r"^```", text, re.M)
    require(len(fences) % 2 == 0, f"unpaired code fence: {name}")
    expected = None
    in_fence = False
    expect_separator = False
    for number, line in enumerate(text.splitlines(), 1):
        line = line.rstrip()
        if line.startswith("```"):
            require(not expect_separator, f"missing table header separator: {name}:{number}")
            in_fence = not in_fence
            expected = None
            continue
        if in_fence:
            continue
        if not line.startswith("|"):
            require(not expect_separator, f"missing table header separator: {name}:{number}")
            expected = None
            continue
        require(line.endswith("|"), f"unfinished table row: {name}:{number}")
        count = len(table_cells(line))
        if expected is None:
            expected = count
            expect_separator = True
        elif expect_separator:
            require(all(re.fullmatch(r":?-+:?", cell) for cell in table_cells(line)),
                    f"missing table header separator: {name}:{number}")
            expect_separator = False
        require(count == expected, f"table width {count}, expected {expected}: {name}:{number}")
    require(not expect_separator, f"missing table header separator at end: {name}")


def tracked_markdown():
    paths = subprocess.check_output(
        ["git", "ls-files", "-z", "--", ":(icase,glob)**/*.md",
         ":(exclude,glob)spec/history/**"], cwd=ROOT,
    ).decode("utf-8")
    return sorted(set(paths.split("\0")) - {""})


def negative_label(text):
    return re.match(
        r"(?:negative(?: example| boundary)?|reject(?:ed)?(?: example)?|"
        r"反例|禁止|不得|拒绝|must not|do not|no)\b|^(?:反例|禁止|不得|拒绝)",
        text.strip().lstrip("-*># ").lower(),
    )


def retired_findings(name, text):
    """Check explicit retired markers in current claims, not negative examples."""
    findings = []
    negative_columns = set()
    negative_block = False
    next_fence_negative = False
    in_fence = False
    material = []

    def check(lines):
        if lines:
            number = lines[0][0]
            statement = " ".join(value.strip() for _, value in lines)
            for rule, pattern in RETIRED_RULES.items():
                if re.search(pattern, statement, re.I):
                    findings.append((name, number, rule))

    for number, line in enumerate(text.splitlines(), 1):
        value = line.strip()
        if value.startswith("```"):
            check(material)
            material.clear()
            negative_block = next_fence_negative if not in_fence else False
            next_fence_negative = False
            in_fence = not in_fence
            continue
        if in_fence:
            if not negative_block:
                material.append((number, line))
            continue
        if not value:
            check(material)
            material.clear()
            negative_columns.clear()
            negative_block = False
            continue
        if value.startswith("|") and value.endswith("|"):
            check(material)
            material.clear()
            cells = table_cells(value)
            if re.match(r"^\|\s*(?:ID|Rule|Clause)\s*\|", value):
                negative_columns = {i for i, cell in enumerate(cells)
                                    if negative_label(cell) or re.search(r"不得|必须拒绝", cell)}
            for i, cell in enumerate(cells):
                if i not in negative_columns and not negative_label(cell):
                    check([(number, cell)])
            continue
        if negative_label(value):
            check(material)
            material.clear()
            negative_block = next_fence_negative = True
            continue
        if not negative_block:
            next_fence_negative = False
            if value.startswith("#"):
                check(material)
                material.clear()
                check([(number, value)])
            else:
                material.append((number, line))
    check(material)
    return findings


def audit_owner_sync(documents):
    for name in ENTRY_POINTS:
        directory = posixpath.dirname(name)
        for reference in OWNER_REFS.values():
            owner, anchor = reference.split("#")
            dest = posixpath.relpath(owner, directory) + "#" + anchor
            require(f"]({dest})" in documents[name], f"{name} is missing the owner link {dest}")

    results = {}
    for line in documents[LEDGER].splitlines():
        if line.startswith("|") and line.endswith("|"):
            cells = table_cells(line)
            if cells[0] in RULE_STATUS:
                require(cells[0] not in results, f"duplicate status: {cells[0]}")
                results[cells[0]] = tuple(cells[-2:])
    require(results == RULE_STATUS, "definition and execution statuses differ from the audit ledger")


def main():
    bad_links, links = [], 0
    documents = {name: read(name) for name in tracked_markdown()}
    audit_owner_sync(documents)
    retired = []
    for name, body in documents.items():
        path = ROOT / name
        markdown_structure(name, body)
        retired.extend(retired_findings(name, body))
        link_text = re.sub(r"(`+).*?\1", "CODE", prose(body))
        for dest in re.findall(r"(?<!!)\[[^\]\n]+\]\(([^)\n]+)\)", link_text):
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
    require(sorted(ids) == N_IDS, "N owner headings must match the active clause inventory")
    unified = read(OWNERS[-1])
    require(re.findall(r"^## (U\d\d) —", unified, re.M) == [f"U{i:02}" for i in range(1, 13)], "U headings mismatch")
    ledger = documents[LEDGER]
    require(re.findall(r"^\| (N\d\d) \|", ledger, re.M) == N_IDS, "N inventory mismatch")
    for prefix, count in (("M", 31), ("F", 19)):
        require(re.findall(rf"^\| ({prefix}\d\d) \|", ledger, re.M) ==
                [f"{prefix}{i:02}" for i in range(1, count + 1)], f"{prefix} inventory mismatch")
    acceptance = read("spec/planning/lifetime-concurrency-acceptance.md")
    groups = {"REG": 8, "DFLT": 10, "CLR": 6, "HET": 4, "REP": 13,
              "JOIN": 6, "IDX": 7, "FORG": 11, "HEAP": 14, "ATOM": 30,
              "MEM": 16, "INT": 6}
    cases = re.findall(r"^\| ((?:" + "|".join(groups) + r")\d\d) \|", acceptance, re.M)
    expected_cases = {f"{prefix}{i:02}" for prefix, count in groups.items()
                      for i in range(1, count + 1)}
    require(len(cases) == len(set(cases)) == 131 and set(cases) == expected_cases,
            "131 unique contiguous paired acceptance cases required")
    require(acceptance.count("| ID | 规范条款 | 正向要求 | 必须拒绝或不得推导 | D | P | S | C | T | B |") == 11,
            "all 11 acceptance groups must expose D/P/S/C/T/B")
    require(acceptance.count("| ID | 正向语义要求 | 不得推导 | D | P | S | C | T | B |") == 1,
            "DFLT group must expose its own D/P/S/C/T/B columns")
    for line in acceptance.splitlines():
        if re.match(r"^\| (?:" + "|".join(groups) + r")\d\d \|", line):
            require(table_cells(line)[-6:] == ["D", "—", "—", "—", "—", "—"],
                    "unimplemented source target must retain pending P/S/C/T/B: " + line.split("|")[1])
    redesign_cases = re.findall(r"^\| (AS\d\d) \|",
                                   read("spec/planning/canonical-semantic-conformance.md"), re.M)
    require(redesign_cases == [f"AS{i:02}" for i in range(1, 15)],
            "AS01-AS14 Pattern/transform/Atomic/Heap case inventory mismatch")

    evidence = pathlib.Path(__file__).resolve().parent
    for name, expected_hash in MODEL_HASHES.items():
        require(hashlib.sha256((evidence / name).read_bytes()).hexdigest() == expected_hash,
                f"preserved evidence hash mismatch: {name}")
    result = subprocess.run([sys.executable, str(evidence / "expressiveness_checks_v0_1.py")],
                            cwd=ROOT, capture_output=True, check=True)
    normalize = lambda data: data.replace(b"\r\n", b"\n")
    require(normalize(result.stdout) == normalize((evidence / "model_output.txt").read_bytes()),
            "bounded model stdout differs from supplied output")
    print(json.dumps({"markdown_files": len(documents), "local_links": links, "retired_findings": retired,
                      "checked_entry_points": len(ENTRY_POINTS), "definition_statuses": len(RULE_STATUS),
                      "bad_links": bad_links, "unique_N_owners": len(ids), "U": 12,
                      "M": 31, "F": 19, "paired_cases": len(cases),
                      "redesign_cases": len(redesign_cases),
                      "model_hashes_verified": len(MODEL_HASHES), "model_exit_code": result.returncode,
                      "model_stdout_matches": True}, ensure_ascii=True))
    return bool(bad_links or retired)


if __name__ == "__main__":
    sys.exit(main())
