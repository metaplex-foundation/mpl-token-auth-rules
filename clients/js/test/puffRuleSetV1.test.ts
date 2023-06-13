import { generateSigner, some } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  createOrUpdateV1,
  findRuleSetPda,
  programOwnedV2,
  puffRuleSetV1,
} from '../src';
import { createUmi, fetchRuleSetSize } from './_setup';

test('it can update the account size of a rule set by 10 000 bytes', async (t) => {
  // Given an existing rule set account.
  const umi = await createUmi();
  const owner = umi.identity;
  const name = 'transfer_test';
  const program = generateSigner(umi).publicKey;
  const ruleSetPda = findRuleSetPda(umi, { owner: owner.publicKey, name });
  await createOrUpdateV1(umi, {
    payer: owner,
    ruleSetPda,
    ruleSetRevision: some({
      libVersion: 2,
      name,
      owner: owner.publicKey,
      operations: { Transfer: programOwnedV2('Destination', program) },
    }),
  }).sendAndConfirm(umi);
  const initialSize = await fetchRuleSetSize(umi, ruleSetPda);

  // When we puff the rule set account.
  await puffRuleSetV1(umi, {
    payer: owner,
    ruleSetPda,
    ruleSetName: name,
  }).sendAndConfirm(umi);

  // Then we expect the rule set account to have increased in size by 10 000 bytes.
  const newSize = await fetchRuleSetSize(umi, ruleSetPda);
  t.is(newSize, initialSize + 10_000);
});
