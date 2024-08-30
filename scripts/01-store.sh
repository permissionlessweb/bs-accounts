bitsongd config node $NODE
bitsongd config chain-id $CHAIN_ID
bitsongd config output json

bitsongd tx wasm store name_marketplace.wasm --from $GOV \
    --instantiate-only-address $ADMIN \
    --gas-prices 0.025ubtsg --gas-adjustment 1.7 \
    --gas auto -y -b block -o json | jq '.logs' | grep -A 1 code_id

bitsongd tx wasm store name_minter.wasm --from $GOV \
    --instantiate-only-address $ADMIN \
    --gas-prices 0.025ubtsg --gas-adjustment 1.7 \
    --gas auto -y -b block -o json | jq '.logs' | grep -A 1 code_id

bitsongd tx wasm store sg721_name.wasm --from $GOV \
    --gas-prices 0.025ubtsg --gas-adjustment 1.7 \
    --gas auto -y -b block -o json | jq '.logs' | grep -A 1 code_id

bitsongd tx wasm store whitelist_updatable.wasm --from $GOV \
    --gas-prices 0.025ubtsg --gas-adjustment 1.7 \
    --gas auto -y -b block -o json | jq '.logs' | grep -A 1 code_id
