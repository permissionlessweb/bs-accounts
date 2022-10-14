MSG=$(cat <<EOF
{
  "mint_and_list": {
    "name": "$1"
  }
}
EOF
)

starsd tx wasm execute $MINTER "$MSG" \
  --amount 1000000000ustars \
  --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 --from testnet -y
 