# TUI Mode State Machine

Input modes are an enum; `handle_key` matches on mode first, then key.

```rust
enum InputMode { Normal, Searching }

match self.mode {
    InputMode::Normal => match code { ... },
    InputMode::Searching => match code { ... },
}
```

Rules: one enum variant per mode (no bool flags); outer match on mode, inner on `KeyCode`; `_ => {}` eats unmapped keys.
