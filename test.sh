#!/bin/bash

cargo test --workspace --lib --bins --tests --all-features -- --test-threads=1
