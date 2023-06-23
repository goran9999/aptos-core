#!/bin/bash


RUSTFLAGS="--cfg tokio_unstable" /root/.cargo/bin/cargo  run  -p aptos-node --features "indexer" -- -f ./fullnode.yaml  
