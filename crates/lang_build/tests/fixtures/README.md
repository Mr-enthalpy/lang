These fixtures are committed physical source trees used by lang_build integration tests.
They are not manifest syntax and do not imply package-manager semantics.

Some workspaces are intentionally invalid (used by malformed-source and
diagnostic-boundary tests) and are expected to fail to build. They are still real
committed source trees, not generated at test time.

Source fixtures preserve declarations and syntax. Rust tests assert connected
semantic facts through ordinary namespace, binding and invocation APIs.
Standalone source expression completion and the selected Verify builtin consumer
remain unavailable until common E is connected. There is no source verification
post-pass. Negative fixtures assert the relevant diagnostics and consumer frontiers.
