These fixtures are committed physical source trees used by lang_build integration tests.
They are not manifest syntax and do not imply package-manager semantics.

Some workspaces are intentionally invalid (used by malformed-source and
diagnostic-boundary tests) and are expected to fail to build. They are still real
committed source trees, not generated at test time.

Successful semantic fixture expectations should be expressed inside `.lang`
sources with build-time `verify ...` forms. Rust integration tests should run
the fixture workspace and assert pass/fail plus expected diagnostic category for
fixtures that fail before source verification can run.
