bitsongd config node $NODE
bitsongd config chain-id $CHAIN_ID
bitsongd config output json

USER=terp1n5x097nd7v8dv8ng4x4xeux5xdv6jas6ekwwkn
MKT_CODE_ID=167
KEY=$(bitsongd keys show $USER | jq -r .name)



MSG=$(cat <<EOF
{
  "trading_fee_bps": 200,
  "min_price": "5000000",
  "ask_interval": 60,
  "max_renewals_per_block": 1,
  "valid_bid_query_limit": 30,
  "renew_window": 30,
  "renewal_bid_percentage": "1000000000000000000",
  "operator": "terp1n5x097nd7v8dv8ng4x4xeux5xdv6jas6ekwwkn"
}
EOF
)
 
  echo 'Using single signer'
  bitsongd tx wasm instantiate $MKT_CODE_ID "$MSG" --label "NameMarketplace" \
    --admin terp1n5x097nd7v8dv8ng4x4xeux5xdv6jas6ekwwkn \
    --gas auto --gas-adjustment 1.9  --fees 20000ubtsg \
    --from ica -y -o json | jq .

    # E16B68542DE6C0B6ABF5FB285811DC6AF1B0A73BB2AFE0BA34377A4A9AF99192
    # terp1063teh2nt8vqvgm8f2pckvwy5sle8x8xauldjuanq5e6g92e7s9sfntm6s
