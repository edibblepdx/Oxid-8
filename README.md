# `Oxid-8`

```
▄▄▄▄              ▄▄▄▄
█  █ ▜▄▛ █ █▀▄ ▄▄ █▄▄█
█▄▄█ █ █ █ █▄▀    █▄▄█
```
is a Chip-8 interpreter written in rust and drawn to the terminal using `ratatui`.

![oxid-tetris](https://github.com/user-attachments/assets/ab1f3bdc-4ab0-48f8-8563-1ee89c436e90)

(Currently the only way to run a game) Inside `src/bin` is `oxid-cli`—a lighter version without a menu and useful for testing. You can load a ROM by passing its path as the first command-line argument (all others will be discarded). If no arguments are given, it will fallback to an `OXID_ROM` environment variable if set; it's recommended to use `realpath` with environment variables.

## Terminals that support the Kitty Keyboard Protocol 

> Most terminals do not differentiate key press, release, and repeat. [read more][Kitty Protocol]

- The alacritty terminal
- The ghostty terminal
- The foot terminal
- The iTerm2 terminal
- The rio terminal
- The WezTerm terminal

Non-kitty terminals are supported by clearing the virtual keyboard after each cycle. The result is playable, but noticeably worse.

## Sound

Sound is played by printing the bell character `\x07`. If you don't hear anything when you are expecting to, it's possible that you may have muted the bell (I don't blame you).

## TODO

- menus
- wasm version
- would like to make a neat shader
- debug to step through instructions
- super chip-8 extension

## License

This project is licensed under the [MIT License][License].

[License]: ./LICENSE
[Kitty Protocol]: https://sw.kovidgoyal.net/kitty/keyboard-protocol/
