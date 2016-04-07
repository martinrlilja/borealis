#!/bin/bash

set -ex

if [ $TRAVIS_RUST_VERSION = nightly ]
then
    (cd borealis && cargo build --verbose --features nightly)
    (cd borealis && cargo test --verbose --features nightly)
    (cd borealis_codegen && cargo build --verbose)
    (cd borealis_codegen && cargo test --verbose)
else
    (cd borealis && cargo build --verbose)
    (cd borealis && cargo test --verbose)
fi
