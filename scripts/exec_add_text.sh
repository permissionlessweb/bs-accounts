MSG=$(cat <<EOF
{
  "update_text_record": {
    "account": "$1",
    "record": {
      "account": "twitter",
      "value": "something"
    }
  }
}
EOF
)

bitsongd tx wasm execute $COLLECTION "$MSG" \
  --gas-prices 0.025ubtsg --gas auto --gas-adjustment 1.9 \
  --from $USER -y -b block -o json | jq .
 