# Architecture

## Overview

This crate provides an immutable-style ordered map with a movable cursor. Each
mutation returns a new `LinkedMap<K, V>` instead of modifying the original
instance in place.

## Data Model

`LinkedMap` stores:

- `entries`: a `BTreeMap<K, Arc<V>>` for keyed lookup with shared value storage
- `order`: a `Vec<K>` that preserves iteration order and supports relative
  insertion, swapping, slicing, and reversal
- `current`: an optional key representing the active cursor position

Keys are unique. The core invariant is that every key in `order` must exist in
`entries`, and every key in `entries` must appear exactly once in `order`.

## Behavioral Mapping

The Rust crate preserves the key behaviors of the source JavaScript package:

- ordered insertion with `push`, `prepend`, `insert_after`, and `insert_before`
- immutable-style updates via returned copies
- cursor movement with `move_to`, `move_to_start`, `move_to_end`, `next`, and
  `prev`
- relative extraction and deletion with `get_between` and `delete_between`
- ordered iteration through `iter`, `values`, `reduce`, and `reduce_right`

The public API is adapted to Rust idioms:

- construction uses `LinkedMap::new()`, `LinkedMap::from_entries(...)`, or the
  `linked_map!` macro
- lookups use explicit methods like `get` and `current_value` instead of a
  JavaScript-style optional argument overload
- most read-only APIs work with any `V`; only materialization helpers like
  `to_vec` and `to_map` require `V: Clone`

## Tradeoffs

This implementation favors correctness and a small API surface over deeply
optimized persistent data structures. Shared `Arc<V>` storage avoids cloning
values for cursor-only copies and read-heavy transformations, while operations
that change keys or ordering still clone the map structure and order vector.

## Tooling

Coverage is generated with `cargo-llvm-cov`. The repository includes a local
helper script at `scripts/coverage.sh`, and GitHub Actions runs separate build,
test, and coverage jobs. The coverage job publishes an LCOV artifact for CI
inspection.
