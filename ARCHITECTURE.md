# Architecture

## Overview

This crate provides an immutable-style ordered map with a movable cursor. Each
mutation returns a new `LinkedMap<K, V>` instead of modifying the original
instance in place.

## Data Model

`LinkedMap` stores:

- `entries`: a `BTreeMap<K, V>` for keyed lookup
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
- values are generic over `K` and `V`, with `K: Ord + Clone` and `V: Clone`

## Tradeoffs

This implementation favors correctness and a small API surface over structural
sharing. Operations that change order clone the internal map and order vector.
That keeps the semantics close to the original project while staying simple and
predictable in Rust.
