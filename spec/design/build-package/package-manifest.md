# Compiler Input and Tool Configuration Boundary

Status: canonical input boundary. This path documents engineering configuration;
it does not define a manifest language with program-meaning authority.

The compiler selects one compilation level and finds main.lang as its explicit
root anchor. Source meta evaluation determines Objects, dependencies, external
resources, policies, and the target machine. Configuration cannot add semantic
facts through dependency mounts, package roots, target options, feature flags,
defines, include paths or library paths.

Distinguish CompilerSemanticInvocation from surrounding process/tool setup:

    CompilerSemanticInvocation = selected Level + optional O1/O2 planner controls
    ProcessConfiguration = engineering storage/scheduling/diagnostic facilities

Extra parameters of the language compiler invocation control only planner search;
they do not add cache, target, feature or namespace choices to that interface.
A surrounding process may arrange cache storage, parallel workers, diagnostics
and artifact locations where they preserve program meaning. That process setup
is not another langc semantic argument channel. Planner controls may alter
equivalent-rewrite search budget and strategy without altering E.
A convenience tool that generates source still produces explicit source actions;
its private configuration is not a second evaluator input.

Dependency/version retrieval can be implemented through ordinary source-defined
host IO and representation. A registry/version solver is not a prerequisite
language subsystem or a namespace authority.

The existing Rust BuildManifest and package/workspace records are current
implementation carriers whose source wiring is pending migration. Their presence
does not establish canonical input semantics. See
[physical normalization](build-system-design.md) and the
[roadmap](../../planning/roadmap.md).
