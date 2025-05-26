## 1. Mint An Account 

#### Initial Fees
```
5+ chars = 100 BITSONG
4 chars  = 1,000 BITSONG
3 chars  = 10,000 BITSONG
``` 

When an account is minted, it is done by calling the account minter contract:
```rs

``` 
It must be after the mint time set by admins of the minter, and the account name is set as the token id for that account. 



Before being minted the account name is validated to be a length within the range set by admins,and  payments required are validated to have been sent.If so all mint proceeds are burnt. Only native BTSG is accepted currently.

**Bitsong Accounts reserve the `token_uri` object for its reverse mapping to external addresses.** Due to this, the account minter initially sets `token_uri` to None. 

Along with the mint message, the minter forms a default `SetAsk` msg to the Account Marketplace, letting the marketplace know that a new token has been minted. *NOTE: This seems sub-optimal when considering these nfts are cornerstones to on-chain accounts. This should be improved.*


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