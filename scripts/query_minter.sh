MSG=$(cat <<EOF
{
  "collection": {}
}
EOF
)

bitsongd q wasm contract-state smart $MINTER "$MSG"
 

MSG=$(cat <<EOF
{
  "admin": {}
}
EOF
)

bitsongd q wasm contract-state smart $MINTER "$MSG"


MSG=$(cat <<EOF
{
  "whitelists": {}
}
EOF
)

bitsongd q wasm contract-state smart $MINTER "$MSG"

MSG=$(cat <<EOF
{
  "config": {}
}
EOF
)

bitsongd q wasm contract-state smart $MINTER "$MSG"