/* eslint-disable @typescript-eslint/no-explicit-any */
import { Keypair } from '@solana/web3.js';
import { encode } from '@msgpack/msgpack';
import test from 'ava';
import {
  additionalSignerV2,
  allV2,
  amountV2,
  anyV2,
  getLatestRuleSet,
  namespaceV2,
  notV2,
  passV2,
  pdaMatchV2,
  programOwnedListV2,
  programOwnedTreeV2,
  programOwnedV2,
  PROGRAM_ID,
  pubkeyListMatchV2,
  pubkeyMatchV2,
  pubkeyTreeMatchV2,
  RuleSetV2,
  serializeRuleSetV2,
  RuleSetV1,
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
  const ruleSetV1: RuleSetV1 = {
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
  const latestDeserializedRuleSet = getLatestRuleSet(rawRuleSetPdaAccount?.data);
  t.deepEqual(latestDeserializedRuleSet, ruleSetV1);

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
  const updatedLatestDeserializedRuleSet = getLatestRuleSet(updatedRawRuleSetPdaAccount?.data);
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

  const ruleSetV1: RuleSetV1 = {
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
  const latestDeserializedRuleSet = getLatestRuleSet(rawRuleSetPdaAccount?.data) as RuleSetV1;
  t.deepEqual(latestDeserializedRuleSet, ruleSetV1);
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

test('it can create a composed ruleset v2', async (t) => {
  // Given a serialized composed ruleset v2 account data.
  const { connection, payer } = await getConnectionAndPayer();
  const name = 'My Composed Rule Set';

  const ruleSet: RuleSetV2 = {
    name,
    owner: payer.publicKey,
    operations: {
      'Transfer:Holder': anyV2([
        allV2([
          additionalSignerV2(Keypair.generate().publicKey),
          additionalSignerV2(Keypair.generate().publicKey),
        ]),
        notV2(amountV2('Amount', '=', 1)),
      ]),
    },
  };
  const serializedRuleSet = serializeRuleSetV2(ruleSet);

  // When we create a new ruleset account using it.
  const ruleSetPda = await createOrUpdateRuleset(connection, payer, name, serializedRuleSet);

  // Then we can deserialize the account data and get the same ruleset.
  const rawRuleSetPdaAccount = await connection.getAccountInfo(ruleSetPda);
  const deserializedRuleSet = getLatestRuleSet(rawRuleSetPdaAccount?.data) as RuleSetV2;
  // convert the deserialized BN to a number
  (deserializedRuleSet.operations['Transfer:Holder'] as any).rules[1].rule.amount = Number(
    (deserializedRuleSet.operations['Transfer:Holder'] as any).rules[1].rule.amount,
  );

  t.deepEqual(deserializedRuleSet, ruleSet);
});

test('it can create a ruleset v2 with all rule types', async (t) => {
  // Given a serialized ruleset v2 using all rule types account data.
  const { connection, payer } = await getConnectionAndPayer();
  const name = 'My Composed Rule Set';

  const ruleSet: RuleSetV2 = {
    name,
    owner: payer.publicKey,
    operations: {
      'Transfer:Holder': anyV2([
        allV2([
          additionalSignerV2(Keypair.generate().publicKey),
          additionalSignerV2(Keypair.generate().publicKey),
        ]),
        notV2(amountV2('Amount', '=', 1)),
        pubkeyMatchV2('Destination', Keypair.generate().publicKey),
        programOwnedListV2('Source', [PROGRAM_ID]),
      ]),
      'Transfer:Delegate': anyV2([
        allV2([
          additionalSignerV2(Keypair.generate().publicKey),
          additionalSignerV2(Keypair.generate().publicKey),
          namespaceV2(),
        ]),
        notV2(programOwnedV2('Destination', PROGRAM_ID)),
        passV2(),
        pubkeyTreeMatchV2(
          'Source',
          'Proof',
          new Uint8Array([...Array(32)].map(() => Math.floor(Math.random() * 40))),
        ),
      ]),
      'Transfer:Authority': anyV2([
        pubkeyListMatchV2('Destination', [Keypair.generate().publicKey]),
        pdaMatchV2('Destination', PROGRAM_ID, 'Seed'),
        programOwnedV2('Source', PROGRAM_ID),
        programOwnedTreeV2(
          'Source',
          'Proof',
          new Uint8Array([...Array(32)].map(() => Math.floor(Math.random() * 40))),
        ),
      ]),
    },
  };
  const serializedRuleSet = serializeRuleSetV2(ruleSet);

  // When we create a new ruleset account using it.
  const ruleSetPda = await createOrUpdateLargeRuleset(connection, payer, name, serializedRuleSet);

  // Then we can deserialize the account data and get the same ruleset.
  const rawRuleSetPdaAccount = await connection.getAccountInfo(ruleSetPda);
  const deserializedRuleSet = getLatestRuleSet(rawRuleSetPdaAccount?.data) as RuleSetV2;
  // convert the deserialized BN to a number
  (deserializedRuleSet.operations['Transfer:Holder'] as any).rules[1].rule.amount = Number(
    (deserializedRuleSet.operations['Transfer:Holder'] as any).rules[1].rule.amount,
  );

  t.deepEqual(deserializedRuleSet, ruleSet);
});

test('it can update a ruleset v2', async (t) => {
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

  // Given a updated ruleset v2 account data.

  const updatedRuleSetV2: RuleSetV2 = {
    name,
    owner: payer.publicKey,
    operations: {
      deposit: pubkeyListMatchV2('Source', [publicKeyA]),
      withdraw: pubkeyMatchV2('Source', publicKeyB),
    },
  };
  const updatedSerializedRuleSetV2 = serializeRuleSetV2(updatedRuleSetV2);

  // When we update the ruleset account using the v2 data.
  await createOrUpdateRuleset(connection, payer, name, updatedSerializedRuleSetV2);

  // Then the latest ruleset is a ruleset v2.
  const rawRuleSetPdaAccount = await connection.getAccountInfo(ruleSetPda);
  const latestDeserializedRuleSet = getLatestRuleSet(rawRuleSetPdaAccount?.data) as RuleSetV2;
  t.deepEqual(latestDeserializedRuleSet, updatedRuleSetV2);
});
