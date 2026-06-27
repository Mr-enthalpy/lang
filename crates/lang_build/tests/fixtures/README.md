These fixtures are committed physical source trees used by lang_build integration tests.
They are not manifest syntax and do not imply package-manager semantics.

Some workspaces are intentionally invalid (used by malformed-source and
diagnostic-boundary tests) and are expected to fail to build. They are still real
committed source trees, not generated at test time.
