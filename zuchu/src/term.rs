//! VT/xterm parser and the text grid of the terminal.
//!
//! v0.1: a subset (printable / LF / CR / BS / SGR 16-color palette /
//! CUP cursor moves / erase ops). Extended gradually; the `vte` crate
//! provides the state machine scaffolding.

use vte::{Parser, Perform};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Cell {
    pub ch: char,
    pub bold: bool,
    pub fg: Option<u8>, // 0..=15 palette index
    pub bg: Option<u8>,
}

impl Default for Cell {
    fn default() -> Self {
        Cell { ch: ' ', bold: false, fg: None, bg: None }
    }
}

#[derive(Default)]
struct Sgr {
    fg: Option<u8>,
    bg: Option<u8>,
    bold: bool,
}

/// Text grid + cursor. Implements `vte::Perform`.
pub struct Terminal {
    pub cols: usize,
    pub rows: usize,
    pub grid: Vec<Cell>,
    pub cx: usize,
    pub cy: usize,
    sgr: Sgr,
}

impl Terminal {
    pub fn new(cols: usize, rows: usize) -> Self {
        Terminal { cols, rows, grid: vec![Cell::default(); cols * rows], cx: 0, cy: 0, sgr: Sgr::default() }
    }

    fn put(&mut self, ch: char) {
        let idx = self.cy * self.cols + self.cx;
        if idx < self.grid.len() {
            self.grid[idx] = Cell { ch, bold: self.sgr.bold, fg: self.sgr.fg, bg: self.sgr.bg };
        }
        self.advance();
    }

    fn advance(&mut self) {
        self.cx += 1;
        if self.cx >= self.cols {
            self.cx = 0;
            self.newline();
        }
    }

    fn newline(&mut self) {
        self.cy += 1;
        if self.cy >= self.rows {
            self.cy = self.rows - 1;
            // Scroll: simple O(n) grid shift up (fine for v0.1)
            self.grid.drain(0..self.cols);
            self.grid.extend(std::iter::repeat(Cell::default()).take(self.cols));
        }
    }

    /// Dump the grid as text — used by the headless test mode.
    pub fn dump(&self) -> String {
        let mut out = String::with_capacity(self.rows * (self.cols + 1));
        for r in 0..self.rows {
            for c in 0..self.cols {
                out.push(self.grid[r * self.cols + c].ch);
            }
            while out.ends_with(' ') {
                out.pop();
            }
            out.push('\n');
        }
        out
    }
}

impl Perform for Terminal {
    fn print(&mut self, c: char) {
        self.put(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            b'\r' => self.cx = 0,
            0x08 => self.cx = self.cx.saturating_sub(1), // backspace
            b'\t' => {
                let next = (self.cx / 8 + 1) * 8;
                self.cx = next.min(self.cols - 1);
            }
            _ => {}
        }
    }

    fn csi_dispatch(&mut self, params: &vte::Params, _intermediates: &[u8], _ignore: bool, action: char) {
        let get = |i: usize, def: i64| -> i64 {
            params.iter().nth(i).and_then(|g| g.first().map(|&v| v as i64)).filter(|&v| v != 0).unwrap_or(def)
        };
        match action {
            'm' => {
                // SGR: colors/attributes (subset)
                let mut it = params.iter();
                while let Some(g) = it.next() {
                    match g.first().copied().unwrap_or(0) {
                        0 => self.sgr = Sgr::default(),
                        1 => self.sgr.bold = true,
                        22 => self.sgr.bold = false,
                        v @ 30..=37 => self.sgr.fg = Some(v as u8 - 30),
                        v @ 40..=47 => self.sgr.bg = Some(v as u8 - 40),
                        v @ 90..=97 => self.sgr.fg = Some(v as u8 - 90 + 8),
                        v @ 100..=107 => self.sgr.bg = Some(v as u8 - 100 + 8),
                        38 | 48 => { /* 24-bit RGB TODO (v0.2) */ }
                        _ => {}
                    }
                }
            }
            'H' => {
                // CUP: cursor position
                let row = get(0, 1).max(1) as usize;
                let col = get(1, 1).max(1) as usize;
                self.cy = (row - 1).min(self.rows - 1);
                self.cx = (col - 1).min(self.cols - 1);
            }
            'J' => {
                let mode = get(0, 0);
                if mode == 2 {
                    self.grid.iter_mut().for_each(|c| *c = Cell::default());
                }
                // TODO: mode 0 (from cursor) and mode 1
            }
            'K' => {
                let mode = get(0, 0);
                let start = if mode == 0 { self.cx } else { 0 };
                let end = if mode == 1 { self.cx } else { self.cols };
                for c in &mut self.grid[self.cy * self.cols + start..self.cy * self.cols + end] {
                    *c = Cell::default();
                }
            }
            _ => {}
        }
    }
}

/// Parser + session: holds the vte::Parser state (must live long).
pub struct Session {
    parser: Parser,
    pub term: Terminal,
}

impl Session {
    pub fn new(cols: usize, rows: usize) -> Self {
        Session { parser: Parser::new(), term: Terminal::new(cols, rows) }
    }

    /// Feed bytes from the PTY; the parser updates the grid.
    pub fn feed(&mut self, bytes: &[u8]) {
        for &b in bytes {
            let term = &mut self.term;
            self.parser.advance(term, b); // vte 0.13: advance(performer, byte)
        }
    }
}
