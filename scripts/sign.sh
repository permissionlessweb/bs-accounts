KEY=$(bitsongd keys show $USER | jq -r .name)

bitsongd tx sign unsignedTx.json \
    --multisig $ADMIN \
    --from $USER \
    --output-document $KEY.json \
    --chain-id $CHAIN_ID
