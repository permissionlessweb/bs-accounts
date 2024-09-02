import codegen from "@cosmwasm/ts-codegen";

codegen({
  contracts: [
    // {
    //   name: "AccountMarketplace",
    //   dir: "../contracts/account-market/schema",
    // },
    {
      name: "AccountMinter",
      dir: "../contracts/bs721-account-minter/schema",
    },
    {
      name: "Bs721Account",
      dir: "../contracts/bs721-account/schema",
    },
  ],
  outPath: "./src/",

  // options are completely optional ;)
  options: {
    bundle: {
      bundleFile: "index.ts",
      scope: "contracts",
    },
    types: {
      enabled: true,
    },
    client: {
      enabled: true,
    },
    reactQuery: {
      enabled: false,
      optionalClient: true,
      version: "v4",
      mutations: true,
      queryKeys: true,
    },
    recoil: {
      enabled: false,
    },
    messageComposer: {
      enabled: true,
    },
  },
}).then(() => {
  console.log("✨ all done!");
});
