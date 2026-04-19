# Fail-Soft Pattern Loading

Discovery drops unparseable files. Never panic, never block startup on one bad file.

```rust
fs::read_dir(&patterns_dir)
    .ok().into_iter().flatten()
    .flat_map(|e| e.ok())
    .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
    .filter_map(|e| load_pattern(&e.path()))
    .collect()
```

Channel rule: stdout pristine JSON only; stderr `tracing::warn!` for drops. Targeted lookups still error normally — fail-soft is discovery-only.
