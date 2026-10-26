//! Zuchu — terminal emulator for kacuOS.
//!
//! Language roles in the stack:
//!   Rust : PTY, VT parser, Lua config, GUI glue (this crate)
//!   Zig  : render core (grid/atlas)         -> zuchu-render/
//!   C    : system helpers (zuchu-stat)      -> zuchu-stat/
//!   Lua  : config ~/.config/zuchu/config.lua

mod config;
mod gui;
mod pty;
mod shellgen;
mod term;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let headless = args.iter().any(|a| a == "--headless");

    let cfg = config::Config::load()?;
    eprintln!(
        "zuchu: config loaded (shell={}, grid={}x{})",
        cfg.shell, cfg.cols, cfg.rows
    );

    // Generate shell snippets from the Lua config
    // (bash/zsh hook is an artifact, not a config language)
    let hook_path = shellgen::write_hook(&cfg)?;
    eprintln!("zuchu: shell hook written: {}", hook_path.display());

    if headless {
        // Test mode with no window: feed PTY output through the VT parser
        // and dump the grid to stdout. Built for WSL: parser logic is testable
        // even without WSLg.
        let mut session = term::Session::new(cfg.cols as usize, cfg.rows as usize);
        let hook = hook_path.to_string_lossy().into_owned();
        let mut pty = pty::PtySession::spawn(&cfg.shell, &hook, cfg.cols, cfg.rows)?;

        // Keyboard forwarding: stdin (raw mode) -> PTY writer.
        // Without this the shell never sees what you type — you only see
        // your own terminal's local echo, which is exactly "it is silent".
        let guard = pty::raw_stdin::enable();
        let writer = pty.take_writer();
        std::thread::spawn(move || {
            let mut writer = writer;
            let Some(writer) = writer.as_mut() else { return };
            let stdin = std::io::stdin();
            let mut lock = stdin.lock();
            let mut buf = [0u8; 256];
            loop {
                match lock.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        let _ = writer.write_all(&buf[..n]);
                    }
                }
            }
        });

        let mut buf = [0u8; 8192];
        use std::io::{Read as _, Write as _};
        loop {
            let n = pty.read(&mut buf)?;
            if n == 0 {
                break;
            }
            session.feed(&buf[..n]);
            print!("\x1b[H\x1b[2J{}", session.term.dump());
            std::io::stdout().flush().ok();
        }

        pty::raw_stdin::restore(&guard);
        eprintln!("zuchu: pty closed, leaving headless mode");
        Ok(())
    } else {
        gui::run(&cfg)
    }
}
