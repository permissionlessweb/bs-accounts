use bs721_account::msg::{Bs721AccountsQueryMsg as QueryMsg, ExecuteMsg, InstantiateMsg};
use btsg_account::Metadata;
use cosmwasm_schema::write_api;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg<Metadata>,
        query: QueryMsg,
    }
}
