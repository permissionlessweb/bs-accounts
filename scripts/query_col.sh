MSG=$(cat <<EOF
{
  "minter": {}
}
EOF
)

bitsongd q wasm contract-state smart $COLLECTION "$MSG"
 
