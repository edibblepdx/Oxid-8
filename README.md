# `Oxid-8`

```
▄▄▄▄              ▄▄▄▄
█  █ ▜▄▛ █ █▀▄ ▄▄ █▄▄█
█▄▄█ █ █ █ █▄▀    █▄▄█
```
is a Chip-8 interpreter written in rust and drawn to the terminal using `ratatui`.

Inside `src/bin` is `oxid-cli`—a lighter version without a menu and useful for testing. You can load a ROM by passing its path as the first command-line argument (all others will be discarded). If no arguments are given, it will fallback to an `OXID_ROM` environment variable if set; it's recommended to use `realpath` with environment variables.

## Terminals that support the Kitty Keyboard Protocol 

> Most terminals do not differentiate key press, release, and repeat.

[read more](https://sw.kovidgoyal.net/kitty/keyboard-protocol/)

- The alacritty terminal
- The ghostty terminal
- The foot terminal
- The iTerm2 terminal
- The rio terminal
- The WezTerm terminal

## TODO

- wasm version
- debug to step through instructions
- super chip-8 extension

## License

This project is licensed under the [MIT License][License].

[License]: ./LICENSE
