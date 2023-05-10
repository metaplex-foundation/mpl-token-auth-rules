/* eslint-disable @typescript-eslint/no-explicit-any */
import { Keypair } from '@solana/web3.js';
import test from 'ava';
import {
  AmountOperator,
  PROGRAM_ID,
  RuleSetV1,
  getLatestRuleSet,
  serializeRuleSetRevision,
} from '../src/mpl-token-auth-rules';
import { createOrUpdateLargeRuleset, createOrUpdateRuleset, getConnectionAndPayer } from './_setup';

test('it can create a ruleset v1', async (t) => {
  // Given a serialized ruleset v1 account data.
  const { connection, payer } = await getConnectionAndPayer();
  const publicKeyA = Keypair.generate().publicKey;
  const publicKeyB = Keypair.generate().publicKey;
  const name = 'My Rule Set';
  const ruleSet: RuleSetV1 = {
    libVersion: 1,
    ruleSetName: name,
    owner: [...payer.publicKey.toBytes()],
    operations: {
      deposit: {
        AdditionalSigner: { account: [...publicKeyA.toBytes()] },
      },
      withdraw: {
        AdditionalSigner: { account: [...publicKeyB.toBytes()] },
      },
    },
  };
  const serializedRuleSet = serializeRuleSetRevision(ruleSet);

  // When we create a new ruleset account using it.
  const ruleSetPda = await createOrUpdateRuleset(connection, payer, name, serializedRuleSet);

  // Then we can deserialize the account data and get the same ruleset.
  const rawRuleSetPdaAccount = await connection.getAccountInfo(ruleSetPda);
  const deserializedRuleSet = getLatestRuleSet(rawRuleSetPdaAccount?.data) as RuleSetV1;
  t.deepEqual(deserializedRuleSet, ruleSet);
});

test.skip('it can create a ruleset v1 with all rule types', async (t) => {
  // Given a serialized ruleset v2 using all rule types account data.
  const { connection, payer } = await getConnectionAndPayer();
  const name = 'My Composed Rule Set';

  const ruleSet: RuleSetV1 = {
    libVersion: 1,
    ruleSetName: name,
    owner: [...payer.publicKey.toBytes()],
    operations: {
      'Transfer:Holder': {
        Any: {
          rules: [
            {
              All: {
                rules: [
                  { AdditionalSigner: { account: [...Keypair.generate().publicKey.toBytes()] } },
                  { AdditionalSigner: { account: [...Keypair.generate().publicKey.toBytes()] } },
                ],
              },
            },
            {
              Not: {
                rule: {
                  Amount: { amount: 1, operator: AmountOperator.Eq, field: 'Amount' },
                },
              },
            },
            {
              PubkeyMatch: {
                pubkey: [...Keypair.generate().publicKey.toBytes()],
                field: 'Destination',
              },
            },
            {
              ProgramOwnedList: {
                programs: [[...PROGRAM_ID.toBytes()]],
                field: 'Source',
              },
            },
          ],
        },
      },
      'Transfer:Delegate': {
        Any: {
          rules: [
            {
              All: {
                rules: [
                  { AdditionalSigner: { account: [...Keypair.generate().publicKey.toBytes()] } },
                  { AdditionalSigner: { account: [...Keypair.generate().publicKey.toBytes()] } },
                  'Namespace',
                ],
              },
            },
            {
              Not: {
                rule: {
                  ProgramOwned: { program: [...PROGRAM_ID.toBytes()], field: 'Destination' },
                },
              },
            },
            'Pass',
            // {
            //   PubkeyTreeMatch: {
            //     root: [...Array(32)].map(() => Math.floor(Math.random() * 40)),
            //     pubkeyField: 'Source',
            //     proofField: 'Proof',
            //   },
            // },
          ],
        },
      },
      'Transfer:Authority': {
        Any: {
          rules: [
            {
              PubkeyListMatch: {
                pubkeys: [[...Keypair.generate().publicKey.toBytes()]],
                field: 'Destination',
              },
            },
            // {
            //   PDAMatch: {
            //     program: [...PROGRAM_ID.toBytes()],
            //     pdaField: 'Destination',
            //     seedsField: 'Seed',
            //   },
            // },
            {
              ProgramOwned: {
                program: [...PROGRAM_ID.toBytes()],
                field: 'Source',
              },
            },
            // {
            //   ProgramOwnedTree: {
            //     root: [...Array(32)].map(() => Math.floor(Math.random() * 40)),
            //     pubkeyField: 'Source',
            //     proofField: 'Proof',
            //   },
            // },
          ],
        },
      },
    },
  };
  const serializedRuleSet = serializeRuleSetRevision(ruleSet);

  // When we create a new ruleset account using it.
  const ruleSetPda = await createOrUpdateLargeRuleset(connection, payer, name, serializedRuleSet);

  // Then we can deserialize the account data and get the same ruleset.
  const rawRuleSetPdaAccount = await connection.getAccountInfo(ruleSetPda);
  const deserializedRuleSet = getLatestRuleSet(rawRuleSetPdaAccount?.data) as RuleSetV1;
  t.deepEqual(deserializedRuleSet, ruleSet);
});
