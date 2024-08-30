bitsongd config node $NODE
bitsongd config chain-id $CHAIN_ID
bitsongd config output json

KEY=$(bitsongd keys show $USER | jq -r .name)

MSG=$(cat <<EOF
{
  "trading_fee_bps": 200,
  "min_price": "5000000",
  "ask_interval": 60
}
EOF
)
 
if [ "$ADMIN_MULTISIG" = true ] ; then
  echo 'Using multisig'
  bitsongd tx wasm instantiate $MKT_CODE_ID "$MSG" --label "NameMarketplace" \
    --admin $ADMIN \
    --gas-prices 0.025ubtsg --gas auto --gas-adjustment 1.9 \
    --from $ADMIN \
    --generate-only > unsignedTx.json

  bitsongd tx sign unsignedTx.json \
    --multisig=$ADMIN --from $USER --output-document=$KEY.json \
    --chain-id $CHAIN_ID
else
  echo 'Using single signer'
  bitsongd tx wasm instantiate $MKT_CODE_ID "$MSG" --label "NameMarketplace" \
    --admin $ADMIN \
    --gas-prices 0.025ubtsg --gas auto --gas-adjustment 1.9 \
    --from $ADMIN -y -b block -o json | jq .
fi