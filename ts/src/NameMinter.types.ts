/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.24.0.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export type Uint128 = string;
export interface InstantiateMsg {
  admin?: string | null;
  base_price: Uint128;
  collection_code_id: number;
  fair_burn_bps: number;
  marketplace_addr: string;
  max_name_length: number;
  min_name_length: number;
  verifier?: string | null;
  whitelists: string[];
}
export type ExecuteMsg = {
  mint_and_list: {
    name: string;
  };
} | {
  update_admin: {
    admin?: string | null;
  };
} | {
  pause: {
    pause: boolean;
  };
} | {
  add_whitelist: {
    address: string;
  };
} | {
  remove_whitelist: {
    address: string;
  };
} | {
  update_config: {
    config: Config;
  };
};
export type Timestamp = Uint64;
export type Uint64 = string;
export interface Config {
  public_mint_start_time: Timestamp;
}
export type QueryMsg = {
  admin: {};
} | {
  whitelists: {};
} | {
  collection: {};
} | {
  params: {};
} | {
  config: {};
};
export interface AdminResponse {
  admin?: string | null;
}
export interface CollectionResponse {
  collection: string;
}
export interface ConfigResponse {
  config: Config;
}
export type Decimal = string;
export interface ParamsResponse {
  params: SudoParams;
}
export interface SudoParams {
  base_price: Uint128;
  fair_burn_percent: Decimal;
  max_name_length: number;
  min_name_length: number;
}
export type Addr = string;
export type ArrayOfAddr = Addr[];