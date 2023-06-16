const path = require("path");

const programDir = path.join(__dirname, "..", "programs");
function getProgram(dir, programName) {
  return path.join(programDir, dir, "target", "deploy", programName);
}

module.exports = {
  validator: {
    commitment: "processed",
    accountsCluster: "https://api.mainnet-beta.solana.com/",
    programs: [
      {
        label: "Token Auth Rules",
        programId: "auth9SigNpDKz4sJJ1DfCTuZrZNSAgh9sFD3rboVmgg",
        deployPath: getProgram("token-auth-rules", "mpl_token_auth_rules.so"),
      },
    ],
    accounts: [
      {
        label: "Metaplex Default RuleSet",
        accountId: "eBJLFYPxJmMGKuFwpDWkzxZeUrad92kZRC5BJLpzyT9",
        executable: false,
      },
    ],
  },
};
