# btsg-wavs

An authentication contract validated an BLS12_381 signature, specifically for validating actions to take from transactions broadcasted by AVS operators.

## Spec 
- `wavs_ops_pubkeys`: the list of G1 point public-keys used from the WAVS operator set.
- `wavs_ops_signatures`: the list of G2 point signatures from the WAVS operator set.
- `aggregated_pubkey`: the resulting G1 public key, determined by the aggregation of all `wavs_ops_pubkeys` provided.
- `aggregated_signature`: the resulting G2 signature,determined by the aggregation of all `wavs_ops_signatures` provided.
- `wavs_action_msgs`: array of actions being authorized to perform 
- `wavs_action_msgs_hash`: sha256 hash of the list `wavs_action_msgs`. This value is what is signed by wavs operator set
- `dst`: ?


## TODO
- integration tests
- bls12 - Threshold Verification: Allow a threshold of signers to not be included in the aggregated set to authorize actions

 
 