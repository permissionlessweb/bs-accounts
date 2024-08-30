MSG=$(cat <<EOF
{
  "associate_address": {
    "account": "$1",
    "address": "$USER"
  }
}
EOF
)

bitsongd tx wasm execute $COLLECTION "$MSG" \
  --gas-prices 0.025ubtsg --gas auto --gas-adjustment 1.9 \
  --from $USER -y -b block -o json | jq .
 