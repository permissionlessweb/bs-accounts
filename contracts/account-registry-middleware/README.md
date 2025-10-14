# Abstract Account: Namespace middleware support

## DO NOT USE: Support for middleware integration with abstract-framework is pending. This is experiental and does not yet function as anticipated

## Requirements/Goals
<!-- - accept all hook methods from the bs-account marketplace -->
- register namespaces in abstract:registry
- allow admin to claim all balances set into it
- route all registry related admin functions through itself

## Things to Keep Concious

- abstract framework registry contract MUST have enabled security so only registry admin can set namespaces
- account owners can forgoe their namespace in the registry contract.
- `AccountId` will be manually defined by the marketplace `Ask.id`. This does not communicate with the abstract account registries global account sequence counter.

| Event / Action                     | Hook Msg                   | Description |
|------------------------------------|----------------------------|-----------|
| Framework Deployment               | —                          | - `middleware` must be registered as admin for `registry` |
| Account Token Mint                 | `AskHookExecuteMsg::AskCreatedHook`       | -Mint account on behalf of token minter <br> - Reserve namespace for the token-id <br> - ask-id is account seq |
| Account Token Transfer             | `AskHookExecuteMsg::AskUpdatedHook`       | - assert `registry` namespace details are still aligned with `marketplace` state |
| Account Token Burn                 | `AskHookExecuteMsg::AskDeletedHook`       | Forgo namespace in `registry` since ownership key no longer exists<br>  |
| Account Sale (During FinalizeCooldown) | `SaleHookMsg`                     | If buyer and seller are the same, the associated account is invalid → forgo and reset namespace to prevent incorrect prior account association |
| Bid Creation                       | `BidHookExecuteMsg::BidCreatedHook`                          | Query registry to check if token ID is registered in namespaces<br>If claimed, verify that account address matches NFT's token URI<br>Prevent bid if mismatch |
| Bid Removal                        | `BidHookExecuteMsg::BidDeletedHook`                        | No operation (noop) |
| Bid Update                         |  `BidHookExecuteMsg::BidUpdatedHook`                          | No operation (noop) |

## Account Registry Middleware Issues Discovered

Our intended design to integrate within the existing framework involves a registry-middleware contract, participating as the admin of the registry contract, and accepting hooks coming from an account nft minting/marketplace framework, that is invoked on mints, bids on accounts, and transfers of nft account tokens.

### Issue #1: Manual Defining/Updating/Accessing Local Sequence

It would unlock more programabiliity to the frameowrk if we are able to access/set the account sequence in a non-incrementing manner. For example, we aim to use the ask-id of an accounts ownership token coming form the marketplace as the local account sequence when creating an account.  

### Issue #2 : Namespace Conflict Possibility On Registry Middleware Use

 A design goal is to have our abstract-framework instance deployed such that only an authorized registry admin may reserve and claim namespaces on behalf of users.

Currently, if the security_enabled features is true, then only the registry admin can claim namespaces on behalf of account_ids, however this does not prevent any account instantiator that wants to define a namespace to be reserved (but not claimed) during account creation can prevent the registry owner from claiming a namespace. This will prevent the middleware from claiming a namespace on behalf of an account-id if an account is created using an indentical namespace as a token-id minted (token-ids are being set as the namespace in our middleware implementaion)
Our reasoning for implementing this feature is to have full integration of existing account tokens and namespace for reverse lookups between the account token minting/marketplace framework, and the abstract-framework registry.

### Design Questions

#### Q: what happens when an account owner updates/removes their own namespace in the registry?

Account users can only update/forgoe their namespace if the security feature is disabled in `registry`. In relation to our middleare, if a user is able to update their namespace (via first forgoeing, then claiming a new namespace), this would prevent account tokens from being minted due to the middleware erroring on claiming namespace for an local account id set.

#### Q: what happens when a namespace is forgoed?

when a namespace is forgoed in the `registry`, all registered modules from the account are yanked, and the internal reverse maps for the namespace to the account are also removed. **Only the registry admin or an account owner can forgoe their namespace**

#### Q: what happens when account token is removed as ownership token from abstract account, and there still is the namespace reserved in the `registry` contract?

A: the user still has ownership of the token, and the `ExecuteMsg::UpdateOwnership` entrypoint does not perform modifications to the `registry` namespace so the namespace reserved in the `registry` namespace will still consist of the `token` id. At any moment the token is transferred, the bs721-account invokes the update_ask for our marketplace, which in turn invokes the UpdateAsk hook, and performs validation checks that will determine whether or not to update the `registry` state to accurately reflect the state of the abstract-account in the `registry` contract. This may include forboding a namespace
