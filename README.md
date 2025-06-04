# Bitsong Accounts: A Tokenized Account Framework

This implementation is a compatible instance of [sg-names](https://github.com/public-awesome/names) for Bitsong. To the stargaze contributors, thank you for setting the tone with these!
<!-- ##  [API Docs](./API.md) -->
 

## [Account Marketplace](./contracts/bs721-account-marketplace/README.md)
The secondary marketplace for accounts. Accounts are automatically listed here once they are minted.

## [Account Minter](./contracts/bs721-account-minter/README.md)
Account minter is responsible for minting, validating, and updating accounts and their metadata.

## [Bs721-Account](./contracts/bs721-account/README.md)
A cw721 contract with on-chain metadata for an account.

## Scripting Library
In this repo are [cw-orchestrator scripts](../../scripts/src/bin/manual_deploy.rs) that highlight this first step.

### Compile the contracts 
```sh
make optimize
```
### Test the workspace: 
```sh
cargo test
```
### Cw-Orchestrator
To run the integration tests:
```sh 
cd scripts/ && cargo test
```
### Deploy 
To learn more about the deployment scripts, [check here](./scripts/README).

## DISCLAIMER

BITSONG CODE IS PROVIDED “AS IS”, AT YOUR OWN RISK, AND WITHOUT WARRANTIES OF ANY KIND. No developer or entity involved in creating or instantiating Bitsong smart contracts will be liable for any claims or damages whatsoever associated with your use, inability to use, or your interaction with other users of Bitsong, including any direct, indirect, incidental, special, exemplary, punitive or consequential damages, or loss of profits, cryptocurrencies, tokens, or anything else of value. Although Discover Decentralization DAO, and it's members configured existing code for the accounts, it does not own or control the Bitsong network.

