/* eslint-disable prefer-template */
import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { RuleSetRevisionV1, isAdditionalSignerRuleV1 } from '../../../src';
import { createUmiSync } from '../../_setup';

test('isAdditionalSigner', async (t) => {
  const umi = createUmiSync();
  const owner = generateSigner(umi).publicKey;
  const publicKeyA = generateSigner(umi).publicKey;
  const revision: RuleSetRevisionV1 = {
    libVersion: 1,
    ruleSetName: 'My Rule Set',
    owner: [...owner.bytes],
    operations: {
      deposit: {
        AdditionalSigner: {
          account: [...publicKeyA.bytes],
        },
      },
    },
  };

  t.true(isAdditionalSignerRuleV1(revision.operations.deposit));
});
