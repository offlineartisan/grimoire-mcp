# Named Proptest Strategies

Each input space gets a named function.

```rust
fn space_containing_pattern_name() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z0-9 _-]{1,100}")
        .unwrap()
        .prop_filter("must contain at least one space", |s| s.contains(' '))
}
```

Rules: named `fn` returning `impl Strategy<Value = T>`; doc-commented invariants; name describes the subset; share one strategy across tests.
