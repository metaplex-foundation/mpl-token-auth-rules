/* eslint-disable prefer-template */
import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  AmountOperator,
  RuleSetRevisionV1,
  isAmountRuleV1,
} from '../../../src';
import { createUmiSync } from '../../_setup';

test('isAmountRuleV1', async (t) => {
  const umi = createUmiSync();
  const owner = generateSigner(umi).publicKey;

  const revision: RuleSetRevisionV1 = {
    libVersion: 1,
    ruleSetName: 'My Rule Set',
    owner: [...owner.bytes],
    operations: {
      deposit: {
        Amount: {
          amount: 100,
          operator: AmountOperator.Eq,
          field: 'amount',
        },
      },
    },
  };

  t.true(isAmountRuleV1(revision.operations.deposit));
});
