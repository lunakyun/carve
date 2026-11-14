# Contributing

Thanks for helping! The full guide for code changes (adding an effect, rules
for code) is
[CONTRIBUTING.md](https://github.com/void1z2/carve/blob/master/CONTRIBUTING.md)
in the repository. This page covers the commands and the docs site.

## Run the tests

You need Rust (stable, edition 2024).

```
cargo test -p carve-core                               # language tests, no server needed
cargo clippy -p carve-core --all-targets -- -D warnings
cargo fmt --all
```

To test against a real server, see [in-game testing](ingame-testing.md).

## Build the docs locally

The site uses [VitePress](https://vitepress.dev). You need Node 20 or newer.

```
npm ci              # once
npm run docs:gen    # regenerate the syntax reference
npm run docs:dev    # live preview at http://localhost:5173/carve/
npm run docs:build  # static site in docs/.vitepress/dist
npm run docs:preview
```

Pages are the Markdown files in `docs/`. Any new `docs/syntax/*.md` file shows
up in the sidebar under **Language → More syntax** by itself, titled by its
first `# Heading`.

## Regenerate the reference

The pages in `docs/reference/` and `docs/public/syntax.json` are generated. Do
not edit them by hand.

```
cargo run -p carve-core --example gen_docs
```

The generator reads:

- the effect and condition pattern registries (`*_PATTERNS` consts), listed in
  `docs_meta::registries()` in `crates/carve-core/src/docs_meta.rs`. A new
  `*_PATTERNS` const that is not listed there is still found in the sources,
  with a warning;
- the event names from `EventKind::name` and the type names from `Ty::name`;
- names, descriptions, examples and hand-parsed syntax from the `DOCS` table in
  `docs_meta.rs`.

When you add syntax, add or extend an entry in `DOCS`. For a registry pattern,
put the exact pattern text in the entry's `patterns` so the entry claims it.
Patterns without an entry still appear, marked "No description yet", and the
generator prints a warning for each. Examples in `DOCS` are compiled by
`cargo test -p carve-core docs_meta`, so a broken example fails the tests.

The output is sorted and only rewritten when it changes, so running the
generator twice changes nothing.

## Deploy

Pushing to `master` runs `.github/workflows/docs.yml`: it runs the generator,
builds the site and publishes it to GitHub Pages at
<https://void1z2.github.io/carve/>. In the repository settings, Pages must use
**GitHub Actions** as its source.
