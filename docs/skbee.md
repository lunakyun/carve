# SkBee parity

Carve adds the most useful parts of the SkBee addon by default, with syntax close to SkBee so scripts port easily. Legend: ✅ done, 🟡 partial, ⛔ blocked.

## Text components

Syntax details: [syntax/skbee-text.md](syntax/skbee-text.md). Carve components are MiniMessage markup text, not a separate type. That is the main difference from SkBee.

- ✅ Text component from text (`text component from "x"`)
- ✅ Hover event showing text (`hover event showing "x"`)
- ✅ Hover event showing an item (`hover event showing 2 diamonds`; id and amount only)
- 🟡 Hover event showing an entity (MiniMessage tag only, no expression)
- ✅ Click event: open url, run command, suggest command, copy to clipboard, change page
- ✅ Set hover event / click event of a component (single variable only)
- ✅ Insertion (`set insertion of {c} to "x"`, `<insertion:..>`)
- ✅ Translate component with values (`translate component "key" with "a" and "b"`)
- ✅ Keybind component (`keybind component "key.jump"`)
- ✅ Merge components (`merged components of a, b and c`, or of a list variable)
- ✅ Send component to a player, send action bar component, broadcast component
- ✅ MiniMessage: `mini message from "..."` and the tags `<hover>`, `<click>`, `<insertion>`, `<key>`, `<lang>`, `<newline>`
- ✅ MiniMessage gradient (`<gradient:red:blue>`, any number of colors, optional shift) and `<rainbow>`
- ✅ Plain text of a component (`plain text of x`)
- 🟡 Component styles as expressions (use tags like `<bold>` or `&l` instead)
- 🟡 Events on a list of components (set events on one variable at a time)
- ⛔ Fonts and shadow color on a component: the plugin API supports them, but Carve has no syntax yet
- ⛔ Component object tags (`<sprite>`, `<head>`) and `<selector>`/`<nbt>` tags: not planned
