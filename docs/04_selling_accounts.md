# Selling Accounts

- marketplace powers the sale of accounts. Users can make bids on accounts, and account owners can accept bids. Bids must include the funds, which are held in escrow until a bid is accepted on an account, to which all unaccepted bids will have their funds returned.

## Account Cooldown

All bids accepted will begin the account token cooldown workflow. This is a saftey feature to protect the owner of an account from possibly unintentionally
accepting a bid. This puts the account token in an escrow stage, where after the cooldown period of 14 days, the account token sale can be finalized, which will complete the ownership transfer to the bidder and the payment to the previous owner.

### Cancelling Cooldown Period

In the scenario an account owner who has accepted a bid wants to cancel the purchase, they may do so by calling the `CancelCooldown` entrypoint. They are required to included a cooldown cancel fee, which is split between the bidder and the bitsong development team.

### Bid Refunds

There are a number of scenarios where existing bids are removed and assets included in bids are returned to bidders. To prevent reaching gas limits on refunding bids, a temporary caching state keeping records of any bids that may need to be refunded by an account is kept. Anyone is able to have the contract process these cached bids for a token id by calling the `CheckedRemoveBids` marketplace entrypoint.

## Abstract Account Ownership Retention Guarantees

When an account is transferred, if it is making use of the abstract-account feature, the contract ensures the account token is still the ownership token of the associated abstract account contract. If it is, we retain the ownership details in the metadata, and if not we disable the feature. In the scenario an account token is in cooldown and is the ownership token for an abstract account, upon finalizing the cooldown the contract does one final check ensuring the token is still indeed in use for ownership verification of the abstract account. If it is not, meaning that during the time a bid was accepted and the time the bid is being finalized, the abstract owner changed the ownership configuration of the account, the contract will refund the bidder, but still transfer the account token to the bidder as well.

> This ensures the new owner will always retain the ownership of an abstract account associated to this token id during its purchase.
