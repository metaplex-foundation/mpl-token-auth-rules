import { createOrUpdateLargeRuleset } from './helpers/mpl-token-auth-rules';
import { Keypair, Connection, LAMPORTS_PER_SOL, PublicKey } from '@solana/web3.js';
import { program } from 'commander';
import log from 'loglevel';
import * as fs from 'fs';
import {
  findRuleSetPDA,
  getLatestRuleSetRevision,
  getRuleSetRevisionFromJson,
  getRuleSetRevisionV2FromV1,
  isRuleSetV1,
  isRuleSetV2,
  serializeRuleSetRevision,
} from '@metaplex-foundation/mpl-token-auth-rules';
import colorizeJson from 'json-colorizer';

//-----------//
// Constants //
//-----------//

const MAINNET_DEFAULT_RPC = 'https://api.mainnet-beta.solana.com';

const DEVNET_DEFAULT_RPC = 'https://api.devnet.solana.com';

const LOCALNET_DEFAULT_RPC = 'http://127.0.0.1:8899';

//-----------//
// Commands  //
//-----------//

program
  .command('create')
  .description('creates a new rule set revision')
  .option('-e, --env <string>', 'Solana cluster env name', 'devnet')
  .option('-r, --rpc <string>', 'The endpoint to connect to')
  .option('-k, --keypair <file>', 'Solana wallet file')
  .option('-l, --log-level <string>', 'Log level', setLogLevel)
  .option('--revision <file>', 'The rule set revision json file')
  .action(async (_directory, cmd) => {
    let { keypair, env, rpc, revision } = cmd.opts();

    rpc = rpc ?? envDefault(env);

    const connection = new Connection(rpc, 'finalized');
    const payer = loadKeypair(keypair);

    // Airdrop some Sol if we're on localnet.
    if (rpc == LOCALNET_DEFAULT_RPC) {
      const airdropSignature = await connection.requestAirdrop(
        payer.publicKey,
        10 * LAMPORTS_PER_SOL,
      );
      await connection.confirmTransaction(airdropSignature);
    }

    const ruleSet = getRuleSetRevisionFromJson(fs.readFileSync(revision, 'utf-8'));

    const owner = new PublicKey(ruleSet.owner);
    const name = isRuleSetV1(ruleSet) ? ruleSet.ruleSetName : ruleSet.name;

    if (!payer.publicKey.equals(owner)) {
      throw new Error('The payer must be the owner of the rule set.');
    }

    const [ruleSetAddress] = await findRuleSetPDA(payer.publicKey, name);
    console.log(`📝 Creating rule set revision for '${ruleSetAddress}'`);

    let rulesetAddress = await createOrUpdateLargeRuleset(
      connection,
      payer,
      name,
      serializeRuleSetRevision(ruleSet),
    );

    let rulesetData = await connection.getAccountInfo(rulesetAddress);
    let data = rulesetData?.data as Buffer;
    let latest = getLatestRuleSetRevision(data);

    console.log(`\n✅ Revision V${latest.libVersion} created.`);
  });

program
  .command('convert')
  .description('converts a rule set revision from V1 to V2')
  .option('-e, --env <string>', 'Solana cluster env name', 'devnet')
  .option('-r, --rpc <string>', 'The endpoint to connect to')
  .option('-k, --keypair <path>', 'Solana wallet file')
  .option('-l, --log-level <string>', 'Log level', setLogLevel)
  .option('-a, --address <string>', 'The address of the rule set')
  .action(async (_directory, cmd) => {
    let { keypair, env, rpc, address } = cmd.opts();

    rpc = rpc ?? envDefault(env);
    const connection = new Connection(rpc, 'finalized');

    console.log(`🔎 Retrieving latest rule set revision for '${address}'`);

    let rulesetPDA = new PublicKey(address);
    let rulesetData = await connection.getAccountInfo(rulesetPDA);
    let data = rulesetData?.data as Buffer;
    let ruleset = getLatestRuleSetRevision(data);

    if (isRuleSetV2(ruleset)) {
      console.log('\n✅ Latest rule set revision is already V2.');
      return;
    }

    const ruleSetV2 = getRuleSetRevisionV2FromV1(ruleset);
    console.log('   + updating rule set revision...');

    const owner = new PublicKey(ruleSetV2.owner);
    const name = ruleSetV2.name;
    const payer = loadKeypair(keypair);

    if (!payer.publicKey.equals(owner)) {
      throw new Error('The payer must be the owner of the rule set.');
    }

    // Airdrop some Sol if we're on localnet.
    if (rpc == LOCALNET_DEFAULT_RPC) {
      const airdropSignature = await connection.requestAirdrop(
        payer.publicKey,
        10 * LAMPORTS_PER_SOL,
      );
      await connection.confirmTransaction(airdropSignature);
    }

    let rulesetAddress = await createOrUpdateLargeRuleset(
      connection,
      payer,
      name,
      serializeRuleSetRevision(ruleSetV2),
    );

    console.log('   + ...done.');
    console.log(
      '\nIf you are managing your rule set via a JSON file,' +
        ' use the print command to get the latest revision' +
        ' as a JSON object and update your file.',
    );

    console.log('\n✅ Your rule set was updated to V2.');
  });

program
  .command('print')
  .description('prints the latest rule set revision as a JSON object')
  .option('-e, --env <string>', 'Solana cluster env name', 'devnet')
  .option('-r, --rpc <string>', 'The endpoint to connect to')
  .option('-l, --log-level <string>', 'Log level', setLogLevel)
  .option('-a, --address <string>', 'The address of the rule set')
  .option('--pretty', 'Pretty print the JSON output')
  .option('-o, --output <file>', 'The file to save the output to')
  .action(async (_directory, cmd) => {
    let { env, rpc, address, pretty, output } = cmd.opts();

    rpc = rpc ?? envDefault(env);
    const connection = new Connection(rpc, 'finalized');

    console.log(`🔎 Retrieving latest rule set revision for '${address}'`);

    let rulesetPDA = new PublicKey(address);
    let rulesetData = await connection.getAccountInfo(rulesetPDA);
    let data = rulesetData?.data as Buffer;
    let ruleset = getLatestRuleSetRevision(data);

    if (output) {
      // output ignores the pretty flag
      console.log('   + writing revision to file');
      fs.writeFileSync(output, JSON.stringify(ruleset, null, 2));
    } else {
      console.log('\n' + colorizeJson(JSON.stringify(ruleset, pretty ? replacePubkey : undefined, 2)));
    }

    console.log(`\n✅ Revision retrieved.`);
  });

program.version('1.0.0').description('CLI for managing RuleSet revisions.').parse(process.argv);

//-----------//
// Helpers   //
//-----------//

// eslint-disable-next-line @typescript-eslint/no-unused-vars
function setLogLevel(value, prev) {
  if (value === undefined || value === null) {
    return;
  }
  log.info('setting the log value to: ' + value);
  log.setLevel(value);
}

function loadKeypair(keypairPath) {
  const decodedKey = new Uint8Array(JSON.parse(fs.readFileSync(keypairPath).toString()));

  return Keypair.fromSecretKey(decodedKey);
}

function envDefault(env) {
  switch (env) {
    case 'mainnet-beta':
      return MAINNET_DEFAULT_RPC;
    case 'devnet':
      return DEVNET_DEFAULT_RPC;
    case 'localnet':
      return LOCALNET_DEFAULT_RPC;
  }
}

function replacePubkey(key, value) {
    if (Array.isArray(value) && value.length == 32) {
        return new PublicKey(value).toString();
    }
    return value;
}
