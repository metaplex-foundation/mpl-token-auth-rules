import {
  base58PublicKey,
  generateSigner,
  publicKey,
} from '@metaplex-foundation/umi';
import test from 'ava';
import {
  Key,
  RuleSet,
  RuleSetRevisionV1,
  RuleSetRevisionV2,
  anyV2,
  createOrUpdateWithBufferV1,
  fetchRuleSet,
  findRuleSetPda,
  getRuleSetRevisionSerializer,
  programOwnedV2,
} from '../src';
import { createUmi, fetchRuleSetSize } from './_setup';

test('it can create a new rule set V1 using a buffer', async (t) => {
  // Given a large rule set revision V1 with more than 50 rules.
  const umi = await createUmi();
  const owner = umi.identity;
  const name = 'transfer_test';
  const anyRules = Array.from({ length: 50 }, () => ({
    ProgramOwned: {
      program: [...generateSigner(umi).publicKey.bytes],
      field: 'Destination',
    },
  }));
  const revision: RuleSetRevisionV1 = {
    libVersion: 1,
    ruleSetName: name,
    owner: [...owner.publicKey.bytes],
    operations: { Transfer: { Any: { rules: anyRules } } },
  };

  // When we create a new rule set account using this data.
  const ruleSetPda = findRuleSetPda(umi, { owner: owner.publicKey, name });
  await createOrUpdateWithBufferV1(umi, {
    payer: owner,
    ruleSetName: name,
    ruleSetRevision: revision,
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

  // And the rule set account is exactly the size it should be.
  const ruleSetAccountSize = await fetchRuleSetSize(umi, ruleSetPda);
  const serializedRevisionSize =
    getRuleSetRevisionSerializer(umi).serialize(revision).length;
  const expectedRuleSetAccountSize =
    serializedRevisionSize + // Revision.
    1 + // Revision version (extra byte for V1).
    9 + // Header.
    13; // Revision Map.
  t.is(ruleSetAccountSize, expectedRuleSetAccountSize);
});

test('it can create a new rule set V2 using a buffer', async (t) => {
  // Given a large rule set revision V2 with more than 50 rules.
  const umi = await createUmi();
  const owner = umi.identity;
  const name = 'transfer_test';
  const anyRules = Array.from({ length: 50 }, () =>
    programOwnedV2('Destination', generateSigner(umi).publicKey)
  );
  const revision: RuleSetRevisionV2 = {
    libVersion: 2,
    name,
    owner: base58PublicKey(owner),
    operations: { Transfer: anyV2(anyRules) },
  };

  // When we create a new rule set account using this data.
  await createOrUpdateWithBufferV1(umi, {
    payer: owner,
    ruleSetName: name,
    ruleSetRevision: revision,
  }).sendAndConfirm(umi);

  // Then we expect the rule set account to exist and contain the rule set data.
  const ruleSetPda = findRuleSetPda(umi, { owner: owner.publicKey, name });
  const ruleSetAccount = await fetchRuleSet(umi, ruleSetPda);
  t.like(ruleSetAccount, <RuleSet>{
    publicKey: publicKey(ruleSetPda),
    key: Key.RuleSet,
    latestRevision: revision,
    revisions: [revision],
    revisionMap: { version: 1 },
  });

  // And the rule set account is exactly the size it should be.
  const ruleSetAccountSize = await fetchRuleSetSize(umi, ruleSetPda);
  const serializedRevisionSize =
    getRuleSetRevisionSerializer(umi).serialize(revision).length;
  const expectedRuleSetAccountSize =
    serializedRevisionSize + // Revision.
    9 + // Header.
    13 + // Revision Map.
    7; // Bytemuck padding (for alignment).
  t.is(ruleSetAccountSize, expectedRuleSetAccountSize);
});
