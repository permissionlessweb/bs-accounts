
# 2. Managing An Account

Once an account it minted, the nft owner can add details to the nft for futher customization of their account token. **Bitsong Accounts reserve the `token_uri` object for its reverse mapping to Abstract Accounts.** Due to this, the account minter initially sets `token_uri` to None.

## Associating an Address to an Account Token

### Interoperable Design

When you buy a Bitsong Account, you are really getting a account on _every_ Cosmos chain that is using the same type address as Bitsong.

```
jimi -> D93385094E906D7DA4EBFDEC2C4B167D5CAA431A (in hex)
```

#### Bitsong Use Of Coin Type 639

Cosmos-sdk chains have key derivation support, which is why chains using different coin types default to resolves a bech32 addr differently. The addresses still in control of the private key, it is just the current nature of our wasmvm that introduces this discrepency when parsing between human readable addrs of a public key from chains with different coin types. _This is also how a single key can control multiple accounts on a single chain_.

> For example, a user may want to associate different chains accounts with their bs-account token, even potentially an address derived from a completely different public key.

### Mapping an **outside address** to your `bitsong1..` addr

`REVERSE_MAP_KEY` is the storage object used to map an outside address (non `bitsong1...`) as the value of various storage keys. This enables effecient data retrieval from the contract for multiple items in the storage mapped to a specific address.

For this support, the entrypoint `ExecuteMsg::UpdateMyReverseMapKey` exists, where a user can submit multiple external addresses, signed via ADR036  method. This way, front ends and indexers can make use of `QueryMsg::ReverseMapAccount` query by requesting with an address that will resolve the account token associated with the address.

> a maximum of 10 external accounts may be mapped to a token, and upon any transfer  of an account, these mappings are removed from the store.

#### Arbitrary Cosmos Signature

In order to avoid someone mapping a wallet not under control to their own, we make use of the generic signature verification spec to verify a private key signature from the out-side address was generated, containing the bitsong wallet address.

Now this can be resolved per chain, but notice the discrepency with chains using differnet coin types:

```
jimi.bitsong  -> bitsong1myec2z2wjpkhmf8tlhkzcjck04w25sc6ymhplz
# will be incorrect due to mismatch slip44 coin types with cosmos hub and bitsong 
jimi.cosmos -> cosmos1myec2z2wjpkhmf8tlhkzcjck04w25sc6y2xq2r
```

Chains that use different account types or key derivation paths has support with the use of the custom entry point `UpdateMyReverseMapKey`, which lets mapping and retrieval of external accounts quick and compatible without any custom cryptographic library.

#### Image NFT

```json
{"update_image_nft": {"account": "<eret-skeret>", "nft": {"collection": "","token_id":""} }}
```

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

## Abstract Account Support

### Overview

We have support for associating an bitsong account token with an abstract account contract. This makes building front ends that want to prioritize EOA contracts as recipients for funds,assets, and other use-cases.

### Enabling Support

During an account token mint, a user may set in the Metadata struct that this token serves as an ownership key for an abstract account, via `account_ownership`. This will activate a internal circuit to mitigate any undesirable actions that may lead to a user being locked out of their abstarct account.

### Associating An Abstract Account

This is done by the user calling the `ExecuteMsg::AssociateAddress`

Once associated, queries the entrypoint `QueryMsg::AssociatedAddress` with the account token id to resolve the abstract account associated with the token-id.

### Updating Association 

 