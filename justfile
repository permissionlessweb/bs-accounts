#!/bin/bash

fmt:
	cargo fmt --all --check
schema:
    #!/usr/bin/env bash
    sh scripts/tools/schema.sh

lint:
	cargo clippy --fix --tests -- -D warnings
   
build:
 
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

