MSG=$(cat <<EOF
{
  "approve_all": {
    "operator": "$MKT"
  }
}
EOF
)

bitsongd tx wasm execute $COLLECTION "$MSG" \
  --gas-prices 0.025ubtsg --gas auto --gas-adjustment 1.9 \
  --from $USER -b block -y -o json | jq .
 

MSG=$(cat <<EOF
{
  "mint_and_list": {
    "account": "$1"
  }
}
EOF
)

bitsongd tx wasm execute $MINTER "$MSG" \
  --amount 50000000ubtsg \
  --gas-prices 0.025ubtsg --gas auto --gas-adjustment 1.9 \
  --from $USER -b block -y -o json | jq .
 