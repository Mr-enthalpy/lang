# Compiler Input and Tool Configuration Boundary

Status: canonical input boundary. This path documents engineering configuration;
it does not define a manifest language with program-meaning authority.

The compiler selects one compilation level and finds main.lang as its explicit
root anchor. Source meta evaluation determines Objects, dependencies, external
resources, policies, and the target machine. Configuration cannot add semantic
facts through dependency mounts, package roots, target options, feature flags,
defines, include paths or library paths.

Tools may configure diagnostics, cache storage, parallel scheduling and artifact
locations where these choices preserve program meaning. Future planner options
may alter equivalent-rewrite search budget and strategy without altering E.
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
