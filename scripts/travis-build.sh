#!/bin/bash

set -ex

(cd borealis && cargo build --verbose)
(cd borealis && cargo test --verbose)

if [ $TRAVIS_RUST_VERSION = nightly ]
then
    (cd borealis_codegen && cargo build --verbose)
    (cd borealis_codegen && cargo test --verbose)
fi
