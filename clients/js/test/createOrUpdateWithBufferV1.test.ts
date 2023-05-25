import {
  base58PublicKey,
  generateSigner,
  publicKey,
} from '@metaplex-foundation/umi';
import test from 'ava';
import {
  AmountOperator,
  Key,
  MPL_TOKEN_AUTH_RULES_PROGRAM_ID,
  RuleSet,
  RuleSetRevisionV1,
  RuleSetRevisionV2,
  additionalSignerV2,
  allV2,
  amountV2,
  anyV2,
  createOrUpdateWithBufferV1,
  fetchRuleSet,
  findRuleSetPda,
  getRuleSetRevisionSerializer,
  namespaceV2,
  notV2,
  passV2,
  pdaMatchV2,
  programOwnedListV2,
  programOwnedTreeV2,
  programOwnedV2,
  pubkeyListMatchV2,
  pubkeyMatchV2,
  pubkeyTreeMatchV2,
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

test('it can create a new rule set V1 with all rule types using a buffer', async (t) => {
  // Given a large rule set revision V1 with all rule types.
  const umi = await createUmi();
  const owner = umi.identity;
  const name = 'transfer_test';
  const getRandomRoot = () =>
    [...Array(32)].map(() => Math.floor(Math.random() * 40));
  const revision: RuleSetRevisionV1 = {
    libVersion: 1,
    ruleSetName: name,
    owner: [...owner.publicKey.bytes],
    operations: {
      'Transfer:Holder': {
        Any: {
          rules: [
            {
              All: {
                rules: [
                  {
                    AdditionalSigner: {
                      account: [...generateSigner(umi).publicKey.bytes],
                    },
                  },
                  {
                    AdditionalSigner: {
                      account: [...generateSigner(umi).publicKey.bytes],
                    },
                  },
                ],
              },
            },
            {
              Not: {
                rule: {
                  Amount: {
                    amount: 1,
                    operator: AmountOperator.Eq,
                    field: 'Amount',
                  },
                },
              },
            },
            {
              PubkeyMatch: {
                pubkey: [...generateSigner(umi).publicKey.bytes],
                field: 'Destination',
              },
            },
            {
              ProgramOwnedList: {
                programs: [[...MPL_TOKEN_AUTH_RULES_PROGRAM_ID.bytes]],
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
                  {
                    AdditionalSigner: {
                      account: [...generateSigner(umi).publicKey.bytes],
                    },
                  },
                  {
                    AdditionalSigner: {
                      account: [...generateSigner(umi).publicKey.bytes],
                    },
                  },
                  'Namespace',
                ],
              },
            },
            {
              Not: {
                rule: {
                  ProgramOwned: {
                    program: [...MPL_TOKEN_AUTH_RULES_PROGRAM_ID.bytes],
                    field: 'Destination',
                  },
                },
              },
            },
            'Pass',
            {
              PubkeyTreeMatch: {
                root: getRandomRoot(),
                pubkey_field: 'Source',
                proof_field: 'Proof',
              },
            },
          ],
        },
      },
      'Transfer:Authority': {
        Any: {
          rules: [
            {
              PubkeyListMatch: {
                pubkeys: [[...generateSigner(umi).publicKey.bytes]],
                field: 'Destination',
              },
            },
            {
              PDAMatch: {
                program: [...MPL_TOKEN_AUTH_RULES_PROGRAM_ID.bytes],
                pda_field: 'Destination',
                seeds_field: 'Seed',
              },
            },
            {
              ProgramOwned: {
                program: [...MPL_TOKEN_AUTH_RULES_PROGRAM_ID.bytes],
                field: 'Source',
              },
            },
            {
              ProgramOwnedTree: {
                root: getRandomRoot(),
                pubkey_field: 'Source',
                proof_field: 'Proof',
              },
            },
          ],
        },
      },
    },
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

test('it can create a new rule set V2 with all rule types using a buffer', async (t) => {
  // Given a large rule set revision V2 with all rule types.
  const umi = await createUmi();
  const owner = umi.identity;
  const name = 'transfer_test';
  const getRandomRoot = () =>
    new Uint8Array([...Array(32)].map(() => Math.floor(Math.random() * 40)));
  const revision: RuleSetRevisionV2 = {
    libVersion: 2,
    name,
    owner: base58PublicKey(owner),
    operations: {
      'Transfer:Holder': anyV2([
        allV2([
          additionalSignerV2(generateSigner(umi).publicKey),
          additionalSignerV2(generateSigner(umi).publicKey),
        ]),
        notV2(amountV2('Amount', '=', 1)),
        pubkeyMatchV2('Destination', generateSigner(umi).publicKey),
        programOwnedListV2('Source', [MPL_TOKEN_AUTH_RULES_PROGRAM_ID]),
      ]),
      'Transfer:Delegate': anyV2([
        allV2([
          additionalSignerV2(generateSigner(umi).publicKey),
          additionalSignerV2(generateSigner(umi).publicKey),
          namespaceV2(),
        ]),
        notV2(programOwnedV2('Destination', MPL_TOKEN_AUTH_RULES_PROGRAM_ID)),
        passV2(),
        pubkeyTreeMatchV2('Source', 'Proof', getRandomRoot()),
      ]),
      'Transfer:Authority': anyV2([
        pubkeyListMatchV2('Destination', [generateSigner(umi).publicKey]),
        pdaMatchV2('Destination', MPL_TOKEN_AUTH_RULES_PROGRAM_ID, 'Seed'),
        programOwnedV2('Source', MPL_TOKEN_AUTH_RULES_PROGRAM_ID),
        programOwnedTreeV2('Source', 'Proof', getRandomRoot()),
      ]),
    },
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
});
