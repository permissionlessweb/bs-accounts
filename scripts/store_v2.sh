bitsongd config node $NODE
bitsongd config chain-id $CHAIN_ID
bitsongd config output json

bitsongd tx wasm store artifacts/name_marketplace.wasm --from hot-wallet \
    --keyring-backend test \
    --gas-prices 0.025ubtsg --gas-adjustment 1.7 \
    --gas auto -y -b block -o json | jq '.logs' | grep -A 1 code_id

    # bitsongd tx wasm store artifacts/sg721_name.wasm --from hot-wallet \
    #     --keyring-backend test \
    #     --gas-prices 0.025ubtsg --gas-adjustment 1.7 \
    #     --gas auto -y -b block -o json | jq '.logs' | grep -A 1 code_id

    