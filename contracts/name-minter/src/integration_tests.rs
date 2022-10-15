use crate::msg::{ExecuteMsg, InstantiateMsg};
use cosmwasm_std::{coins, Addr, Uint128};
use cw721::{NumTokensResponse, OwnerOfResponse};
use cw_multi_test::{BankSudo, Contract, ContractWrapper, Executor, SudoMsg as CwSudoMsg};
use name_marketplace::msg::{
    AskResponse, BidResponse, ExecuteMsg as MarketplaceExecuteMsg, QueryMsg as MarketplaceQueryMsg,
};
use sg721_name::ExecuteMsg as Sg721NameExecuteMsg;
use sg_multi_test::StargazeApp;
use sg_name::{NameMarketplaceResponse, SgNameQueryMsg};
use sg_std::{StargazeMsgWrapper, NATIVE_DENOM};

pub fn contract_minter() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::query::query,
    )
    .with_reply(crate::contract::reply);
    Box::new(contract)
}

pub fn contract_marketplace() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        name_marketplace::execute::execute,
        name_marketplace::execute::instantiate,
        name_marketplace::query::query,
    )
    .with_sudo(name_marketplace::sudo::sudo);
    Box::new(contract)
}

pub fn contract_collection() -> Box<dyn Contract<StargazeMsgWrapper>> {
    let contract = ContractWrapper::new(
        sg721_name::entry::execute,
        sg721_name::entry::instantiate,
        sg721_name::entry::query,
    );
    Box::new(contract)
}

const USER: &str = "user";
const USER2: &str = "user2";
const BIDDER: &str = "bidder";
const BIDDER2: &str = "bidder2";
const ADMIN: &str = "admin";
const ADMIN2: &str = "admin2";
const NAME: &str = "bobo";

const TRADING_FEE_BPS: u64 = 200; // 2%
const BASE_PRICE: u128 = 100_000_000;
const BID_AMOUNT: u128 = 1_000_000_000;

const SECONDS_PER_YEAR: u64 = 31536000;

const MKT: &str = "contract0";
const MINTER: &str = "contract1";
const COLLECTION: &str = "contract2";

// NOTE: This are mostly Marketplace integration tests. They could possibly be moved into the marketplace contract.

pub fn custom_mock_app() -> StargazeApp {
    StargazeApp::default()
}

// 1. Instantiate Name Marketplace
// 2. Instantiate Name Minter (which instantiates Name Collection)
// 3. Update Name Marketplace with Name Minter address
// 4. Update Name Marketplace with Name Collection address
fn instantiate_contracts(admin: Option<String>) -> StargazeApp {
    let mut app = custom_mock_app();
    let mkt_id = app.store_code(contract_marketplace());
    let minter_id = app.store_code(contract_minter());
    let sg721_id = app.store_code(contract_collection());

    // 1. Instantiate Name Marketplace
    let msg = name_marketplace::msg::InstantiateMsg {
        trading_fee_bps: TRADING_FEE_BPS,
        min_price: Uint128::from(5u128),
    };
    let marketplace = app
        .instantiate_contract(
            mkt_id,
            Addr::unchecked(ADMIN),
            &msg,
            &[],
            "Name-Marketplace",
            admin.clone(),
        )
        .unwrap();

    // 2. Instantiate Name Minter (which instantiates Name Collection)
    let msg = InstantiateMsg {
        admin,
        collection_code_id: sg721_id,
        marketplace_addr: marketplace.to_string(),
        base_price: Uint128::from(BASE_PRICE),
        min_name_length: 3,
        max_name_length: 63,
    };
    let minter = app
        .instantiate_contract(
            minter_id,
            Addr::unchecked(ADMIN2),
            &msg,
            &[],
            "Name-Minter",
            None,
        )
        .unwrap();

    // 3. Update Name Marketplace with Name Minter address
    let msg = name_marketplace::msg::SudoMsg::UpdateNameMinter {
        minter: minter.to_string(),
    };
    let res = app.wasm_sudo(marketplace.clone(), &msg);
    assert!(res.is_ok());

    let res: NameMarketplaceResponse = app
        .wrap()
        .query_wasm_smart(COLLECTION, &SgNameQueryMsg::NameMarketplace {})
        .unwrap();
    assert_eq!(res.address, marketplace.to_string());

    // 4. Update Name Marketplace with Name Collection address
    let msg = name_marketplace::msg::SudoMsg::UpdateNameCollection {
        collection: COLLECTION.to_string(),
    };
    let res = app.wasm_sudo(marketplace, &msg);
    assert!(res.is_ok());

    app
}

fn owner_of(app: &StargazeApp, token_id: String) -> String {
    let res: OwnerOfResponse = app
        .wrap()
        .query_wasm_smart(
            COLLECTION,
            &sg721_base::msg::QueryMsg::OwnerOf {
                token_id,
                include_expired: None,
            },
        )
        .unwrap();

    res.owner
}

fn update_block_height(app: &mut StargazeApp, height: u64) {
    let mut block = app.block_info();
    block.height = height;
    app.set_block(block);
}

fn mint_and_list(app: &mut StargazeApp, name: &str, user: &str, contract: Option<String>) {
    // set approval for user, for all tokens
    // approve_all is needed because we don't know the token_id before-hand
    let approve_all_msg = Sg721NameExecuteMsg::ApproveAll {
        operator: MKT.to_string(),
        expires: None,
    };
    let res = app.execute_contract(
        Addr::unchecked(user),
        Addr::unchecked(COLLECTION),
        &approve_all_msg,
        &[],
    );
    assert!(res.is_ok());

    let four_letter_name_cost = BASE_PRICE * 10;

    // give user some funds
    let name_fee = coins(four_letter_name_cost, NATIVE_DENOM);
    app.sudo(CwSudoMsg::Bank({
        BankSudo::Mint {
            to_address: user.to_string(),
            amount: name_fee.clone(),
        }
    }))
    .map_err(|err| println!("{:?}", err))
    .ok();

    let msg = ExecuteMsg::MintAndList {
        name: name.to_string(),
        contract,
    };
    let res = app.execute_contract(
        Addr::unchecked(user),
        Addr::unchecked(MINTER),
        &msg,
        &name_fee,
    );
    // println!("{:?}", res);
    assert!(res.is_ok());

    // check if name is listed in marketplace
    let res: AskResponse = app
        .wrap()
        .query_wasm_smart(
            MKT,
            &MarketplaceQueryMsg::Ask {
                token_id: name.to_string(),
            },
        )
        .unwrap();
    assert_eq!(res.ask.unwrap().token_id, name);

    // check if token minted
    let _res: NumTokensResponse = app
        .wrap()
        .query_wasm_smart(
            Addr::unchecked(COLLECTION),
            &sg721_base::msg::QueryMsg::NumTokens {},
        )
        .unwrap();

    assert_eq!(owner_of(app, name.to_string()), user.to_string());
}

fn bid(app: &mut StargazeApp, bidder: &str, amount: u128) {
    let bidder = Addr::unchecked(bidder);

    // give bidder some funds
    let amount = coins(amount, NATIVE_DENOM);
    app.sudo(CwSudoMsg::Bank({
        BankSudo::Mint {
            to_address: bidder.to_string(),
            amount: amount.clone(),
        }
    }))
    .map_err(|err| println!("{:?}", err))
    .ok();

    let msg = MarketplaceExecuteMsg::SetBid {
        token_id: NAME.to_string(),
    };
    let res = app.execute_contract(bidder.clone(), Addr::unchecked(MKT), &msg, &amount);
    assert!(res.is_ok());

    // query if bid exists
    let res: BidResponse = app
        .wrap()
        .query_wasm_smart(
            MKT,
            &MarketplaceQueryMsg::Bid {
                token_id: NAME.to_string(),
                bidder: bidder.to_string(),
            },
        )
        .unwrap();
    let bid = res.bid.unwrap();
    assert_eq!(bid.token_id, NAME.to_string());
    assert_eq!(bid.bidder, bidder.to_string());
    assert_eq!(bid.amount, amount[0].amount);
}

mod execute {
    use cw721::OperatorsResponse;

    use super::*;

    #[test]
    fn check_approvals() {
        let mut app = instantiate_contracts(None);

        mint_and_list(&mut app, NAME, USER, None);

        // check operators
        let res: OperatorsResponse = app
            .wrap()
            .query_wasm_smart(
                COLLECTION,
                &sg721_base::msg::QueryMsg::AllOperators {
                    owner: USER.to_string(),
                    include_expired: None,
                    start_after: None,
                    limit: None,
                },
            )
            .unwrap();
        assert_eq!(res.operators.len(), 1);
    }

    #[test]
    fn test_mint() {
        let mut app = instantiate_contracts(None);

        mint_and_list(&mut app, NAME, USER, None);
    }

    #[test]
    fn test_mint_for_contract() {
        // contract creator can mint a name for contract
        let mut app = instantiate_contracts(None);
        mint_and_list(&mut app, NAME, ADMIN2, Some(MINTER.to_string()));

        // contract admin can mint a name for contract
        let mut app = instantiate_contracts(Some(ADMIN.to_string()));
        mint_and_list(&mut app, NAME, ADMIN, Some(MKT.to_string()));

        // wrong creator cannot mint a name for contract
        // let mut app = instantiate_contracts(None);
        // mint_and_list(&mut app, NAME, ADMIN, Some(MINTER.to_string()));

        // wrong admin cannot mint a name for contract
        // let mut app = instantiate_contracts(Some(ADMIN.to_string()));
        // mint_and_list(&mut app, NAME, ADMIN2, Some(MKT.to_string()));
    }

    #[test]
    fn test_bid() {
        let mut app = instantiate_contracts(None);

        mint_and_list(&mut app, NAME, USER, None);
        bid(&mut app, BIDDER, BID_AMOUNT);
    }

    #[test]
    fn test_accept_bid() {
        let mut app = instantiate_contracts(None);

        mint_and_list(&mut app, NAME, USER, None);
        bid(&mut app, BIDDER, BID_AMOUNT);

        // user (owner) starts off with 0 internet funny money
        let res = app
            .wrap()
            .query_balance(USER.to_string(), NATIVE_DENOM)
            .unwrap();
        assert_eq!(res.amount, Uint128::new(0));

        let msg = MarketplaceExecuteMsg::AcceptBid {
            token_id: NAME.to_string(),
            bidder: BIDDER.to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(USER), Addr::unchecked(MKT), &msg, &[]);
        assert!(res.is_ok());

        // check if bid is removed
        let res: BidResponse = app
            .wrap()
            .query_wasm_smart(
                MKT,
                &MarketplaceQueryMsg::Bid {
                    token_id: NAME.to_string(),
                    bidder: BIDDER.to_string(),
                },
            )
            .unwrap();
        assert!(res.bid.is_none());

        // verify that the bidder is the new owner
        assert_eq!(owner_of(&app, NAME.to_string()), BIDDER.to_string());

        // check if user got the bid amount
        let res = app
            .wrap()
            .query_balance(USER.to_string(), NATIVE_DENOM)
            .unwrap();
        let protocol_fee = 20_000_000u128;
        assert_eq!(res.amount, Uint128::from(BID_AMOUNT - protocol_fee));

        // confirm that a new ask was created
        let res: AskResponse = app
            .wrap()
            .query_wasm_smart(
                MKT,
                &MarketplaceQueryMsg::Ask {
                    token_id: NAME.to_string(),
                },
            )
            .unwrap();
        let ask = res.ask.unwrap();
        assert_eq!(ask.token_id, NAME);
        assert_eq!(ask.seller, BIDDER.to_string());
    }

    //  test two sales cycles in a row to check if approvals work
    #[test]
    fn test_two_sales_cycles() {
        let mut app = instantiate_contracts(None);

        mint_and_list(&mut app, NAME, USER, None);
        bid(&mut app, BIDDER, BID_AMOUNT);

        let msg = MarketplaceExecuteMsg::AcceptBid {
            token_id: NAME.to_string(),
            bidder: BIDDER.to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(USER), Addr::unchecked(MKT), &msg, &[]);
        assert!(res.is_ok());

        bid(&mut app, BIDDER2, BID_AMOUNT);

        // have to approve marketplace spend for bid acceptor (bidder)
        let msg = Sg721NameExecuteMsg::Approve {
            spender: MKT.to_string(),
            token_id: NAME.to_string(),
            expires: None,
        };
        let res = app.execute_contract(
            Addr::unchecked(BIDDER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        let msg = MarketplaceExecuteMsg::AcceptBid {
            token_id: NAME.to_string(),
            bidder: BIDDER2.to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(BIDDER), Addr::unchecked(MKT), &msg, &[]);
        assert!(res.is_ok());
    }
}

mod admin {
    use crate::msg::{QueryMsg, WhitelistResponse};

    use super::*;

    #[test]
    fn update_admin() {
        let mut app = instantiate_contracts(Some(ADMIN.to_string()));

        let msg = ExecuteMsg::UpdateAdmin { admin: None };
        let res = app.execute_contract(Addr::unchecked(USER), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_err());

        let msg = ExecuteMsg::UpdateAdmin {
            admin: Some(USER2.to_string()),
        };
        let res = app.execute_contract(Addr::unchecked(ADMIN), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_ok());

        let msg = ExecuteMsg::UpdateAdmin { admin: None };
        let res = app.execute_contract(Addr::unchecked(USER2), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_ok());

        // cannot update admin after its been removed
        let msg = ExecuteMsg::UpdateAdmin { admin: None };
        let res = app.execute_contract(Addr::unchecked(USER2), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_err());
    }

    #[test]
    fn update_whitelist() {
        let mut app = instantiate_contracts(Some(ADMIN.to_string()));

        let msg = ExecuteMsg::UpdateWhitelist { whitelist: None };

        let res = app.execute_contract(Addr::unchecked(USER), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_err());

        let res = app.execute_contract(Addr::unchecked(ADMIN), Addr::unchecked(MINTER), &msg, &[]);
        assert!(res.is_ok());

        let msg = QueryMsg::Whitelist {};
        let res: WhitelistResponse = app.wrap().query_wasm_smart(MINTER, &msg).unwrap();
        assert_eq!(res.whitelist, None);
    }
}

mod query {
    use cosmwasm_std::StdResult;
    use name_marketplace::msg::{AskCountResponse, AsksResponse, BidsResponse};
    use sg_name::NameResponse;

    use super::*;

    #[test]
    fn query_ask() {
        let mut app = instantiate_contracts(None);

        mint_and_list(&mut app, NAME, USER, None);

        let msg = MarketplaceQueryMsg::Ask {
            token_id: NAME.to_string(),
        };
        let res: AskResponse = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.ask.unwrap().token_id, NAME.to_string());
    }

    #[test]
    fn query_asks() {
        let mut app = instantiate_contracts(None);

        mint_and_list(&mut app, NAME, USER, None);

        let height = app.block_info().height;
        update_block_height(&mut app, height + 1);
        mint_and_list(&mut app, "hack", USER, None);

        let msg = MarketplaceQueryMsg::Asks {
            start_after: None,
            limit: None,
        };
        let res: AsksResponse = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.asks[0].id, 1);
    }

    #[test]
    fn query_reverse_asks() {
        let mut app = instantiate_contracts(None);

        mint_and_list(&mut app, NAME, USER, None);

        let height = app.block_info().height;
        update_block_height(&mut app, height + 1);
        mint_and_list(&mut app, "hack", USER, None);

        let msg = MarketplaceQueryMsg::ReverseAsks {
            start_before: None,
            limit: None,
        };
        let res: AsksResponse = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.asks[0].id, 2);
    }

    #[test]
    fn query_asks_by_seller() {
        let mut app = instantiate_contracts(None);

        mint_and_list(&mut app, NAME, USER, None);

        let height = app.block_info().height;
        update_block_height(&mut app, height + 1);
        mint_and_list(&mut app, "hack", "user2", None);

        let msg = MarketplaceQueryMsg::AsksBySeller {
            seller: USER.to_string(),
            start_after: None,
            limit: None,
        };
        let res: AsksResponse = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.asks.len(), 1);
    }

    #[test]
    fn query_ask_count() {
        let mut app = instantiate_contracts(None);

        mint_and_list(&mut app, NAME, USER, None);

        let height = app.block_info().height;
        update_block_height(&mut app, height + 1);
        mint_and_list(&mut app, "hack", USER, None);

        let msg = MarketplaceQueryMsg::AskCount {};
        let res: AskCountResponse = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.count, 2);
    }

    #[test]
    fn query_top_bids() {
        let mut app = instantiate_contracts(None);

        mint_and_list(&mut app, NAME, USER, None);
        bid(&mut app, BIDDER, BID_AMOUNT);
        bid(&mut app, BIDDER2, BID_AMOUNT * 5);

        let msg = MarketplaceQueryMsg::ReverseBidsSortedByPrice {
            start_before: None,
            limit: None,
        };
        let res: BidsResponse = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.bids.len(), 2);
        assert_eq!(res.bids[0].amount.u128(), BID_AMOUNT * 5);
    }

    #[test]
    fn query_renewal_queue() {
        let mut app = instantiate_contracts(None);

        // mint two names at the same time
        mint_and_list(&mut app, NAME, USER, None);
        mint_and_list(&mut app, "hack", USER, None);

        let res: AsksResponse = app
            .wrap()
            .query_wasm_smart(
                MKT,
                &MarketplaceQueryMsg::RenewalQueue {
                    time: app.block_info().time.plus_seconds(SECONDS_PER_YEAR),
                },
            )
            .unwrap();
        assert_eq!(res.asks.len(), 2);
        assert_eq!(res.asks[1].token_id, "hack".to_string());
    }

    #[test]
    fn query_name() {
        let mut app = instantiate_contracts(None);

        mint_and_list(&mut app, NAME, USER, None);

        // fails with "user" string, has to be a bech32 address
        let res: StdResult<NameResponse> = app.wrap().query_wasm_smart(
            COLLECTION,
            &SgNameQueryMsg::Name {
                address: USER.to_string(),
            },
        );
        assert!(res.is_err());

        let stars_address = "stars1hsk6jryyqjfhp5dhc55tc9jtckygx0eprx6sym";
        let cosmos_address = "cosmos1hsk6jryyqjfhp5dhc55tc9jtckygx0eph6dd02";

        mint_and_list(&mut app, "yoyo", stars_address, None);

        let res: NameResponse = app
            .wrap()
            .query_wasm_smart(
                COLLECTION,
                &SgNameQueryMsg::Name {
                    address: cosmos_address.to_string(),
                },
            )
            .unwrap();
        assert_eq!(res.name, "yoyo".to_string());
    }
}

mod collection {
    use super::*;

    fn transfer(app: &mut StargazeApp) {
        let msg = Sg721NameExecuteMsg::TransferNft {
            recipient: USER2.to_string(),
            token_id: NAME.to_string(),
        };
        let res = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        let msg = MarketplaceQueryMsg::Ask {
            token_id: NAME.to_string(),
        };
        let res: AskResponse = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert_eq!(res.ask.unwrap().seller.to_string(), USER2.to_string());
    }

    #[test]
    fn transfer_nft() {
        let mut app = instantiate_contracts(None);

        mint_and_list(&mut app, NAME, USER, None);
        transfer(&mut app);
    }

    #[test]
    fn transfer_nft_and_bid() {
        let mut app = instantiate_contracts(None);

        mint_and_list(&mut app, NAME, USER, None);

        // transfer to user2
        transfer(&mut app);

        bid(&mut app, BIDDER, BID_AMOUNT);

        // user2 must approve the marketplace to transfer their name
        let msg = Sg721NameExecuteMsg::Approve {
            spender: MKT.to_string(),
            token_id: NAME.to_string(),
            expires: None,
        };
        let res = app.execute_contract(
            Addr::unchecked(USER2),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        // accept bid
        let msg = MarketplaceExecuteMsg::AcceptBid {
            token_id: NAME.to_string(),
            bidder: BIDDER.to_string(),
        };
        let res = app.execute_contract(Addr::unchecked(USER2), Addr::unchecked(MKT), &msg, &[]);
        assert!(res.is_ok());
    }

    #[test]
    fn burn_nft() {
        let mut app = instantiate_contracts(None);

        mint_and_list(&mut app, NAME, USER, None);

        let msg = Sg721NameExecuteMsg::Burn {
            token_id: NAME.to_string(),
        };
        let res = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_ok());

        let msg = MarketplaceQueryMsg::Ask {
            token_id: NAME.to_string(),
        };
        let res: AskResponse = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert!(res.ask.is_none());
    }

    #[test]
    fn burn_with_existing_bids() {
        let mut app = instantiate_contracts(None);

        mint_and_list(&mut app, NAME, USER, None);

        bid(&mut app, BIDDER, BID_AMOUNT);

        let msg = Sg721NameExecuteMsg::Burn {
            token_id: NAME.to_string(),
        };
        let res = app.execute_contract(
            Addr::unchecked(USER),
            Addr::unchecked(COLLECTION),
            &msg,
            &[],
        );
        assert!(res.is_err());

        let msg = MarketplaceQueryMsg::Ask {
            token_id: NAME.to_string(),
        };
        let res: AskResponse = app.wrap().query_wasm_smart(MKT, &msg).unwrap();
        assert!(res.ask.is_some());
    }
}
