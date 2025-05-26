#!/bin/bash

for d in ../artifacts/bs721_account.wasm; do 
echo $d;
bitsongd tx wasm store ../artifacts/bs721_account.wasm  --from ica --gas auto --fees 1000000ubtsg --gas-adjustment 2 -y
done 
sleep 6
# # 05AFAFEB0DD83201FDF89DF98BE64BF01F224BAA95E8C99892BB1D4492C8B2DB (166)

for d in ../artifacts/bs721_account_marketplace.wasm; do 
echo $d;
bitsongd tx wasm store ../artifacts/bs721_account_marketplace.wasm  --from ica --gas auto --fees 1000000ubtsg --gas-adjustment 2 -y
done 
sleep 6
# # 2E950E4304EA28CCA25A69DB0B09DDD13E6F9DE45AC2A5FB33A7A1CF5485EC09 (167)

for d in ../artifacts/bs721_account_minter.wasm; do 
echo $d;
bitsongd tx wasm store ../artifacts/bs721_account_minter.wasm  --from ica --gas auto --fees 1000000ubtsg --gas-adjustment 2 -y
done 
sleep 6
# EED5DBEEE40D817AF1E6BC8D31BEB2C2FD1DB6733E56CC4947BC1CCCDE5EB89D (168)