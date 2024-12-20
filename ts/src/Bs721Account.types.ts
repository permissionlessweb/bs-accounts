/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.24.0.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export interface InstantiateMsg {
  base_init_msg: InstantiateMsg1;
  verifier?: string | null;
}
export interface InstantiateMsg1 {
  minter: string;
  name: string;
  symbol: string;
}
export type ExecuteMsg = {
  set_marketplace: {
    address: string;
  };
} | {
  associate_address: {
    account: string;
    address?: string | null;
  };
} | {
  update_image_nft: {
    account: string;
    nft?: NFT | null;
  };
} | {
  add_text_record: {
    account: string;
    record: TextRecord;
  };
} | {
  remove_text_record: {
    account: string;
    record_account: string;
  };
} | {
  update_text_record: {
    account: string;
    record: TextRecord;
  };
} | {
  verify_text_record: {
    account: string;
    record_account: string;
    result: boolean;
  };
} | {
  update_verifier: {
    verifier?: string | null;
  };
} | {
  transfer_nft: {
    recipient: string;
    token_id: string;
  };
} | {
  send_nft: {
    contract: string;
    msg: Binary;
    token_id: string;
  };
} | {
  approve: {
    expires?: Expiration | null;
    spender: string;
    token_id: string;
  };
} | {
  revoke: {
    spender: string;
    token_id: string;
  };
} | {
  approve_all: {
    expires?: Expiration | null;
    operator: string;
  };
} | {
  revoke_all: {
    operator: string;
  };
} | {
  mint: {
    extension: Metadata;
    owner: string;
    token_id: string;
    token_uri?: string | null;
  };
} | {
  burn: {
    token_id: string;
  };
} | {
  freeze_collection_info: {};
};
export type Addr = string;
export type Binary = string;
export type Expiration = {
  at_height: number;
} | {
  at_time: Timestamp;
} | {
  never: {};
};
export type Timestamp = Uint64;
export type Uint64 = string;
export interface NFT {
  collection: Addr;
  token_id: string;
}
export interface TextRecord {
  account: string;
  value: string;
  verified?: boolean | null;
}
export interface Metadata {
  image_nft?: NFT | null;
  records: TextRecord[];
}
export type QueryMsg = {
  params: {};
} | {
  account: {
    address: string;
  };
} | {
  account_marketplace: {};
} | {
  associated_address: {
    account: string;
  };
} | {
  image_n_f_t: {
    account: string;
  };
} | {
  text_records: {
    account: string;
  };
} | {
  is_twitter_verified: {
    account: string;
  };
} | {
  verifier: {};
} | {
  owner_of: {
    include_expired?: boolean | null;
    token_id: string;
  };
} | {
  approval: {
    include_expired?: boolean | null;
    spender: string;
    token_id: string;
  };
} | {
  approvals: {
    include_expired?: boolean | null;
    token_id: string;
  };
} | {
  all_operators: {
    include_expired?: boolean | null;
    limit?: number | null;
    owner: string;
    start_after?: string | null;
  };
} | {
  num_tokens: {};
} | {
  contract_info: {};
} | {
  nft_info: {
    token_id: string;
  };
} | {
  all_nft_info: {
    include_expired?: boolean | null;
    token_id: string;
  };
} | {
  tokens: {
    limit?: number | null;
    owner: string;
    start_after?: string | null;
  };
} | {
  all_tokens: {
    limit?: number | null;
    start_after?: string | null;
  };
} | {
  minter: {};
};
export type String = string;
export interface AllNftInfoResponseForMetadata {
  access: OwnerOfResponse;
  info: NftInfoResponseForMetadata;
}
export interface OwnerOfResponse {
  approvals: Approval[];
  owner: string;
}
export interface Approval {
  expires: Expiration;
  spender: string;
}
export interface NftInfoResponseForMetadata {
  extension: Metadata;
  token_uri?: string | null;
}
export interface OperatorsResponse {
  operators: Approval[];
}
export interface TokensResponse {
  tokens: string[];
}
export interface ApprovalResponse {
  approval: Approval;
}
export interface ApprovalsResponse {
  approvals: Approval[];
}
export interface ContractInfoResponse {
  name: string;
  symbol: string;
}
export type NullableNFT = NFT | null;
export type Boolean = boolean;
export interface OwnershipForAddr {
  owner?: Addr | null;
  pending_expiry?: Expiration | null;
  pending_owner?: Addr | null;
}
export interface NumTokensResponse {
  count: number;
}
export interface SudoParams {
  max_record_count: number;
}
export type ArrayOfTextRecord = TextRecord[];
export type NullableString = string | null;