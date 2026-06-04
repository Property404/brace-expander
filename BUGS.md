# Bugs

## Design

* Parsing stage is recursive, this should probably be refactored

## Differences from Bash

BraceExpander strives for (at least optional) compatibility with Bash, but
there are some differences

* `{},}` and `{},}text` will resolve on neither Bash nor BraceExpander.
  Bash will, however, expand `text{},}`. This seems niche and I don't really
  understand Bash's parsing logic here

## Missing features

* Bash parity: quote parsing
* Bash parity: prevent tokenizing after `$`
* Ability to turn off whitespace parsing
