# Abstract Account: Namespace middleware support

## Requirements

- accept hooks from the bs-account marketplace
- register namespaces in abstract:registry
- allow admin to claim all balances set into it
- route all registry related admin functions through itself (TODO)

## Things to Keep Concious

- abstract framework registry contract  MUST have enabled security so only registry admin can set namespaces
- account owners can forgoe their namespace in the registry contract.
- `AccountId` will be manually defined by the marketplace `Ask.id`. This does not communicate with the abstract account registries global account sequence counter, but that is okay cause all abstract accounts store their AccountId in their own internal state for reference. ACCOUNT_ID

### On framework deployment

- fee on account market and namespace registration fee must be identical
- middleware must be registered as amind for abstract registry

### On Account Token Mint (AskHookMsg::Create)

- reserve namespace for the token-id & ask-id (EOA may not have been created, but nft owner is reserved this namespace in our registry)

### On Account Token Burn (AskHookMsg::Delete)

- forgoe namespace from registry as ownership key is not used for any accounts, and if a new token id is minted

### On Account Sale (During FinalizeCooldown)

- if buyer and seller are the same, we know that the account associated with this namespace is not accurate, so we must forgoe & reset the namespace to prevent any association of an prior account with the namespace in the registry.

## On Bid Creation ()

- we can query the registry that the token id is registered in the namespaces, and if it is claimed we can assert that the account address is identical to the token_uri for the nft, and prevent a bid from occuring if not.

## On Bid Removal ()

// noop

## On Bid Update ()

// noop
