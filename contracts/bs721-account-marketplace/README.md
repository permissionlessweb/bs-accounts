# Account Marketplace 

## Marketplace Actions 
|Marketplace Actions | Description | 
| --- | --- | 
| `SetAsk` | List account NFT on the marketplace by creating a new ask. Only the account factory can call this. | 
| `RemoveAsk` | Remove account on the marketplace. Only the account collection can call this (i.e: when burned). | 
| `UpdateAsk` | Update ask when an NFT is transferred. Only the account collection can call this. | 
| `SetBid` | Place a bid on an existing ask. | 
| `RemoveBid` | Remove an existing bid from an ask. | 
| `AcceptBid` | Accept a bid on an existing ask. | 

## Admin Only Actions 
|Admin Only Actions | Description | 
| --- | --- | 
| `Setup` | Setup the marketplace with a minter and collection contract. | 

## Sudo Actions 
|Sudo Actions | Description | 
| --- | --- | 
| `UpdateParams` | Setup the marketplace with a minter and collection contract. | 
| `UpdateAccountFactory` | Setup the marketplace with a minter and collection contract. | 
| `UpdateAccountCollection` |   | 
| `AddAskHook` |   | 
| `AddBidHook` |   | 
| `RemoveBidHook` |   | 
| `AddSaleHook` |   | 
| `RemoveSaleHook` |   | 

## Marketplace Queries 
| Marketplace Queries | Description | 
| --- | --- | 
| `Ask` | Get the current ask for a specific NFT. | 
| `Asks` | Get all asks for a collection, with optional pagination. | 
| `AskCount` | Get the total count of all asks. | 
| `AsksBySeller` | Get all asks by a specific seller, with optional pagination. | 
| `Bid` | Get data for a specific bid. | 
| `BidsByBidder` | Get all bids by a specific bidder, with optional pagination. | 
| `Bids` | Get all bids for a specific NFT, with optional pagination. | 
| `BidsSortedByPrice` | Get all bids for a collection, sorted by price, with optional pagination. | 
| `ReverseBidsSortedByPrice` | Get all bids for a collection, sorted by price in reverse, with optional pagination. | 
| `BidsForSeller` | Get all bids for a specific seller, with optional pagination. | 
| `HighestBid` | Get the highest bid for a specific NFT. | 
| `AskHooks` | Show all registered ask hooks. | 
| `BidHooks` | Show all registered bid hooks. | 
| `SaleHooks` | Show all registered sale hooks. | 
| `Params` | Get the config for the contract. | 
| `Config` | Get the minter and collection. |