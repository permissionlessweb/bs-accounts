

## Thoughts On Account Token Security Design 
Since account tokens are the key for smart accounts, it is essential to keep in mind the security vulnurabilities that exists in a multi contract system that involved the smart contract of your Bitsong Account Token. Below will list a few obvious examples, but as always security is never static.

### Centralized Smart Contract Ownership & Migration Attack
If the code-id of the accounts used has a global contract admin, then the possible risk of the admins integrity being compromised, and an attack may occur on wallets. Bitsong mitigates this risk by having the governance module set as the admin able to migrate a contract for all of the contracts used in the accounts framework. 


There is still a risk of any wallet address compromized that is also authorized to execute actions as the module, however there are none. 

### Undesired Operator Or Approval Authorization
Whenever browsing dapps with the account that is currently under ownership of the account token, it is crucial to keep concious of any unwantedauthorized operator or approval messages during interations with them, as this may result in full compromise of your account without delays  including other smart contracts, then this may put  account tokens at risk. 
