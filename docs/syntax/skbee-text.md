# Text components (SkBee style)

A text component is text with hover, click and other extras. In Carve a component is a normal text value written in MiniMessage markup. You can store it in a variable, join it with other text and send it with `send`. The plugin turns the markup into real Minecraft text components when it sends.

Every message the plugin sends understands the tags below, so you can write them straight into a string.

## Tags

| Tag | Meaning |
| --- | --- |
| `<hover:show_text:'text'>..</hover>` | Hover text. The text can have colors and tags. |
| `<hover:show_item:diamond:3>..</hover>` | Hover shows an item (id, optional amount). |
| `<hover:show_entity:pig:'uuid':'name'>..</hover>` | Hover shows an entity. The name is optional. |
| `<click:run_command:'/spawn'>..</click>` | Click runs a command. |
| `<click:suggest_command:'/msg '>..</click>` | Click puts a command in the chat box. |
| `<click:open_url:https://example.com>..</click>` | Click opens a link. |
| `<click:copy_to_clipboard:'text'>..</click>` | Click copies text. |
| `<click:change_page:2>..</click>` | Click turns a book page. |
| `<insertion:'text'>..</insertion>` | Shift-click puts text in the chat box. |
| `<key:key.jump>` | Shows the player's own key for that action. |
| `<lang:chat.type.text:'Steve':'hi'>` | A translated text. The extra values fill the `%s` slots. |
| `<gradient:red:blue>..</gradient>` | Color fades across the letters. Any number of colors. An optional last number from -1 to 1 shifts it. |
| `<rainbow>..</rainbow>` | Rainbow colors. `<rainbow:!>` runs it backwards. |
| `<newline>` or `<br>` | New line. |
| `\<` | A literal `<`. |

All color tags and `&` codes from [colors](colors.md) work too. Quote a value with `'...'` if it has `:`, `<` or `>` in it. Use `\'` for a quote inside quotes. A tag that is not valid stays as plain text.

Gradient and rainbow color each letter on its own, so very long gradients make large messages.

## Expressions

| Expression | Result |
| --- | --- |
| `text component from "text"` | The text as typed. `<` and `\` are escaped, so tags do not run. `&` codes still work. Also `a new text component of ...`. |
| `mini message from "markup"` | The markup as is. Also `minimessage from` and `mm from`. |
| `hover event showing "text"` | A hover event. Also takes an item: `hover event showing 2 diamonds`. |
| `click event to run command "/spawn"` | A click event. Also `open url`, `suggest command`, `copy to clipboard`, `change page`. |
| `translate component "key" with "a" and "b"` | A translated text with values. Also `translatable component`. |
| `keybind component "key.jump"` | A keybind. |
| `merged components of a, b and c` | Joins components. Also takes a list: `merged components of {list::*}`. |
| `plain text of x` | The text with every tag and code removed. |

## Effects

```
set hover event of {_c} to hover event showing "Click for spawn"
set click event of {_c} to click event to run command "/spawn"
set insertion of {_c} to "shift-clicked"
send component {_c}
send component {_c} to player
send action bar component {_c} to player
broadcast component {_c}
```

`set ... event of` and `set insertion of` need a single variable. They wrap the component, so the new event covers all of it. If an event is already set inside, that part keeps its own.

## Example

```
command /rules:
    trigger:
        set {_c} to text component from "&6Click to read the rules"
        set hover event of {_c} to hover event showing "&7Opens our website"
        set click event of {_c} to click event to open url "https://example.com/rules"
        send component {_c}
        send "<gradient:gold:red>Welcome</gradient> <click:run_command:'/spawn'><hover:show_text:'Go to spawn'>[spawn]</hover></click>"
```

## Limits

- Components are markup text, not a separate type. `text component from "<red>x"` shows `<red>x`, but the stored value has a `\` in it. Use `plain text of` to read it back.
- `hover:show_item` passes only the item id and amount. Custom item data is not supported.
