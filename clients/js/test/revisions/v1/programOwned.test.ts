/* eslint-disable prefer-template */
import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { RuleSetRevisionV1, isProgramOwnedRuleV1 } from '../../../src';
import { createUmiSync } from '../../_setup';

test('isProgramOwnedV1', async (t) => {
  const umi = createUmiSync();
  const owner = generateSigner(umi).publicKey;
  const publicKeyA = generateSigner(umi).publicKey;

  const revision: RuleSetRevisionV1 = {
    libVersion: 1,
    ruleSetName: 'My Rule Set',
    owner: [...owner.bytes],
    operations: {
      deposit: {
        ProgramOwned: {
          field: 'myField',
          program: [...publicKeyA.bytes],
        },
      },
    },
  };

  t.is(isProgramOwnedRuleV1(revision.operations.deposit), true);
});
