MSG=$(cat <<EOF
{
  "config": {}
}
EOF
)

bitsongd q wasm contract-state smart $MKT "$MSG"
 
