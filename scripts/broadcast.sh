bitsongd tx multisign unsignedTx.json $MULTISIG_NAME $1 $2 $3 > signedTx.json

bitsongd tx broadcast signedTx.json