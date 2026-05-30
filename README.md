# brace-expander

Library to support bash-like brace expansions

[![Repository](https://img.shields.io/badge/github-brace--expander-/)](https://github.com/Property404/brace-expander)
[![crates.io](https://img.shields.io/crates/v/brace-expander.svg)](https://crates.io/crates/brace-expander)
[![Documentation](https://docs.rs/brace-expander/badge.svg)](https://docs.rs/brace-expander)

## Examples

```rust
use brace_expander::BraceExpander;
let be = BraceExpander::default();

// Basic cartesian product
assert_eq!(be.expand("{a,b}").unwrap().join(" "), "a b");
assert_eq!(be.expand("hello_{a,b}").unwrap().join(" "), "hello_a hello_b");
assert_eq!(be.expand("G{o,u}{b,g}").unwrap().join(" "), "Gob Gog Gub Gug");

// Nested product
assert_eq!(be
    .expand("{{a,b}{c,{e,f}},g}")
    .unwrap()
    .join(" "), "ac ae af bc be bf g");

// Numeric expansion
assert_eq!(be.expand("thing{1..3}").unwrap().join(" "), "thing1 thing2 thing3");

// Or backwards
assert_eq!(be.expand("thing{3..1}").unwrap().join(" "), "thing3 thing2 thing1");

// char expansion
assert_eq!(be.expand("{A..C}").unwrap().join(" "), "A B C");

// Step by
assert_eq!(be.expand("{A..E..2}").unwrap().join(" "), "A C E");

// Leading zeroes
assert_eq!(be.expand("Agent{006..008}").unwrap().join(" "), "Agent006 Agent007 Agent008");

```

## License

MIT or Apache-2.0

Pull requests encouraged
