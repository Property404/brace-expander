# Bugs

## Performance bugs

* The parsing stage is very slow on inputs like `{{{{{{{{{{{{{{{{{{{{{{{{{{{{{`
   * Requires `2^n - 1` calls to `parse_expansion`
   * Recursion depth is another factor to consider

## Differences from Bash

BraceExpander strives for (at least optional) compatibility with Bash, but
there are some differences

* `{},}` and `{},}text` by itself will not resolve on either Bash nor
  BraceExpander. Bash will, however, expand `text{},}` This seems niche and I
  don't really understand Bash's parsing logic here

## Missing features

* Bash parity: quote parsing
* Bash parity: prevent tokenizing after `$`
* Ability to turn off whitespace parsing
