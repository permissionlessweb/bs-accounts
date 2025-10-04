#![cfg(not(test))]
use btsg_account::market::{ExecuteMsg, MarketplaceInstantiateMsg, QueryMsg};
use cosmwasm_schema::write_api;

fn main() {
    write_api! {
        instantiate: MarketplaceInstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
