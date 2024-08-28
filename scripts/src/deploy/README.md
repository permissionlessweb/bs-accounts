## Deployment of Bitsong Account Token Framework

To deploy, simply run `cargo run -- --bin deploy`. This will upload, instantiate, and configure the airdrop contracts with the key provided. 


### Deploy as x/gov 
`cargo run -- --bin deploy-as-gov` deploys the contracts as the governance module, via authz.
