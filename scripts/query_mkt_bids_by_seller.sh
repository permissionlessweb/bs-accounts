MSG=$(cat <<EOF
{
  "bids_for_seller": { "seller": "$1" }
}
EOF
)

bitsongd q wasm contract-state smart $MKT "$MSG"
 
