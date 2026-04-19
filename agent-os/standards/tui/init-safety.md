# TUI Init & Restore Safety

Validate preconditions BEFORE `ratatui::init()`. Pair init/restore. Install panic hook that restores terminal.

```rust
let original_hook = std::panic::take_hook();
std::panic::set_hook(Box::new(move |info| {
    ratatui::restore();
    original_hook(info);
}));
let mut terminal = ratatui::init();
// ...
ratatui::restore();
```
