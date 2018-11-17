# refmove: an experimental implementation of library-level by-move references

[![Build Status](https://travis-ci.org/qnighy/rust-refmove.svg?branch=master)](https://travis-ci.org/qnighy/rust-refmove)
[![Latest Version](https://img.shields.io/crates/v/refmove.svg)](https://crates.io/crates/refmove)

This crate contains an experimental implementation of library-level
by-move references.

It will enable you to use `self: RefMove<Self>` to pass your trait
object by value, even without allocation.

See [#48055][#48055] for another approach to allow by-value trait objects.

[#48055]: https://github.com/rust-lang/rust/issues/48055

## Usage

### Borrowing

```rust
#![feature(nll)]
extern crate refmove;
use refmove::{Anchor, AnchorExt, RefMove};

...

// Borrowing from stack
let _: RefMove<i32> = 42.anchor().borrow_move();

// Borrowing from box
let _: RefMove<i32> = Box::new(42).anchor_box().borrow_move();
```

### Extracting

```rust
#![feature(nll)]
extern crate refmove;
use refmove::{Anchor, AnchorExt, RefMove};

...

fn f(x: RefMove<String>) {
    // Borrowing by dereference
    println!("{}", &x as &String);

    // Move out ownership
    let _: String = x.into_inner();
}
```
