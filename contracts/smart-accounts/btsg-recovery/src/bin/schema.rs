use cosmwasm_schema::write_api;
use btsg_account_backup::BtsgAccountBackup;
use btsg_account_backup::{ExecuteMsg, InstantiateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
        sudo: btsg_auth::AuthSudoMsg,
    }
}
