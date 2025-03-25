# Bitsong Accounts: A Tokenized Account Framework

This implementation is a compatible instance of [sg-names](https://github.com/public-awesome/names) for Bitsong. To the stargaze contributors, thank you for setting the tone with these!
<!-- ##  [API Docs](./API.md) -->

## Architecture

### [Account Marketplace](./contracts/bs721-account-marketplace/README.md)
The secondary marketplace for accounts. Accounts are automatically listed here once they are minted.

### [Account Minter](./contracts/bs721-account-minter/README.md)
Account minter is responsible for minting, validating, and updating accounts and their metadata.

### [Bs721-Account](./contracts/bs721-account/README.md)
A cw721 contract with on-chain metadata for an account.

#### Thoughts On Account Token Security Design 
Since account tokens are the key for smart accounts, it is essential to keep in mind the security vulnurabilities that exists in a multi contract system that involved the smart contract of your Bitsong Account Token. Below will list a few obvious examples, but as always security is never static.

### Centralized Smart Contract Ownership & Migration Attack
If the code-id of the accounts used has a global contract admin, then the possible risk of the admins integrity being compromised, and an attack may occur on wallets. Bitsong mitigates this risk by having the governance module set as the admin able to migrate a contract for all of the contracts used in the accounts framework. 


There is still a risk of any wallet address compromized that is also authorized to execute actions as the module, however there are none. 

### Undesired Operator Or Approval Authorization
Whenever browsing dapps with the account that is currently under ownership of the account token, it is crucial to keep concious of any unwantedauthorized operator or approval messages during interations with them, as this may result in full compromise of your account without delays  including other smart contracts, then this may put  account tokens at risk. 



## Technical Workflow

In this repo are [cw-orchestrator scripts](../../scripts/src/bin/manual_deploy.rs) that highlight this first step.

### 1. Mint An Account 

#### Initial Fees
```
5+ chars = 100 BITSONG
4 chars  = 1,000 BITSONG
3 chars  = 10,000 BITSONG
``` 

When an account is minted, it is done by calling the account minter contract. It must be after the mint time set by admins of the minter, and the account name is set as the token id for that account. 

Before being minted the account name is validated to be a length within the range set by admins,and  payments required are validated to have been sent.If so all mint proceeds are burnt. Only native BTSG is accepted currently.

**Bitsong Accounts reserve the `token_uri` object for its reverse mapping to external addresses.** Due to this, the account minter initially sets `token_uri` to None. 

Along with the mint message, the minter forms a default `SetAsk` msg to the Account Marketplace, letting the marketplace know that a new token has been minted. *NOTE: This seems sub-optimal when considering these nfts are cornerstones to on-chain accounts. This should be improved.*

<!-- #### Abstract Account Support 
Bitsong accounts uses custom metadata that specificies whether or not this token is being used for an abstract account.   -->

### 2. Managing An Account 
Once an account it minted, the nft owner can add details to the nft for futher customization of their account token. 

#### Associate Address
`REVERSE_MAP_KEY` is the storage object used to map a `bitsong1...` addr as the value of various storage keys. This enables effecient data retrieval from the contract for multiple items in the storage mapped to a specific address. 

### Interoperable Design
When you buy a Bitsong Account, you are really getting a account on _every_ Cosmos chain. Any chain can lookup a account by its local address over IBC. Similarly, any chain can mint a account over IBC that resolves to a local address. 

```
jimi -> D93385094E906D7DA4EBFDEC2C4B167D5CAA431A (in hex)
```

#### Bitsong Use Of Coin Type 639
Resolving an address for different chains is done with logic that includes the chains coin type used, in order to determine the human redimal representation of the account that a private key has control over. This is how a single key can control multiple accounts on multiple chains. 



Now this can be resolved per chain:
```
jimi.bitsong  -> bitsong1myec2z2wjpkhmf8tlhkzcjck04w25sc6ymhplz
# will be incorrect due to mismatch slip44 coin types with cosmos hub and bitsong 
jimi.cosmos -> cosmos1myec2z2wjpkhmf8tlhkzcjck04w25sc6y2xq2r
```

Chains that use different account types or key derivation paths has support with the use of the custom entry point `UpdateMyReverseMapKey`, which lets mapping and retrieval of external accounts quick and compatible without any custom cryptographic library. 

#### Image NFT 

#### Text Record 
Accounts are designed to be as flexible as possible, allowing generic `TextRecord` types to be added. Each record has a `verified` field that can only be modified by a verification oracle. For example, a Twitter verification oracle can verify a user's signature in a tweet, and set `verified` to `true`. Text records can also be used to link the account to other name services such as ENS.

`profile_nft` points to another NFT with on-chain metadata for profile information such as bio, header (banner) image, and follower information. This will be implemented as a separate collection.

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
### 3. Transferring Ownership of An Account 



## DISCLAIMER

BITSONG CODE IS PROVIDED “AS IS”, AT YOUR OWN RISK, AND WITHOUT WARRANTIES OF ANY KIND. No developer or entity involved in creating or instantiating Bitsong smart contracts will be liable for any claims or damages whatsoever associated with your use, inability to use, or your interaction with other users of Bitsong, including any direct, indirect, incidental, special, exemplary, punitive or consequential damages, or loss of profits, cryptocurrencies, tokens, or anything else of value. Although Discover Decentralization DAO, and it's members configured existing code for the accounts, it does not own or control the Bitsong network.

## Compile the contracts 
```sh
make optimize
```
## Test the workspace: 
### Cargo
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

## Code Coverage 
