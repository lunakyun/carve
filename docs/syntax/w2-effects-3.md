# Misc player effects

All effects take a player. `clear title` and `reset title` default to the event player.

| Syntax | What it does |
| --- | --- |
| `poison %player% [for %timespan%]` | Gives poison. Uses the `effect` command. |
| `cure %player% [of poison]` | Removes poison. |
| `hide %player% [from %player%]` | Hides a player from one viewer, or from everyone. |
| `reveal %player% [to %player%]` | Shows a hidden player again. |
| `show/hide [the] custom name of %player%` | Sets custom name visibility. |
| `make %player% start/stop sprinting` | Sets sprinting. |
| `make %player% swing [main/off] hand` | Plays the arm swing. |
| `clear/reset [the] title of %player%` | Clears or resets the title. |
| `dismount %player%`, `eject %player%` | Leave a vehicle, or throw off passengers. |
| `make %player% ride %player%` | Mount another player. |
| `send resource pack "url" [with hash "sha1"] to %player%` | Offers a resource pack (Java players). |
| `save [the] world`, `stop [the] server` | World and server control. |
| `toggle {flag}` | Flips a boolean. An unset variable becomes true. |
| `transform {list::*} with (input * 2)` | Replaces every item with the result. |
