-- Zuchu config — ~/.config/zuchu/config.lua
--
-- This is a regular Lua file: it is EXECUTED at Zuchu startup, not merely
-- parsed. Conditions, functions and computations are allowed. The final
-- result is a table returned via `return`.

local M = {}

-- Shell path (kacuOS standard: zsh + OhMyZsh)
M.shell = "/bin/zsh"

-- Initial grid size (resized further at runtime)
M.cols = 100
M.rows = 30

-- Font (v0.2 adds full font discovery + fallback chain)
M.font = "monospace"
M.font_size = 12.0

-- Theme: RGB components, 0..1
M.theme = {
  bg = { 0.055, 0.055, 0.075 },
  fg = { 0.92, 0.92, 0.94 },
}

-- Example of dynamic theming (Lua full power at work):
-- if os.getenv("KACU_MODE") == "night" then
--   M.theme.bg = { 0.02, 0.02, 0.03 }
-- end

return M
