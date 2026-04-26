# LinkedMap

`LinkedMap` is an immutable-style ordered map for Rust. It combines keyed lookup
with stable insertion order, cursor navigation, and relative reordering
operations like `insert_after`, `insert_before`, `swap`, and `reverse`.

## Example

```rust
use linked_map::{linked_map, LinkedMap};

let linked = linked_map!(
    1 => "one",
    2 => "two",
    3 => "three",
);

let moved = linked
    .insert_after(&1, "between", 4)
    .move_to(&4);

assert_eq!(moved.current_value(), Some(&"between"));
assert_eq!(moved.to_vec(), vec![(1, "one"), (4, "between"), (2, "two"), (3, "three")]);
```

## API Highlights

- `LinkedMap::new()`
- `LinkedMap::from_entries(...)`
- `push`, `prepend`, `shift`, `pop`
- `insert_after`, `insert_before`, `insert_many_after`
- `swap`, `reverse`
- `get_between`, `delete_between`
- `move_to`, `move_to_start`, `move_to_end`, `next`, `prev`
- `iter`, `values`, `reduce`, `reduce_right`

## Coverage

Install `cargo-llvm-cov`, then run:

```bash
scripts/coverage.sh summary
scripts/coverage.sh html
```

The GitHub Actions workflow also runs build, test, and coverage jobs and uploads
the LCOV report as a workflow artifact.

## Architecture

Implementation details and design tradeoffs are documented in `ARCHITECTURE.md`.
