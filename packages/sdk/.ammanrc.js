// @ts-check
'use strict';
const path = require('path');
const { LOCALHOST, tmpLedgerDir } = require('@metaplex-foundation/amman');

function localDeployPath(programName) {
  return path.join(__dirname, 'test', `${programName}.so`);
}

const validator = {
  killRunningValidators: true,
  programs: [
    {
      label: 'Token Auth Rules',
      programId: 'auth9SigNpDKz4sJJ1DfCTuZrZNSAgh9sFD3rboVmgg',
      deployPath: localDeployPath('mpl_token_auth_rules'),
    },
  ],
  commitment: 'singleGossip',
  resetLedger: true,
  verifyFees: false,
  jsonRpcUrl: LOCALHOST,
  websocketUrl: '',
  ledgerDir: tmpLedgerDir(),
  accountsCluster: 'https://api.devnet.solana.com',
};

module.exports = { validator };
