# `Oxid-8 Core`

This is the core interpreter library for `Oxid8`. Developers can create their own renderers on top of this library crate.
It is a Chip-8 interpreter core written in rust. Try it out in the [web with WGPU][Web]. More examples to be found in the
[GitHub repo][GitHub].

```
▄▄▄▄              ▄▄▄▄
█  █ ▜▄▛ █ █▀▄ ▄▄ █▄▄█
█▄▄█ █ █ █ █▄▀    █▄▄█
```

![oxid-tetris](https://github.com/user-attachments/assets/ab1f3bdc-4ab0-48f8-8563-1ee89c436e90)

## Getting Started With a Basic Example

```rust
use oxid8_core::Oxid8;
use std::time::{Duration, Instant};

#[derive(Default)]
struct State {
    should_exit: bool,
    last_frame: Option<Instant>,
}

#[derive(Default)]
struct Emu {
    state: State,
    core: Oxid8,
}

fn main() -> anyhow::Result<()> {
    let mut emu = Emu::default();
    emu.core.load_font();
    emu.core.load_rom("rom_path")?;

    while !emu.state.should_exit {
        let time = Instant::now();

        // TODO: Poll and Handle Events.

        if let Some(last_frame) = emu.state.last_frame {
            if time.duration_since(last_frame) >= Duration::from_millis(16) {
                emu.core.next_frame()?;

                // TODO: Draw current frame.

                emu.state.last_frame = Some(time);
            }

            if emu.core.sound() {
                // TODO: Beep!
            }
        } else {
            emu.state.last_frame = Some(Instant::now());
        }
    }

    Ok(())
}
```

## WASM Compatibility

Add the following to your `Cargo.toml` and `config.toml`.

```toml
# Cargo.toml

[dependencies]
web-time = "1.1.0"

[target.'cfg(target_arch = "wasm32")'.dependencies]
getrandom = { version = "0.3", features = ["wasm_js"] }
```
```toml
# config.toml

[target.'cfg(target_arch = "wasm32")']
rustflags = ["--cfg", 'getrandom_backend="wasm_js"']
```

## License

This project is licensed under the [MIT License][License].

[GitHub]: https://github.com/edibblepdx/Oxid-8/tree/main
[License]: https://github.com/edibblepdx/Oxid-8/blob/main/LICENSE
[Kitty Protocol]: https://sw.kovidgoyal.net/kitty/keyboard-protocol/
[Web]: https://edibblepdx.github.io/Oxid-8/
