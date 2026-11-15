---
layout: home
title: Carve
titleTemplate: Scripts for Pumpkin

hero:
  name: Carve
  text: Scripting for Pumpkin
  tagline: Skript-style scripts for Pumpkin, the Minecraft server written in Rust
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
      link: https://github.com/lunakyun/carve

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
    details: Typos, wrong types and unknown items are caught on reload with file, line and a "did you mean" hint. Like Skript, a broken line is skipped and the rest keeps working.
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
    details: A WebAssembly plugin for Pumpkin, written in Rust. Works on hosted servers too.
    link: /getting-started#hosted-servers
    linkText: Hosted servers
---

## What is Carve?

Carve is a scripting plugin for [Pumpkin](https://github.com/Pumpkin-MC/Pumpkin), the Minecraft server written in Rust. You write commands, events and small minigames in Skript-style English, and `/carve reload` puts them live without a restart.

Scripts are compiled once at load time into a flat instruction list and run by a small virtual machine inside a sandboxed WebAssembly plugin. A per-slice instruction budget stops runaway loops, and variables are saved to disk for you. If you've used Skript on Spigot or Paper, most of Carve will look familiar.
