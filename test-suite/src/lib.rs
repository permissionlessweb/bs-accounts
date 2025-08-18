#[cfg(test)]
mod multi_test;
pub mod suite;
pub use suite::*;

pub mod constants {
    use easy_addr::addr;

    pub const VALIDATOR_1: &str = addr!("val1");
    pub const CREATOR: &str = addr!("creator");
    pub const ADMIN: &str = addr!("admin");
    pub const VERIFIER: &str = addr!("verifier");
    pub const RECIPIENT: &str = addr!("recipient");
    pub const NOT_MINTER: &str = addr!("not-minter");
    pub const UNAUTHORIZED: &str = addr!("unauthorized");

    pub const BID_AMOUNT: u128 = 1_000_000_000;
    pub const BIDDER: &str = addr!("bidder");
    pub const BIDDER1: &str = addr!("bidder1");
    pub const BIDDER2: &str = addr!("bidder2");
    pub const USER3: &str = addr!("user3");
    pub const USER4: &str = addr!("user4");
    pub const USER5: &str = addr!("user5");
    pub const NEW_ADMIN: &str = addr!("new_admin");
    pub const NEW_OWNER: &str = addr!("new_owner");
    pub const DELEGATE: &str = addr!("delegate_user");
}
