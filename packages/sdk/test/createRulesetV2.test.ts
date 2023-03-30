import { Keypair } from '@solana/web3.js';
import { encode } from '@msgpack/msgpack';
import test from 'ava';
import {
  additionalSignerV2,
  getLatestRuleSet,
  pubkeyListMatchV2,
  RuleSetV2,
  serializeRuleSetV2,
} from '../src/mpl-token-auth-rules';
import {
  createOrUpdateLargeRuleset,
  createOrUpdateRuleset,
  getConnectionAndPayer,
  writeAndPuff,
} from './_setup';

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

test('it can update a ruleset from v2 to v1', async (t) => {
  // Given a serialized ruleset v2 account data.
  const { connection, payer } = await getConnectionAndPayer();
  const name = 'My Rule Set';

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

  // When we create a new ruleset account using the v2 data.
  const ruleSetPda = await createOrUpdateRuleset(connection, payer, name, serializedRuleSetV2);

  // Then the latest ruleset is a ruleset v2.
  const updatedRawRuleSetPdaAccount = await connection.getAccountInfo(ruleSetPda);
  const updatedLatestDeserializedRuleSet = getLatestRuleSet(
    updatedRawRuleSetPdaAccount?.data,
  ) as RuleSetV2;
  t.deepEqual(updatedLatestDeserializedRuleSet, ruleSetV2);

  // Additionally, Given a ruleset v1 account data.

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

  // When we update the ruleset account using the v1 data.
  await createOrUpdateRuleset(connection, payer, name, serializedRuleSetV1);

  // Then the latest ruleset is a ruleset v1.
  const rawRuleSetPdaAccount = await connection.getAccountInfo(ruleSetPda);
  const latestDeserializedRuleSet = getLatestRuleSet(rawRuleSetPdaAccount?.data) as string;
  t.is(latestDeserializedRuleSet, JSON.stringify(ruleSetV1, null, 2));
});

test('it can create a ruleset v2 from a buffer account', async (t) => {
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

  // Creating a buffer account.
  const bufferPda = await writeAndPuff(connection, payer, name, serializedRuleSet);

  // When we create a new ruleset account using it.
  const ruleSetPda = await createOrUpdateRuleset(connection, payer, name, bufferPda);

  // Then we can deserialize the account data and get the same ruleset.
  const rawRuleSetPdaAccount = await connection.getAccountInfo(ruleSetPda);
  const deserializedRuleSet = getLatestRuleSet(rawRuleSetPdaAccount?.data) as RuleSetV2;
  t.deepEqual(deserializedRuleSet, ruleSet);
});

test('it can create a large ruleset v2 from a buffer account', async (t) => {
  // Given a serialized ruleset v2 account data.
  const { connection, payer } = await getConnectionAndPayer();
  const name = 'My Large Rule Set';
  const ruleSet: RuleSetV2 = {
    name,
    owner: payer.publicKey,
    operations: {
      transfer: pubkeyListMatchV2(
        'Destination',
        [...Array(350)].map(() => Keypair.generate().publicKey),
      ),
    },
  };
  const serializedRuleSet = serializeRuleSetV2(ruleSet);

  // When we create a new ruleset account using it.
  const ruleSetPda = await createOrUpdateLargeRuleset(connection, payer, name, serializedRuleSet);

  // Then we can deserialize the account data and get the same ruleset.
  const rawRuleSetPdaAccount = await connection.getAccountInfo(ruleSetPda);
  const deserializedRuleSet = getLatestRuleSet(rawRuleSetPdaAccount?.data) as RuleSetV2;
  t.deepEqual(deserializedRuleSet, ruleSet);
});
