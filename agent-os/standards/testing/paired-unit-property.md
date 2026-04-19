# Pair Deterministic + Property Tests

Each assertion gets two tests: one deterministic, one property-based, sharing prefix.

```rust
#[test]
fn bug_condition_unit_error_handling_pattern() { ... }

proptest! {
    #[test]
    fn bug_condition_spaces_accepted(name in space_containing_pattern_name()) { ... }
}
```

Rules: deterministic first (name the exact input); property second (generalize); shared prefix; strategy in a named fn.
