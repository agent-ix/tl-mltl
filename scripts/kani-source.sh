#!/bin/sh
# Print the source file that holds the Kani harness with this qualified name.
set -eu
case "$1" in
    wire::*) echo src/wire/kani_wire.rs ;;
    future::horizon::*) echo src/future/horizon.rs ;;
    *) echo src/future/kani_evaluate.rs ;;
esac
