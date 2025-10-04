# Selling Accounts

- marketplace powers the sale of accounts. Users can make bids on accounts, and account owners can accept bids. Bids must include the funds, which are held in escrow until a bid is accepted on an account, to which all unaccepted bids will have their funds returned.

## Account Cooldown

## Abstract Account Ownership Retention Guarantees

When an account is transferred, if it is making use of the abstract-account feature, the contract ensures the account token is still the ownership token of the associated abstract account contract. If it is, we retain the ownership details in the metadata, and if not we disable the feature. In the scenario an account token is in cooldown and is the ownership token for an abstract account, upon finalizing the cooldown the contract does one final check ensuring the token is still indeed in use for ownership verification of the abstract account. If it is not, meaning that during the time a bid was accepted and the time the bid is being finalized, the abstract owner changed the ownership configuration of the account, the contract will refund the bidder, but still transfer the account token to the bidder as well.

> This ensures the new owner will always retain the ownership of an abstract account associated to this token id during its purchase.