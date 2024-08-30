MSG=$(cat <<EOF
{
  "account": { "address": "$USER" }
}
EOF
)

bitsongd q wasm contract-state smart $COLLECTION "$MSG"
 
