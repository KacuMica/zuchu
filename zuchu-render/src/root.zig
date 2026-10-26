const std = @import("std");

// zuchu-render — the render core of Zuchu, written in Zig.
// Compiled into a static library (libzuchu_render.a) and linked from Rust
// via the C ABI (build.rs in the zuchu crate).
//
// Zig's role: the hot path of rendering — compositing the cell grid into
// a vertex buffer which is then handed to OpenGL from Rust. Data crosses
// the boundary as flat arrays (no pointers to structs) to keep the ABI simple.

/// Cell attributes (numbers match term.rs Cell in Rust 1:1)
pub const CellAttrs = packed struct(u32) {
    fg: u8, // palette index 0..=15
    bg: u8,
    bold: bool,
    _pad: u23 = 0,
};

/// Composites the grid into an RGBA buffer (linear, 0..=255).
/// rows x cols cells, one symbol per cell (v0.1: no ligatures).
pub fn composite(
    cells: []const u8,  // cell text bytes (v0.1: ASCII path only)
    attrs: []const u32, // packed CellAttrs
    bg: [3]u8,
    fg: [3]u8,
    out: []u8, // rows*cols*4 RGBA
    cols: usize,
    rows: usize,
) void {
    var row: usize = 0;
    while (row < rows) : (row += 1) {
        var col: usize = 0;
        while (col < cols) : (col += 1) {
            const i = row * cols + col;
            const attr = @as(CellAttrs, @bitCast(attrs[i]));
            // v0.1: no glyph atlas yet — plain color fill per attribute
            const color: [3]u8 = if (attr.fg != 0) fg else bg;
            out[i * 4 + 0] = color[0];
            out[i * 4 + 1] = color[1];
            out[i * 4 + 2] = color[2];
            out[i * 4 + 3] = 255;
        }
    }
}

test "composite fills color" {
    const cells = [_]u8{ 'a', 'b', 'c', 'd' };
    const attrs = [_]u32{ 0, 0, 512, 0 }; // third cell: fg=2
    var out: [16]u8 = undefined;
    composite(&cells, &attrs, .{ 10, 10, 10 }, .{ 200, 200, 200 }, &out, 2, 2);
    try std.testing.expectEqual(@as(u8, 10), out[0]);
    try std.testing.expectEqual(@as(u8, 200), out[8]);
}
