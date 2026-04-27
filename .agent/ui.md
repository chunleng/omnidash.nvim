# UI Architecture

```
View          ← layout of Panels, user "page"
 └ Panel      ← owns NvimBuffer + NvimWindow, hosts Widgets
     └ Widget ← renders into NvimBuffer (no window)
         └ NvimBuffer / NvimWindow ← raw API wrappers (nvim_primitives)
```

| Layer | Role | Current |
|-------|------|---------|
| **View** | Top-level page. Owns Panel layout, wires Widgets. | `ui/mod.rs` |
| **Panel** | Owns buffer + window. Split/float/tile surface. Hosts Widgets. | `ui/panels/` |
| **Widget** | Embeddable control. Renders into buffer, no window. | `ui/widget/` |
| **nvim_primitives** | Thin `nvim_buf_*` / `nvim_win_*` wrappers. | `ui/nvim_primitives/` |

## Target structure

```
src/ui/
  mod.rs
  panels/
    mod.rs
    fixed.rs
    swappable.rs
  widget/
    mod.rs
    display.rs
  nvim_primitives/
    mod.rs
    buffer.rs
    window.rs
```
