# Smart-Accounts: Cosmwasm Authentication Demos

welcome!

## Thoughts On Account Token Security Design

Since account authenticators are a direct membrane for account authorization, it is essential to keep in mind the security vulnurabilities that exists in a multi authenticator system that involved the your accounts . Below will list a few obvious examples, but as always security is never static and is unique to your decisions.

### Centralized Smart Contract Ownership & Migration Attack

If the code-id of the accounts used has a global contract admin, then the possible risk of the admins integrity being compromised and an attack may occur on wallets actions via contract migration. Always check ownership parameters for smart contracts (and tokens).

### Undesired Operator Or Approval Authorization

Whenever browsing dapps with the account that is currently under ownership of the account token, it is crucial to keep concious of any unwantedauthorized operator or approval messages during interations with them, as this may result in full compromise of your account without delays  including other smart contracts, then this may put  account tokens at risk.
