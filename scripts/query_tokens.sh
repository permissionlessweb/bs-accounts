MSG=$(cat <<EOF
{
  "tokens": {"owner": "$USER"}
}
EOF
)

bitsongd q wasm contract-state smart $COLLECTION "$MSG"
 
