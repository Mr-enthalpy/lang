# Physical Source and Build Infrastructure

This block owns the normalization of physical source into a meta program and
the engineering boundary around its evaluation.

- [Compilation and physical normalization](build-system-design.md)
- [Namespace projection of evaluation](namespace-assembly.md)
- [Library Objects](library-namespace-design-note.md)
- [Compiler input boundary](package-manifest.md)

Names, capability and construction authority belong to ordinary language
semantics. File discovery, decoding, caching, scheduling, diagnostics and artifact
persistence implement those semantics without adding facts. External resources
enter through [host capabilities](../meta-invocation/host-capabilities-and-machine-objects.md).
