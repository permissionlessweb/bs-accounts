bitsongd config node $NODE
bitsongd config chain-id $CHAIN_ID
bitsongd config output json
 
if [ "$ADMIN_MULTISIG" = true ] ; then
    echo 'Using multisig'
    bitsongd tx wasm migrate $1 $MKT_CODE_ID {} \
        --gas-prices 0.025ubtsg --gas auto --gas-adjustment 1.9 \
        --from $ADMIN \
        --generate-only > unsignedTx-mkt.json

    bitsongd tx wasm migrate $2 $MINTER_CODE_ID {} \
        --gas-prices 0.025ubtsg --gas auto --gas-adjustment 1.9 \
        --from $ADMIN \
        --generate-only > unsignedTx-minter.json

    bitsongd tx wasm migrate $3 $COLLECTION_CODE_ID {} \
        --gas-prices 0.025ubtsg --gas auto --gas-adjustment 1.9 \
        --from $ADMIN \
        --generate-only > unsignedTx-collection.json

    bitsongd tx sign unsignedTx-mkt.json \
        --multisig=$ADMIN --from $USER --output-document=$KEY-mkt.json \
        --chain-id $CHAIN_ID

    bitsongd tx sign unsignedTx-minter.json \
        --multisig=$ADMIN --from $USER --output-document=$KEY-minter.json \
        --chain-id $CHAIN_ID

    bitsongd tx sign unsignedTx-collection.json \
        --multisig=$ADMIN --from $USER --output-document=$KEY-collection.json \
        --chain-id $CHAIN_ID
else
    echo 'Using single signer'
    bitsongd tx wasm migrate $1 $MKT_CODE_ID {} \
        --gas-prices 0.025ubtsg --gas auto --gas-adjustment 1.9 \
        --from $ADMIN -y -b block -o json | jq .

    bitsongd tx wasm migrate $2 $MINTER_CODE_ID {} \
        --gas-prices 0.025ubtsg --gas auto --gas-adjustment 1.9 \
        --from $ADMIN -y -b block -o json | jq .

    bitsongd tx wasm migrate $3 $COLLECTION_CODE_ID {} \
        --gas-prices 0.025ubtsg --gas auto --gas-adjustment 1.9 \
        --from $ADMIN -y -b block -o json | jq .

# bitsongd tx wasm migrate $4 $WHITELIST_CODE_ID {} \
#     --gas-prices 0.025ubtsg --gas auto --gas-adjustment 1.9 \
#     --from $ADMIN -y -b block -o json | jq .

fi