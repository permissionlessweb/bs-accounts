use cosmwasm_schema::write_api;

use btsg_eth::{ExecuteMsg, InstantiateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
        sudo: btsg_auth::AuthenticatorSudoMsg,
    }
}
