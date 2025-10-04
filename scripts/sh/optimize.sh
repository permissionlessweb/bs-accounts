#!/usr/bin/env bash

if [[ $(arch) == "arm64" ]]; then
    image="cosmwasm/optimizer-arm64"
else
    image="cosmwasm/optimizer"
fi

# Optimized builds
docker run --rm -v "$(pwd)":/code \
--mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
${image}:0.17.0

# just build the ownership-verifier
# docker run --rm -v "$(pwd)":/code \
#   --mount type=volume,source="devcontract_cache_ownership_verifier",target=/code/contracts/ownership-verifier/target \
#   --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
#   cosmwasm/optimizer-arm64:0.17.0 \
#   ./contracts/ownership-verifier