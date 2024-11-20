# Bitsong Accounts: A Tokenized Account Framework

## API Docs

See [API Docs](./API.md)

## Architecture

Accounts are stored without the TLD so they can be mapped to _any_ Cosmos address. All accounts can be resolved to an address that is derived via the same Cosmos key derivation path as Bitsong (639).

When you buy a Bitsong Account, you are really getting a account on _every_ Cosmos chain. Any chain can lookup a account by its local address over IBC. Similarly, any chain can mint a account over IBC that resolves to a local address.

```
bobo -> D93385094E906D7DA4EBFDEC2C4B167D5CAA431A (in hex)
```

Now this can be resolved per chain:
```
bobo.bitsong  -> bitsong1myec2z2wjpkhmf8tlhkzcjck04w25sc6ymhplz
# will be incorrect due to mismatch slip44 coin types with cosmos hub and bitsong bobo.cosmos -> cosmos1myec2z2wjpkhmf8tlhkzcjck04w25sc6y2xq2r
```

Chains that use different account types or key derivation paths can have support added later by migrating contracts. Account contracts are community-owned contracts that can be migrated by Bitsong governance.

<!-- ### Annual Auction -->
- [x] When a account is minted it is automatically listed in Account Marketplace
- [x] Owner can accept the top bid at any time
<!-- - [ ] After 1 year, owner has to pay 0.5% of the top bid to keep the account
- [ ] If a bid is placed for 4 weeks, account value rises to this value
- [ ] If fee is not paid, account is transferred to the bidder
- [ ] If there are no bids, there is a minimum fee to keep the account based on the number of characters
- [ ] Cap annual fee at X per year -->

## Initial Fees

```
5+ chars = 100 BITSONG
4 chars  = 1,000 BITSONG
3 chars  = 10,000 BITSONG
```

## Contracts

### [Bs721-Account](./contracts/bs721-account/README.md)

A cw721 contract with on-chain metadata for an account.

Types used in metadata:

```rs
pub struct TextRecord {
    pub account: String,           // "twitter"
    pub value: String,          // "shan3v"
    pub verified: Option<bool>  // verified by oracle
}
```

```rs
pub struct Metadata {
    pub image_nft: Option<NFT>,
    pub record: Vec<TextRecord>,
}
```

Accounts are designed to be as flexible as possible, allowing generic `TextRecord` types to be added. Each record has a `verified` field that can only be modified by a verification oracle. For example, a Twitter verification oracle can verify a user's signature in a tweet, and set `verified` to `true`. Text records can also be used to link the account to other name services such as ENS.

`profile_nft` points to another NFT with on-chain metadata for profile information such as bio, header (banner) image, and follower information. This will be implemented as a separate collection.

### [Account Minter](./contracts/account-minter/README.md)

Account minter is responsible for minting, validating, and updating accounts and their metadata.

### Compile the contracts 
```sh
make optimize
```

<!-- ### [Account Marketplace](./contracts/account-market/README.md)

The secondary marketplace for accounts. Accounts are automatically listed here once they are minted. -->

<!-- ### [Whitelist](./contracts/whitelist-updatable/README.md)

Whitelist allows for flexible updating to add / remove addresses at any point in minting. Also adds helper to account for whitelist minting limits. -->

## Cw-Orchestrator 

To run the integration tests:
```sh 
cd scripts/ && cargo test
```

To learn more about the deployment scripts, [check here](./scripts/README).


#### Coverage

To run code coverage checks first install `tarpaulin` (`cargo install cargo-tarpaulin`), then run `make coverage` from the project root.

<!-- ## DISCLAIMER

STARGAZE SOURCE CODE IS PROVIDED “AS IS”, AT YOUR OWN RISK, AND WITHOUT WARRANTIES OF ANY KIND. No developer or entity involved in creating or instantiating Stargaze smart contracts will be liable for any claims or damages whatsoever associated with your use, inability to use, or your interaction with other users of Stargaze, including any direct, indirect, incidental, special, exemplary, punitive or consequential damages, or loss of profits, cryptocurrencies, tokens, or anything else of value. Although Public Awesome, LLC and it's affilliates developed the initial code for Stargaze, it does not own or control the Stargaze network, which is run by a decentralized validator set. -->
