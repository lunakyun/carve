---
layout: home
title: Carve
titleTemplate: Scripts for Pumpkin

hero:
  name: Carve
  text: Scripting for Pumpkin
  tagline: Simple, readable scripts for the Pumpkin Minecraft server
  image:
    src: /carve-icon.png
    alt: Carve pumpkin icon
  actions:
    - theme: brand
      text: Get started
      link: /getting-started
    - theme: alt
      text: Syntax reference
      link: /reference/
    - theme: alt
      text: GitHub
      link: https://github.com/void1z2/carve

features:
  - title: Almost plain English
    details: "Write on join: send \"hi\" to player. Save the file, run /carve reload, done. No Rust and no compiling."
    link: /syntax/triggers
    linkText: Triggers
  - title: Skript-style syntax
    details: Events, effects, conditions, loops, functions, lists and variables, in the style Skript users already know.
    link: /skript-parity
    linkText: Skript parity
  - title: Errors at load, not at runtime
    details: Typos, wrong types and unknown items are caught on reload with file, line and a "did you mean" hint. A bad reload keeps the old scripts.
    link: /errors
    linkText: Reading errors
  - title: Fast and safe
    details: Scripts compile to flat instructions, waits never block the server, and a runaway guard stops endless loops.
    link: /performance
    linkText: Performance
  - title: Searchable reference
    details: Every event, effect, condition and expression, generated from Carve's own syntax registries.
    link: /reference/
    linkText: Search syntax
  - title: Built for Pumpkin
    details: A WebAssembly plugin for the Pumpkin server, written in Rust. Works on hosted servers too.
    link: /getting-started#hosted-servers
    linkText: Hosted servers
---

## A taste

```carve
on join:
    add 1 to {joins::%uuid of player%}
    if {joins::%uuid of player%} is 1:
        broadcast "&eSay hi to %player%, they are new!"
    else:
        send title "Welcome back!" to player

command /heal [<player>]:
    permission: heal
    trigger:
        heal player
```
