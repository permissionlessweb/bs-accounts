# Bitsong Accounts API Docs

Bitsong Accounts associates human-readable useraccounts with Cosmos addresses. Address lookups can be done via an API or Typescript library.

## API

### Variables

| Network | `endpoint`                         | `contract`                                                         |
| ------- | ---------------------------------- | ------------------------------------------------------------------ |
| Testnet | `` | `` |
| Mainnet | ``           | `` |

### Query Associated Address

Given a account, get its associated address. Queries are base64 encoded.

Let's say you want to query the account `alice`.

```json
{
  "associated_address": {
    "account": "alice"
  }
}
```

`query`:

`ewogICJhc3NvY2lhdGVkX2FkZHJlc3MiOiB7CiAgICAibmFtZSI6ICJhbGljZSIKICB9Cn0=`

API call:

```
{endpoint}/cosmwasm/wasm/v1/contract/{contract}/smart/{query}
```

### Query Account

Given an address, query it's associated account. An address can be _any_ Cosmos address for a chain that uses the 118 coin type. In the future, Bitsong Accounts will support other coin types.

```json
{
  "account": { "address": "bitsong1" }
}
```

`query`:

`ewogICJuYW1lIjogeyAiYWRkcmVzcyI6ICJzdGFyczF0cXp6bXhzdnp1NDk1Mm1uZDV1bDgwMHdmdXNyNnA3Mm1hZ3lnZSIgfQp9Cg==`

API call:

```
{endpoint}/cosmwasm/wasm/v1/contract/{contract}/smart/{query}
```

### Query all info about an account 

If you just need to fetch address associated with account, it's recommended to use query from "Query Associated Address" section of this document, however if you need more advanced info like text records, associated NFT, minter address etc, then you might consider querying full account info.


```json
{
  "all_nft_info": { "token_id": "alice" }
}
```

`query`:

`ewogICJhbGxfbmZ0X2luZm8iOiB7ICJ0b2tlbl9pZCI6ICJhbGljZSIgfQp9`

API call:

```
{endpoint}/cosmwasm/wasm/v1/contract/{contract}/smart/{query}
```

## Typescript

### Variables

| Network | `endpoint`                        | `contract`                                                         |
| ------- | --------------------------------- | ------------------------------------------------------------------ |
| Testnet | `rpc.` | `` |
| Mainnet | `rpc.`           | `` |

### Query Associated Address

```ts
import { CosmWasmClient } from "cosmwasm";

const client = await CosmWasmClient.connect(endpoint);

const address = await client.queryContractSmart(contract, {
  associated_address: { account: "alice" },
});

console.log("address:", address);
```

### Query Account

```ts
import { CosmWasmClient } from "cosmwasm";

const client = await CosmWasmClient.connect(endpoint);

const account = await client.queryContractSmart(contract, {
  account: { address: "bitsong1" },
});

console.log("account:", account);
```

### Query all info about account 


```ts
import { CosmWasmClient } from "cosmwasm";

const client = await CosmWasmClient.connect(endpoint);

const fullAccountInfo = await client.queryContractSmart(contract, {
  all_nft_info: { token_id: "alice" },
});

console.log("account info:", fullAccountInfo);
```
