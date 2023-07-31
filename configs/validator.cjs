const path = require("path");

const accountDir = path.join(__dirname, "..", "programs");

function getAccount(accountBinary) {
  return path.join(accountDir, ".bin", accountBinary);
}

module.exports = {
  validator: {
    commitment: "processed",
    accountsCluster: "https://api.mainnet-beta.solana.com/",
    programs: [
      {
        label: "Token Auth Rules",
        programId: "auth9SigNpDKz4sJJ1DfCTuZrZNSAgh9sFD3rboVmgg",
        deployPath: getAccount("mpl_token_auth_rules.so"),
      },
    ],
    accounts: [
      {
        label: "Metaplex Default RuleSet",
        accountId: "eBJLFYPxJmMGKuFwpDWkzxZeUrad92kZRC5BJLpzyT9",
        deployPath: getAccount("metaplex-default-ruleset.bin"),
        executable: false,
      },
    ],
  },
};
