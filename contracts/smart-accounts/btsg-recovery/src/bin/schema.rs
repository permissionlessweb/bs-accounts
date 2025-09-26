use btsg_account_backup::BtsgAccountBackup;
use cosmwasm_schema::write_api;

use btsg_account_backup::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
        sudo: btsg_auth::AuthenticatorSudoMsg,
    }
}
