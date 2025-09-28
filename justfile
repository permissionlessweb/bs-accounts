#!/bin/bash

fmt:
	cargo fmt --all --check
schema:
	sh scripts/sh/schema-and-codegen.sh

lint:
	cargo clippy --fix --tests -- -D warnings
   
optimize:
	sh scripts/sh/optimize.sh
	
 
test:
    cargo test --locked

# publish:
#     #!/usr/bin/env bash
#     crates=(
 
#     )

#     for crate in "${crates[@]}"; do
#       cargo publish -p "$crate"
#       echo "Sleeping before publishing the next crate..."
#       sleep 30
#     done

