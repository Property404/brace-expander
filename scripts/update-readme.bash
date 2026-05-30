#!/usr/bin/env bash
# Updates README.md using the crate-level documentation in `lib.rs`
set -e

main() {
    # Rewind back to crate root
    while ! stat Cargo.toml > /dev/null; do
        cd ..
    done

    echo -e "# brace-expander\n" > README.md
    sed -n 's/^\/\/!//gp' src/lib.rs | sed 's/^ //g'  >> README.md
}

main
