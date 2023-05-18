import {
  base58PublicKey,
  generateSigner,
  publicKey,
  some,
} from '@metaplex-foundation/umi';
import test from 'ava';
import {
  Key,
  RuleSet,
  RuleSetRevisionV1,
  RuleSetRevisionV2,
  createOrUpdateV1,
  fetchRuleSet,
  findRuleSetPda,
  programOwnedV2,
} from '../src';
import { createUmi } from './_setup';

test('it can create a new rule set V1', async (t) => {
  // Given a rule set revision V1.
  const umi = await createUmi();
  const owner = umi.identity;
  const program = generateSigner(umi).publicKey;
  const name = 'transfer_test';
  const revision: RuleSetRevisionV1 = {
    libVersion: 1,
    ruleSetName: name,
    owner: [...owner.publicKey.bytes],
    operations: {
      Transfer: {
        ProgramOwned: {
          program: [...program.bytes],
          field: 'Destination',
        },
      },
    },
  };

  // When we create a new rule set account using this data.
  const ruleSetPda = findRuleSetPda(umi, { owner: owner.publicKey, name });
  await createOrUpdateV1(umi, {
    payer: owner,
    ruleSetPda,
    ruleSetRevision: some(revision),
  }).sendAndConfirm(umi);

  // Then we expect the rule set account to exist and contain the rule set data.
  const ruleSetAccount = await fetchRuleSet(umi, ruleSetPda);
  t.like(ruleSetAccount, <RuleSet>{
    publicKey: publicKey(ruleSetPda),
    key: Key.RuleSet,
    latestRevision: revision,
    revisions: [revision],
    revisionMap: { version: 1 },
  });
});

test('it can create a new rule set V2', async (t) => {
  // Given a rule set revision V2.
  const umi = await createUmi();
  const owner = umi.identity;
  const program = generateSigner(umi).publicKey;
  const name = 'transfer_test';
  const revision: RuleSetRevisionV2 = {
    libVersion: 2,
    name,
    owner: base58PublicKey(owner),
    operations: {
      Transfer: programOwnedV2('Destination', program),
    },
  };

  // When we create a new rule set account using this data.
  const ruleSetPda = findRuleSetPda(umi, { owner: owner.publicKey, name });
  await createOrUpdateV1(umi, {
    payer: owner,
    ruleSetPda,
    ruleSetRevision: some(revision),
  }).sendAndConfirm(umi);

  // Then we expect the rule set account to exist and contain the rule set data.
  const ruleSetAccount = await fetchRuleSet(umi, ruleSetPda);
  t.like(ruleSetAccount, <RuleSet>{
    publicKey: publicKey(ruleSetPda),
    key: Key.RuleSet,
    latestRevision: revision,
    revisions: [revision],
    revisionMap: { version: 1 },
  });
});
