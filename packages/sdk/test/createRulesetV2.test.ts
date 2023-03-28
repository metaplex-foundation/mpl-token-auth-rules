import { Keypair } from '@solana/web3.js';
import { encode } from '@msgpack/msgpack';
import test from 'ava';
import {
  additionalSignerV2,
  getLatestRuleSet,
  RuleSetV2,
  serializeRuleSetV2,
} from '../src/mpl-token-auth-rules';
import { createOrUpdateRuleset, getConnectionAndPayer } from './_setup';

test('it can create a ruleset v2', async (t) => {
  // Given a serialized ruleset v2 account data.
  const { connection, payer } = await getConnectionAndPayer();
  const publicKeyA = Keypair.generate().publicKey;
  const publicKeyB = Keypair.generate().publicKey;
  const name = 'My Rule Set';
  const ruleSet: RuleSetV2 = {
    name,
    owner: payer.publicKey,
    operations: {
      deposit: additionalSignerV2(publicKeyA),
      withdraw: additionalSignerV2(publicKeyB),
    },
  };
  const serializedRuleSet = serializeRuleSetV2(ruleSet);

  // When we create a new ruleset account using it.
  const ruleSetPda = await createOrUpdateRuleset(connection, payer, name, serializedRuleSet);

  // Then we can deserialize the account data and get the same ruleset.
  const rawRuleSetPdaAccount = await connection.getAccountInfo(ruleSetPda);
  const deserializedRuleSet = getLatestRuleSet(rawRuleSetPdaAccount?.data) as RuleSetV2;
  t.deepEqual(deserializedRuleSet, ruleSet);
});

test('it can update a ruleset from v1 to v2', async (t) => {
  // Given a ruleset v1 account data.
  const { connection, payer } = await getConnectionAndPayer();
  const name = 'My Rule Set';
  const ruleSetV1 = {
    libVersion: 1,
    ruleSetName: name,
    owner: Array.from(payer.publicKey.toBytes()),
    operations: {
      Transfer: {
        ProgramOwned: {
          program: Array.from(payer.publicKey.toBytes()),
          field: 'Destination',
        },
      },
    },
  };
  const serializedRuleSetV1 = encode(ruleSetV1);

  // When we create a new ruleset account using the v1 data.
  const ruleSetPda = await createOrUpdateRuleset(connection, payer, name, serializedRuleSetV1);

  // Then the latest ruleset is a ruleset v1.
  const rawRuleSetPdaAccount = await connection.getAccountInfo(ruleSetPda);
  const latestDeserializedRuleSet = getLatestRuleSet(rawRuleSetPdaAccount?.data) as string;
  t.is(latestDeserializedRuleSet, JSON.stringify(ruleSetV1, null, 2));

  // Additionally, Given a serialized ruleset v2 account data.
  const publicKeyA = Keypair.generate().publicKey;
  const publicKeyB = Keypair.generate().publicKey;
  const ruleSetV2: RuleSetV2 = {
    name,
    owner: payer.publicKey,
    operations: {
      deposit: additionalSignerV2(publicKeyA),
      withdraw: additionalSignerV2(publicKeyB),
    },
  };
  const serializedRuleSetV2 = serializeRuleSetV2(ruleSetV2);

  // When we update the ruleset account using the v2 data.
  await createOrUpdateRuleset(connection, payer, name, serializedRuleSetV2);

  // Then the latest ruleset is a ruleset v2.
  const updatedRawRuleSetPdaAccount = await connection.getAccountInfo(ruleSetPda);
  const updatedLatestDeserializedRuleSet = getLatestRuleSet(
    updatedRawRuleSetPdaAccount?.data,
  ) as RuleSetV2;
  t.deepEqual(updatedLatestDeserializedRuleSet, ruleSetV2);
});
