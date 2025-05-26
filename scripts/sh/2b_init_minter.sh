
ADMIN=
MKT=
COLLECTION_CODE_ID=
MINTER_CODE_ID=
MSG=$(cat <<EOF
{
  "collection_code_id": $COLLECTION_CODE_ID,
  "admin": "$ADMIN",
  "marketplace_addr": "$MKT",
  "min_account_length": 3,
  "max_account_length": 63,
  "base_price": "100000000"
}
EOF
)

bitsongd tx wasm instantiate $MINTER_CODE_ID "$MSG" --label "AccountMinter" \
  --admin $ADMIN \
  --fees 500000ubtsg --gas auto --gas-adjustment 1.9 \
  --from $ADMIN -y -o json | jq .
