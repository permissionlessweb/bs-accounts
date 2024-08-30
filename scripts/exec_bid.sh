MSG=$(cat <<EOF
{
  "set_bid": {
    "token_id": "$1"
  }
}
EOF
)

bitsongd tx wasm execute $MKT "$MSG" \
  --amount 6000000ubtsg \
  --gas-prices 0.025ubtsg --gas auto --gas-adjustment 1.9 \
  --from $BIDDER -y -b block -o json | jq .
 
