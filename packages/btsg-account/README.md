# BTSG Authenticator Trait for CosmWasm Contracts

## Overview

This library provides the `BtsgAccountTrait` for implementing CosmWasm-based authenticators in the [x/smart-account module](https://github.com/terpnetwork/terp-core/blob/main/x/smart-account/README.md). The trait enables your contract to handle authentication logic as a "CosmWasm Authenticator," allowing custom, on-chain verification of transactions while integrating seamlessly with the Cosmos SDK's authentication flow.

### Key Concepts from x/smart-account
- **Authenticators**: Custom logic for verifying transactions. CosmWasm authenticators are contracts that respond to sudo messages from the module.
- **Flow Integration**:
  - **Circuit Breaker**: The module acts as an ante handler. If disabled (`is_smart_account_active = false`), it falls back to standard Cosmos SDK auth.
  - **Authenticator Selection**: Users specify authenticators per message via `TxExtension.selected_authenticators`. If unspecified, defaults to classic auth.
  - **High-Level Steps** (see [Authenticator Flow](https://github.com/terpnetwork/terp-core/blob/main/x/smart-account/README.md#authenticator-flow)):
    1. **Authenticate** (pre-execution, stateless): Validate message without state changes.
    2. **Track** (post-auth, pre-execution): Commit state changes (not reverted on execution failure).
    3. **Execute Messages**: Run the tx (auth changes persist even on failure).
    4. **ConfirmExecution** (post-execution): Enforce rules (e.g., limits); fail to revert execution changes.
  - **Account Configuration**: Users add authenticators via `MsgAddAuthenticator` (type: `CosmwasmAuthenticatorV1`), providing contract address and `params` (JSON bytes for user-specific config).
  - **Hooks**: `OnAuthenticatorAdded`/`OnAuthenticatorRemoved` for setup/teardown.
- **Restrictions**:
  - One signer per message.
  - Fee payer is the first signer (or via feegrant).
  - Gas limits during unauthenticated auth (`maximum_unauthenticated_gas` param).
- **Composite Authenticators**: Can nest (e.g., `AnyOf`, `AllOf`) with ID propagation (e.g., `86.1` for sub-authenticators).
- **Signatures**: Support simple (shared) or partitioned (JSON array for multisig).

Implementing `BtsgAccountTrait` makes your contract callable via the module's sudo entrypoint, routing to these hooks.

### When to Use
- Build custom auth like spend limits, multisig, inheritance, or message filters.
- Reuse contracts across users via `params` (e.g., pubkeys, thresholds).
- Store dynamic state in contract storage (e.g., nonce tracking).

## Dependencies
Add to `Cargo.toml`:
```toml
[dependencies]
btsg-auth = "0.1"  # Or latest version
cosmwasm-std = "1.5"  # Compatible with your chain
cw-serde = "0.16"
serde = { version = "1.0", features = ["derive"] }
```

## Implementing the Trait

Define your contract's messages and implement `BtsgAccountTrait`. The trait's associated types must match your contract's (e.g., `InstantiateMsg`, `ExecuteMsg`).

### Associated Types
| Type                  | Description |
|-----------------------|-------------|
| `InstantiateMsg`     | Contract instantiation params (e.g., admin, config). |
| `ExecuteMsg`         | User-executable messages (if any; optional for pure authenticators). |
| `QueryMsg`           | Query variants (if needed). |
| `SudoMsg`            | Internal sudo messages (e.g., `AuthSudoMsg`). Route via `process_sudo_auth`. |
| `ContractError`      | Custom error type (implement `std::error::Error`). |
| `AuthMethodStructs`  | Custom structs for auth extensions (e.g., pubkey wrappers). |
| `AuthProcessResult`  | Return type for hooks (e.g., `Result<(), ContractError>`). |

### Core Methods

#### `extended_authenticate(deps: DepsMut, auth: Self::AuthMethodStructs) -> Self::AuthProcessResult`
- **Purpose**: Internal wrapper for custom auth logic. Use for one-off validation outside the standard flow (e.g., init helpers). Not called by the module—separate from `on_auth_request`.
- **Notes**: Can mutate state if needed, but prefer stateless for module integration.
- **Relation to Module**: N/A (internal use).

#### `process_sudo_auth(deps: DepsMut, env: Env, req: &Self::SudoMsg) -> Self::AuthProcessResult`
- **Purpose**: Entry point for all sudo calls from x/smart-account. Route `req` to appropriate hooks (e.g., match on `AuthSudoMsg`).
- **Usage**: In your contract's `sudo` function:
  ```rust
  #[cfg_attr(not(feature = "library"), entry_point)]
  pub fn sudo(deps: DepsMut, env: Env, msg: AuthSudoMsg) -> Result<Response, ContractError> {
      BtsgAccountTrait::process_sudo_auth(deps, env, &msg)
  }
  ```
- **Relation to Module**: Dispatches to `Authenticate`, `Track`, etc., via sudo.

#### `on_auth_added(deps: DepsMut, env: Env, req: &OnAuthenticatorAddedRequest) -> Self::AuthProcessResult`
- **Purpose**: Validate/setup on `MsgAddAuthenticator`. Check `req.config` (user `params`), store account-specific state (e.g., pubkey).
- **Inputs** (from `OnAuthenticatorAddedRequest`):
  - `account`: Bech32 address.
  - `config`: JSON bytes (user params).
  - `authenticator_id`: Global ID (e.g., "86").
- **Notes**: Reject invalid config (return `Err`). Called before auth is active.
- **Relation to Module**: Invoked on add; ensures integrity (see [OnAuthenticatorAdded](https://github.com/terpnetwork/terp-core/blob/main/x/smart-account/README.md#on-authenticatoradded)).

#### `on_auth_removed(deps: DepsMut, env: Env, req: &OnAuthenticatorRemovedRequest) -> Self::AuthProcessResult`
- **Purpose**: Cleanup on `MsgRemoveAuthenticator`. Remove account state (e.g., counters) for optimization.
- **Inputs** (from `OnAuthenticatorRemovedRequest`):
  - `account`: Bech32 address.
  - `config`: Original params.
  - `authenticator_id`: Global ID.
- **Notes**: Prevent removal if in-use (e.g., pending txs). Always commit changes.
- **Relation to Module**: Ensures stability; global data updates (see [OnAuthenticatorRemoved](https://github.com/terpnetwork/terp-core/blob/main/x/smart-account/README.md#on-authenticatorremoved)).

#### `on_auth_request(deps: DepsMut, env: Env, req: &Box<AuthenticationRequest>) -> Self::AuthProcessResult`
- **Purpose**: Stateless validation of message. Return `Ok` to approve, `Err` to reject.
- **Inputs** (from `AuthenticationRequest`):
  - `message`: Protobuf-encoded msg.
  - `signers`: Signer addresses.
  - `signatures`: Sig bytes (simple or partitioned JSON array).
  - `authenticator_id`: Propagated ID (e.g., "86.1").
  - `account`: Target account.
- **Notes**: **No state changes**—discarded on failure. Enforce gas via `StaticGas` equiv. (module handles).
- **Relation to Module**: Ante handler step 3; fails tx if `Err` (see [Authenticate](https://github.com/terpnetwork/terp-core/blob/main/x/smart-account/README.md#authenticate)).

#### `on_auth_track(deps: DepsMut, env: Env, req: &TrackRequest) -> Self::AuthProcessResult`
- **Purpose**: Commit post-auth state (e.g., increment nonce). Called after all msgs auth, before execution.
- **Inputs** (from `TrackRequest`): Same as `AuthenticationRequest`, plus tx context.
- **Notes**: Changes **committed always** if auth succeeds (not reverted on exec failure). For composites: Called on all subs (AnyOf) or used subs (AllOf).
- **Relation to Module**: Step 5; notifies for future rules (see [Track](https://github.com/terpnetwork/terp-core/blob/main/x/smart-account/README.md#track)).

#### `on_auth_confirm(deps: DepsMut, env: Env, req: &ConfirmExecutionRequest) -> Self::AuthProcessResult`
- **Purpose**: Post-exec rules (e.g., check spend limits). `Ok` commits exec changes; `Err` reverts them.
- **Inputs** (from `ConfirmExecutionRequest`): Includes exec results/events.
- **Notes**: No auth guarantee from `on_auth_request` (e.g., AnyOf may call confirm on unused subs). For composites: OR/AND logic.
- **Relation to Module**: Post-handler step 7; enforces outcomes (see [ConfirmExecution](https://github.com/terpnetwork/terp-core/blob/main/x/smart-account/README.md#confirmexecution)).

#### `on_hooks(deps: DepsMut, env: Env) -> Self::AuthProcessResult`
- **Purpose**: Generic hook for chain-specific events (e.g., epoch triggers). Optional; return `Ok(())` if unused.
- **Notes**: Mutate state as needed. Call from custom sudo if extended.
- **Relation to Module**: N/A—extend for advanced use (e.g., inheritance timers).

## Example Implementation: Simple Spend Limit Authenticator

Contract for limiting daily spends, using user `params` (threshold).

```rust
use btsg_auth::*;
use cosmwasm_std::{DepsMut, Env, Response, StdError};
use cw_serde::Serialize;
use serde::{Deserialize, Serialize as SerdeSerialize};

#[derive(Serialize, Deserialize, SerdeSerialize, Debug, Clone, PartialEq)]
pub struct InstantiateMsg {
    pub admin: String,
}

#[derive(Serialize, Deserialize, SerdeSerialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {}  // Pure authenticator—no user exec.

#[derive(Serialize, Deserialize, SerdeSerialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetSpend { account: String },
}

#[derive(Serialize, Deserialize, SerdeSerialize, Debug, Clone, PartialEq)]
pub enum SudoMsg {
    AuthSudoMsg(AuthSudoMsg),
}

#[derive(Debug)]
pub enum ContractError {
    Std(StdError),
    Unauthorized {},
}

impl From<StdError> for ContractError {
    fn from(err: StdError) -> Self { Self::Std(err) }
}

impl std::fmt::Display for ContractError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Std(err) => write!(f, "Std error: {err}"),
            Self::Unauthorized {} => write!(f, "Unauthorized"),
        }
    }
}

impl std::error::Error for ContractError {}

type AuthProcessResult = Result<Response, ContractError>;

#[derive(Serialize, Deserialize, SerdeSerialize, Debug, Clone, PartialEq)]
pub struct SpendLimitParams {
    pub daily_threshold: u128,
    pub denom: String,
}

pub struct SpendLimitAuthenticator;

impl BtsgAccountTrait for SpendLimitAuthenticator {
    type InstantiateMsg = InstantiateMsg;
    type ExecuteMsg = ExecuteMsg;
    type QueryMsg = QueryMsg;
    type SudoMsg = SudoMsg;
    type ContractError = ContractError;
    type AuthMethodStructs = SpendLimitParams;  // User config.
    type AuthProcessResult = AuthProcessResult;

    fn extended_authenticate(_deps: DepsMut, _auth: Self::AuthMethodStructs) -> Self::AuthProcessResult {
        // Internal: e.g., validate params.
        Ok(Response::default())
    }

    fn process_sudo_auth(deps: DepsMut, env: Env, req: &Self::SudoMsg) -> Self::AuthProcessResult {
        match req {
            SudoMsg::AuthSudoMsg(sudo) => match sudo {
                AuthSudoMsg::OnAuthenticatorAdded(req) => Self::on_auth_added(deps, env, req),
                AuthSudoMsg::OnAuthenticatorRemoved(req) => Self::on_auth_removed(deps, env, req),
                AuthSudoMsg::Authenticate(req) => Self::on_auth_request(deps, env, req),
                AuthSudoMsg::Track(req) => Self::on_auth_track(deps, env, req),
                AuthSudoMsg::ConfirmExecution(req) => Self::on_auth_confirm(deps, env, req),
            },
        }
    }

    fn on_auth_added(deps: DepsMut, _env: Env, req: &OnAuthenticatorAddedRequest) -> Self::AuthProcessResult {
        let params: SpendLimitParams = serde_json::from_slice(&req.config).map_err(|_| ContractError::Std(StdError::generic_err("Invalid params")))?;
        // Store per-account: e.g., set threshold, reset daily spend to 0.
        // Use deps.storage to save under key like format!("{}-{}", req.account, req.authenticator_id).
        Ok(Response::new().add_attribute("action", "auth_added"))
    }

    fn on_auth_removed(deps: DepsMut, _env: Env, req: &OnAuthenticatorRemovedRequest) -> Self::AuthProcessResult {
        // Cleanup: remove account key from storage.
        Ok(Response::new().add_attribute("action", "auth_removed"))
    }

    fn on_auth_request(_deps: DepsMut, _env: Env, req: &Box<AuthenticationRequest>) -> Self::AuthProcessResult {
        // Stateless: Simulate/parse msg for spend, check against cached threshold (no mutate!).
        // E.g., for bank send: extract amount/denom, ensure < threshold (pre-fetch via query if needed).
        // Reject if over: return Err(ContractError::Unauthorized).
        Ok(Response::default())
    }

    fn on_auth_track(deps: DepsMut, _env: Env, req: &TrackRequest) -> Self::AuthProcessResult {
        // Commit: Add spend to daily total in storage.
        // Use req.message to parse amount.
        Ok(Response::new().add_attribute("action", "tracked"))
    }

    fn on_auth_confirm(deps: DepsMut, _env: Env, req: &ConfirmExecutionRequest) -> Self::AuthProcessResult {
        // Post-exec: Verify actual spend <= threshold (using events/results).
        // Err reverts exec changes.
        Ok(Response::new().add_attribute("action", "confirmed"))
    }

    fn on_hooks(_deps: DepsMut, _env: Env) -> Self::AuthProcessResult {
        // Optional: e.g., reset daily limits on epoch.
        Ok(Response::default())
    }
}
```

### Contract Entry Points
```rust
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(...) -> Result<Response, ContractError> { ... }

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(...) -> Result<Response, ContractError> { ... }  // If needed.

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(...) -> Result<Binary, ContractError> { ... }

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    <SpendLimitAuthenticator as BtsgAccountTrait>::process_sudo_auth(deps, env, &msg)
}
```

## Adding to Account
Via CLI/JS:
```bash
# MsgAddAuthenticator
terpd tx smart-account add-authenticator <account> CosmWasmAuthenticatorV1 '{"contract": "<addr>", "params": [{"daily_threshold": "1000000", "denom": "uatom"}]}' --from <user>
```

## Testing
- Use `cw-multi-test` for local sim.
- Mock `AuthenticationRequest` with sample msgs/signers.
- Verify state commits/reverts per flow.

## Advanced: Composites & Examples
- **Multisig**: Parse partitioned sigs in `on_auth_request`, delegate to sub-calls (recurse with updated ID).
- **Inheritance**: Use `on_hooks` for inactivity checks; `AnyOf` with standard sig.
- See module docs for [One-Click Trading](https://github.com/terpnetwork/terp-core/blob/main/x/smart-account/README.md#one-click-trading), [Cosigner](https://github.com/terpnetwork/terp-core/blob/main/x/smart-account/README.md#cosigner).

## Limitations & Future
- No sub-authenticator selection (calls all for AnyOf).
- Confirm may call unused auths.
- Track for [improved tracking](https://github.com/terpnetwork/terp-core/issues/8373).

For full types, see [btsg-auth crate](https://docs.rs/btsg-auth). Contribute via xAI!