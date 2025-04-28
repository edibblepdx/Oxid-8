# `Oxid-8`
```
▄▄▄▄              ▄▄▄▄
█  █ ▜▄▛ █ █▀▄ ▄▄ █▄▄█
█▄▄█ █ █ █ █▄▀    █▄▄█
```
is a Chip-8 interpreter written in rust and drawn to the terminal using `ratatui`.

Inside `src/bin` is `oxid-cli`—a lighter version without a menu and useful for testing. You can load a ROM by passing its path as the first command-line argument (all others will be discarded). If no arguments are given, it will fallback to an `OXID_ROM` environment variable if set; it's recommended to use `realpath` with environment variables.

## TODO
- wasm version
- debug to step through instructions
- super chip-8 extension
