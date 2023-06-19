/* eslint-disable prefer-template */
import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { additionalSignerV2, notV2 } from '../../../src';
import {
  createUmiSync,
  deserializeRuleV2FromHex,
  serializeRuleV2AsHex,
  toHex,
} from '../../_setup';

test('serialize', async (t) => {
  const umi = createUmiSync();
  const { publicKey } = generateSigner(umi);
  const rule = notV2(additionalSignerV2(publicKey));
  const serializedRule = serializeRuleV2AsHex(rule);

  const expectedChildRule = '0100000020000000' + toHex(publicKey);
  t.is(
    serializedRule,
    '08000000' + // Rule type (8)
      '28000000' + // Rule length (8 + 32)
      expectedChildRule
  );
});

test('deserialize', async (t) => {
  const umi = createUmiSync();
  const { publicKey } = generateSigner(umi);
  const childRule = '0100000020000000' + toHex(publicKey);
  const buffer =
    '08000000' + // Rule type (8)
    '28000000' + // Rule length (8 + 32)
    childRule;
  const rule = deserializeRuleV2FromHex(buffer);
  t.deepEqual(rule, notV2(additionalSignerV2(publicKey)));
});
