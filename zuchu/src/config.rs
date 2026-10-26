//! Lua configuration engine for Zuchu.
//!
//! Config file: ~/.config/zuchu/config.lua — a regular Lua script that
//! returns a table. See config/zuchu.lua in the repository for an example.

use std::path::{Path, PathBuf};

use mlua::{Lua, Table, Value};

#[derive(Clone)]
pub struct Theme {
    pub bg: (f32, f32, f32),
    pub fg: (f32, f32, f32),
}

#[derive(Clone)]
pub struct Config {
    pub shell: String,
    pub cols: u16,
    pub rows: u16,
    // Rendered by the GUI in v0.2 (font discovery + atlas)
    #[allow(dead_code)]
    pub font: String,
    #[allow(dead_code)]
    pub font_size: f32,
    pub theme: Theme,
}

pub fn config_dir() -> PathBuf {
    if let Ok(d) = std::env::var("ZUCHU_CONFIG_DIR") {
        return PathBuf::from(d);
    }
    let mut p = dirs_home();
    p.push(".config");
    p.push("zuchu");
    p
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME").map(PathBuf::from).unwrap_or_default()
}

fn read_vec3(t: &Table, key: &str, fallback: (f32, f32, f32)) -> (f32, f32, f32) {
    match t.get::<Value>(key) {
        Ok(Value::Table(c)) => {
            let r = c.get::<f64>(1).unwrap_or(0.0) as f32;
            let g = c.get::<f64>(2).unwrap_or(0.0) as f32;
            let b = c.get::<f64>(3).unwrap_or(0.0) as f32;
            (r, g, b)
        }
        _ => fallback,
    }
}

impl Config {
    /// Loads config.lua; falls back to generous defaults if the file is missing.
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = config_dir().join("config.lua");
        let lua = Lua::new();

        let t: Table = if path.exists() {
            let chunk = lua.load(&*path); // AsChunk is implemented for &Path
            chunk.eval()? // the config must return a table: `return { ... }`
        } else {
            eprintln!("zuchu: {} not found — using defaults", path.display());
            lua.create_table()?
        };

        let shell = t.get::<String>("shell").unwrap_or_else(|_| "/bin/zsh".into());
        let cols = t.get::<u16>("cols").unwrap_or(80);
        let rows = t.get::<u16>("rows").unwrap_or(24);
        let font = t.get::<String>("font").unwrap_or_else(|_| "monospace".into());
        let font_size = t.get::<f64>("font_size").unwrap_or(12.0) as f32;

        let theme = match t.get::<Value>("theme") {
            Ok(Value::Table(th)) => Theme {
                bg: read_vec3(&th, "bg", (0.055, 0.055, 0.075)),
                fg: read_vec3(&th, "fg", (0.92, 0.92, 0.94)),
            },
            _ => Theme {
                bg: (0.055, 0.055, 0.075),
                fg: (0.92, 0.92, 0.94),
            },
        };

        Ok(Config { shell: resolve_shell(shell), cols, rows, font, font_size, theme })
    }
}

/// Validates the shell path and falls back gracefully when it is absent
/// (e.g. a bare WSL Arch without zsh). Order: config -> $SHELL -> /bin/zsh
/// -> /bin/bash. If nothing exists, returns the original so the PTY can
/// surface a precise error to the user.
fn resolve_shell(preferred: String) -> String {
    if Path::new(&preferred).exists() {
        return preferred;
    }
    eprintln!("zuchu: shell '{}' not found — trying fallbacks", preferred);

    if let Ok(sys) = std::env::var("SHELL") {
        if !sys.is_empty() && Path::new(&sys).exists() {
            eprintln!("zuchu: falling back to $SHELL={}", sys);
            return sys;
        }
    }
    for cand in ["/bin/zsh", "/bin/bash"] {
        if Path::new(cand).exists() {
            eprintln!("zuchu: falling back to {}", cand);
            return cand.to_string();
        }
    }
    preferred
}
