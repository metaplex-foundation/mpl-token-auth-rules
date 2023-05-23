/* eslint-disable prefer-template */
import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { RuleSetRevisionV1, isPassRuleV1 } from '../../../src';
import { createUmiSync } from '../../_setup';

test('isPassRuleV1', async (t) => {
  const umi = createUmiSync();
  const owner = generateSigner(umi).publicKey;

  const revision: RuleSetRevisionV1 = {
    libVersion: 1,
    ruleSetName: 'My Rule Set',
    owner: [...owner.bytes],
    operations: {
      deposit: 'Pass',
    },
  };

  t.true(isPassRuleV1(revision.operations.deposit));
});
