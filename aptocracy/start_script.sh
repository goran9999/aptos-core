#!/bin/bash

export RUSTFLAGS="--cfg tokio_unstable"

/root/.cargo/bin/cargo run
