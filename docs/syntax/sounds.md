# Sounds

Play and stop sounds for a player.

```
play sound "entity.experience_orb.pickup" to player
play sound "block.note_block.pling" with volume 0.5 and pitch 2 to player
play sound "block.anvil.land" to player at location(10, 70, 20)
play sound "entity.player.levelup" at location(0, 64, 0)
stop sound "music.game" from player
stop all sounds for player
```

Rules:

- The sound id is text. It is lowercased. Only letters, digits and `_ . : / -`
  are allowed. Anything else is a runtime error.
- `to player` is optional. Without it, the event player hears the sound.
- `at %location%` sets where the sound plays. With no player given, everyone
  online hears it.
- Volume is 0 or more (default 1). Pitch is 0 to 2 (default 1). Pitch is
  clamped.
- `with`, `at` and `and` work before `volume` and `pitch`. `for` works like `to`,
  and `from` like `for` for stop.

How it works: Carve uses the plugin API (the `master` category). A sound name
that isn't a vanilla sound, or a player who isn't online, logs a warning with
the script file and line. Not done yet: sound categories and playing to
several players at once.
