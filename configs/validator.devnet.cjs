const localnet = require("./validator.cjs");

module.exports = {
  ...localnet,
  validator: {
    ...localnet.validator,
    programs: [],
    accountsCluster: "https://api.devnet.solana.com/",
    accounts: [
      ...localnet.validator.accounts,
      ...(localnet.validator.programs ?? []).map((program) => ({
        ...program,
        accountId: program.programId,
        executable: true,
      })),
    ],
  },
};
