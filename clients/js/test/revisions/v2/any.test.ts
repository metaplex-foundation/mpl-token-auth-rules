/* eslint-disable prefer-template */
import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { additionalSignerV2, anyV2 } from '../../../src';
import {
  createUmiSync,
  deserializeRuleV2FromHex,
  serializeRuleV2AsHex,
  toHex,
} from '../../_setup';

test('serialize', async (t) => {
  const umi = createUmiSync();
  const publicKeyA = generateSigner(umi).publicKey;
  const publicKeyB = generateSigner(umi).publicKey;
  const rule = anyV2([
    additionalSignerV2(publicKeyA),
    additionalSignerV2(publicKeyB),
  ]);
  const serializedRule = serializeRuleV2AsHex(umi, rule);

  const expectedRuleA = `0100000020000000${toHex(publicKeyA)}`;
  const expectedRuleB = `0100000020000000${toHex(publicKeyB)}`;
  t.is(
    serializedRule,
    '04000000' + // Rule type
      '58000000' + // Rule length
      '0200000000000000' + // Number of rules
      expectedRuleA +
      expectedRuleB
  );
});

test('deserialize', async (t) => {
  const umi = createUmiSync();
  const publicKeyA = generateSigner(umi).publicKey;
  const publicKeyB = generateSigner(umi).publicKey;
  const ruleA = `0100000020000000${toHex(publicKeyA)}`;
  const ruleB = `0100000020000000${toHex(publicKeyB)}`;
  const buffer =
    '04000000' + // Rule type
    '58000000' + // Rule length
    '0200000000000000' + // Number of rules
    ruleA +
    ruleB;
  const rule = deserializeRuleV2FromHex(umi, buffer);
  t.deepEqual(
    rule,
    anyV2([additionalSignerV2(publicKeyA), additionalSignerV2(publicKeyB)])
  );
});
