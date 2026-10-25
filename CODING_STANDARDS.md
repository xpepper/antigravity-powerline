# Coding standards

Judgement calls the reviewer applies to every diff. Mechanical rules live in tooling (rustfmt, clippy, the tests, `scripts/`), not here.

## Domain words

Identifiers, comments and user-facing docs (README, `examples/powerline.toml`, help text) use the terms in [GLOSSARY.md](GLOSSARY.md) once created. A word on a term's _Avoid_ list is a finding, e.g. "open threads" for Unresolved Threads.

## One setting, one name

A setting that crosses a boundary (config key, hidden worker flag, function parameter) keeps one name and one polarity end to end: `show_unresolved` in config becomes `fetch_unresolved` in the worker and `--fetch-unresolved` on its command line.

## Flag arguments

A function takes at most one `bool` parameter. When two or more travel together, pass the config struct they come from (e.g. `&PrConfig`) or an enum, so a call site reads without the signature.

## Comments describe the code

A comment states what the code does or why, in terms that stay true wherever the code runs. Observations about this repository's own PRs, history or data belong in the commit message.
