# Brokk's TypeScript Grammar for Tree-sitter

[![CI][ci]](https://github.com/BrokkAi/tree-sitter-typescript/actions/workflows/ci.yml)
[![crates][crates]](https://crates.io/crates/brokk-tree-sitter-typescript)
[![docs.rs][docs]](https://docs.rs/brokk-tree-sitter-typescript)

This is the **Brokk-owned and independently maintained fork** of
[`tree-sitter/tree-sitter-typescript`](https://github.com/tree-sitter/tree-sitter-typescript),
which provides TypeScript and TSX grammars for [Tree-sitter][tree-sitter].
Brokk maintains this fork for its code-intelligence tooling and publishes the
Rust package as
[`brokk-tree-sitter-typescript`](https://crates.io/crates/brokk-tree-sitter-typescript).
It intentionally diverges where Brokk needs language support that is not yet
available upstream. Unless you specifically need Brokk's changes, you may
prefer the upstream project.

## Installation

Add the Brokk-maintained TypeScript crate to your project:

```sh
cargo add brokk-tree-sitter-typescript@=0.23.3
```

Or add it directly to `Cargo.toml`:

```toml
[dependencies]
brokk-tree-sitter-typescript = "=0.23.3"
```

The npm, Python, Go, Swift, and C bindings retain their upstream-compatible
package names but are not published by this fork. The Brokk-namespaced Rust
crate prefixes both grammars' native symbols, so it can coexist with the
upstream `tree-sitter-typescript` crate in one executable.

## Changes from upstream

Version 0.23.3 builds on upstream commit
`75b3874edb2dc714fb1fd77a32013d0f8699989f`. It adds generic arguments to
`import()` member types, newline-separated keyword property names in type
declarations, and `abstract override readonly` class properties. Expression
operators and malformed declarations have separate regression coverage.
The generic import change incorporates the public work by
[@elecnix](https://github.com/elecnix) in
[upstream pull request 365](https://github.com/tree-sitter/tree-sitter-typescript/pull/365).

The upstream MIT [license](LICENSE) and author attribution are preserved.

## Development

Use Node.js 24 and the locked dependencies. Parser generation uses exactly
Tree-sitter CLI 0.24.4 and `tree-sitter-javascript` 0.23.1:

```sh
npm ci --ignore-scripts --legacy-peer-deps
npm rebuild tree-sitter-cli
npm run generate
git diff --exit-code -- typescript/src tsx/src
npm run test:corpus
npm run lint
cargo fmt --check
cargo test --locked
cargo package --locked
cargo publish --dry-run --locked
```

The shared corpus runs against both parsers. Dialect-specific type assertions
and JSX cases explicitly select their parser. Rust tests also load the
upstream and fork grammars together and check malformed near misses.

## Releases

This fork publishes only `brokk-tree-sitter-typescript` on crates.io. Tags use
`v` followed by the Cargo package version and must remain immutable. Package
from the clean tagged commit, verify the archive's `.cargo_vcs_info.json`,
and record its SHA-256 before publication.

The first publication requires a short-lived crates.io token supplied through
`CARGO_REGISTRY_TOKEN` to `cargo publish --locked`. After that bootstrap,
configure the crate's trusted publisher for organization `BrokkAi`, repository
`tree-sitter-typescript`, workflow `publish.yml`, and environment `release`.
Set the repository variable `CARGO_TRUSTED_PUBLISHING` to `true` only once that
publisher is configured. Later tagged releases use the protected `release`
environment and the crates.io OIDC workflow.

## Upstream bindings

Because TSX and TypeScript are actually two different dialects, this module defines two grammars. Require them as follows:

```js
require("tree-sitter-typescript").typescript; // TypeScript grammar
require("tree-sitter-typescript").tsx; // TSX grammar
```

For Javascript files with [flow] type annotations you can use the `tsx` parser.

[tree-sitter]: https://github.com/tree-sitter/tree-sitter
[flow]: https://flow.org/en/

References

- [TypeScript Language Spec](https://github.com/microsoft/TypeScript/blob/30cb20434a6b117e007a4959b2a7c16489f86069/doc/spec-ARCHIVED.md)

[ci]: https://img.shields.io/github/actions/workflow/status/BrokkAi/tree-sitter-typescript/ci.yml?logo=github&label=CI
[crates]: https://img.shields.io/crates/v/brokk-tree-sitter-typescript?logo=rust
[docs]: https://img.shields.io/docsrs/brokk-tree-sitter-typescript?logo=docs.rs
