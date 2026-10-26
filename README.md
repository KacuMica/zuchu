<div align="center">

<img src="assets/logo.svg" alt="Zuchu logo" width="220"/>

# Zuchu

**The terminal emulator of [kacuOS](https://github.com/KacuMica/kacuOS)** — an Arch-based distribution.

Customization à la Ghostty: your config lives in Lua, the whole terminal obeys it.

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Language](https://img.shields.io/badge/core-Rust-orange)
![Render](https://img.shields.io/badge/render-Zig-yellow)

</div>

---

Zuchu aims to become the **default terminal of kacuOS**: a slim, low-overhead
terminal that spawns `zsh + OhMyZsh` out of the box and is configured through
a single Lua file — not through a hundred command-line flags.

## Architecture — every language has its job

| Language | Responsibility | Location |
|----------|----------------|----------|
| **Rust** | main binary: PTY (spawning zsh), VT parser (the `vte` crate), Lua config (`mlua`), GUI glue (winit + glow) | `zuchu/` |
| **Zig** | render core: grid compositing into a vertex buffer, built as a static library over the C ABI | `zuchu-render/` |
| **C** | tiny system helpers (reads `/proc`) | `zuchu-stat/` |
| **Lua** | config at `~/.config/zuchu/config.lua` — the same approach WezTerm uses | `config/` |

**Bash/zsh is never a config language.** Instead, Zuchu *compiles* the Lua
config into a shell hook (`~/.config/zuchu/shell/zuchu-hook.sh`) that sessions
source at startup (colors, prompt, env). Bash here is a generated build
artifact, not hand-written code.

## Current status (v0.1.0)

- [x] Cargo workspace + project structure
- [x] PTY session (spawn shell, read/write via `portable-pty`)
- [x] VT parser (subset: printable, LF/CR/BS, SGR 16-color palette, CUP, erase ops)
- [x] **Keyboard forwarding** (raw-mode stdin → PTY): commands actually reach the shell
- [x] Lua config with generous defaults
- [x] Shell hook generated from the Lua config
- [x] Headless mode (`--headless`) — full parser + shell integration testing without a GUI
- [~] GUI (winit + OpenGL): window + event loop exist, real grid rendering is a TODO
- [ ] Rust ↔ Zig FFI (build.rs + linking `libzuchu_render.a`)
- [ ] Tabs, scrollback, selection/copy-paste, 24-bit SGR (v0.2+)

Tested on: **WSL Arch** (keyboard, PTY and the parser are fully functional).

## Building (Arch / WSL Arch)

```bash
pacman -S --needed base-devel clang rust zig
cargo build --release

# render core (built separately for now, build.rs wiring comes with FFI):
cd zuchu-render && zig build && cd ..

# C helper:
cd zuchu-stat && cc -O2 -o zuchu-stat zuchu-stat.c && cd ..
```

### Headless smoke test (works even without WSLg)

```bash
./target/release/zuchu --headless
```

This spawns the shell (zsh if present, otherwise $SHELL/bash), forwards
your keyboard into the PTY, parses everything through the VT state machine
and redumps the grid. Try `whoami`, `ls`, then `exit` to leave.

Details for Windows users: [docs/TESTING-WSL.md](docs/TESTING-WSL.md).

## Configuration

```lua
-- ~/.config/zuchu/config.lua
-- a real Lua file: it is EXECUTED at startup, so conditions,
-- functions and dynamic theming all work
return {
  shell = "/bin/zsh",
  cols = 100, rows = 30,
  font = "monospace", font_size = 12.0,
  theme = {
    bg = { 0.055, 0.055, 0.075 },
    fg = { 0.92, 0.92, 0.94 },
  },
}
```

Every start of `zuchu` re-reads the file and regenerates the shell hook from it.

## Default shell: zsh + OhMyZsh

Zuchu does not force a shell, but in the packaged kacuOS image a deploy step
kicks in: if the user has no `~/.zshrc`, OhMyZsh is installed together with
the Zuchu base `zshrc` (theme, hook integration). This is the job of the
distribution's deploy tooling, not of the terminal binary itself.

## Roadmap to v1.0

1. **v0.1** *(this release)* — skeleton; PTY + parser + keyboard work headless; Lua config.
2. **v0.2** — text rendering through the Zig core; real GUI terminal; hot config reload.
3. **v0.3** — tabs, scrollback, selection/copy-paste, Lua keybindings.
4. **v0.4** — splits, clickable links, 256/true color, ligatures.
5. **v1.0** — stable Lua plugin API, PKGBUILD packaging, deployment into the kacuOS ISO.

## License

MIT — same as kacuOS. See [LICENSE](LICENSE).
