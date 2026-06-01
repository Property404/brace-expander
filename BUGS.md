# Bugs

## Performance bugs

* The parsing stage is very slow on inputs like `{{{{{{{{{{{{{{{{{{{{{{{{{{{{{`
   * Requires `2^n - 1` calls to `parse_expansion`
   * Recursion depth is another factor to consider

## Differences from Bash

BraceExpander strives for (at least optional) compatibility with Bash, but
there are some differences

* `{},}` and `{},}text` will resolve on neither Bash nor BraceExpander.
  Bash will, however, expand `text{},}`. This seems niche and I don't really
  understand Bash's parsing logic here
* BraceExpander doesn't see padding if a minus sign is in front
  Bash: `{-02..2}` => `-02 -01 000 001 002`
  Brace Expander: `{-02..2}` -> `-2 -1 0 1 2`
  Note Bash counts the minus sign as padding

## Missing features

* Bash parity: quote parsing
* Bash parity: prevent tokenizing after `$`
* Ability to turn off whitespace parsing
