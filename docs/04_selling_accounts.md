# Selling Accounts

- marketplace powers the sale of accounts. Users can make bids on accounts, and account owners can accept bids. Bids must include the funds, which are held in escrow until a bid is accepted on an account, to which all unaccepted bids will have their funds returned.

## Account Cooldown

## Abstract Account Ownership Retention

When an account is transferred, if it is making use of the abstract-account feature, the contract ensures the account token is still the ownership token of the associated abstract account contract. If it is, we retain the ownership details in the metadata, and if not we disable the feature.

> This ensures the new owner will always retain the ownership of an abstract account associated to this token id during its purchase.
