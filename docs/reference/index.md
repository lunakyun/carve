---
title: Syntax reference
editLink: false
aside: false
---

# Syntax reference

Every event, effect, condition, expression, type and built-in function Carve
knows. Type to filter, or pick a category. Each card links to its full entry
and to the guide page that explains it.

The list is generated from Carve's syntax registries by
`cargo run -p carve-core --example gen_docs`, so it matches the plugin you
build from the same source. Browse by page:
[events](./events.md) · [effects](./effects.md) · [conditions](./conditions.md) ·
[expressions](./expressions.md) · [types](./types.md) · [functions](./functions.md).

In a pattern, `%type%` is a value, `[...]` is optional and `(a|b)` means one
of the words. The raw data is at [`syntax.json`](/syntax.json){target="_self"}.

<ClientOnly>
  <SyntaxSearch />
</ClientOnly>
