# Filter as Indices

Filtered views are `Vec<usize>` into the owned `Vec<Pattern>`. Never clone, never borrow.

```rust
struct App {
    patterns: Vec<Pattern>,
    filtered_indices: Vec<usize>,
}
```

Lookup: `&self.patterns[filtered_indices[i]]`. After recompute, reset dependent state.
