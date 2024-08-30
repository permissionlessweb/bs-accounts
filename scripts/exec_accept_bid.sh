MSG=$(cat <<EOF
{
  "approve": {
    "spender": "$MKT",
    "token_id": "$1"
  }
}
EOF
)

bitsongd tx wasm execute $COLLECTION "$MSG" \
  --gas-prices 0.025ubtsg --gas auto --gas-adjustment 1.9 \
  --from $USER -b block -y -o json | jq .
 

MSG=$(cat <<EOF
{
  "accept_bid": {
    "token_id": "$1",
    "bidder": "$2"
  }
}
EOF
)

bitsongd tx wasm execute $MKT "$MSG" \
  --amount $3000000ubtsg \
  --gas-prices 0.025ubtsg --gas auto --gas-adjustment 1.9 \
  --from $USER -b block -y -o json | jq .
 