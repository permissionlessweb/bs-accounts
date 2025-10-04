#![cfg(not(test))]
use cosmwasm_schema::write_api;

use bs721_account::{
    msg::{ExecuteMsg, InstantiateMsg},
    QueryMsg,
};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg<btsg_account::Metadata>,
        query: QueryMsg,
    }
}
