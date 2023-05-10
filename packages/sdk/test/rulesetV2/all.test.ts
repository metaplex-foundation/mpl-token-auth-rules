import { Keypair } from '@solana/web3.js';
import test from 'ava';
import {
  additionalSignerV2,
  AllRuleV2,
  allV2,
  deserializeRuleV2,
  serializeRuleV2,
} from '../../src';

test('serialize', async (t) => {
  const publicKeyA = Keypair.generate().publicKey;
  const publicKeyB = Keypair.generate().publicKey;
  const rule = allV2([additionalSignerV2(publicKeyA), additionalSignerV2(publicKeyB)]);
  const serializedRule = serializeRuleV2(rule).toString('hex');

  const expectedRuleA = '0100000020000000' + publicKeyA.toBuffer().toString('hex');
  const expectedRuleB = '0100000020000000' + publicKeyB.toBuffer().toString('hex');
  t.is(
    serializedRule,
    '02000000' + // Rule type
      '58000000' + // Rule length
      '0200000000000000' + // Number of rules
      expectedRuleA +
      expectedRuleB,
  );
});

test('deserialize', async (t) => {
  const publicKeyA = Keypair.generate().publicKey;
  const publicKeyB = Keypair.generate().publicKey;
  const ruleA = '0100000020000000' + publicKeyA.toBuffer().toString('hex');
  const ruleB = '0100000020000000' + publicKeyB.toBuffer().toString('hex');
  const hexBuffer =
    '02000000' + // Rule type
    '58000000' + // Rule length
    '0200000000000000' + // Number of rules
    ruleA +
    ruleB;
  const buffer = Buffer.from(hexBuffer, 'hex');
  const rule = deserializeRuleV2(buffer) as AllRuleV2;
  t.deepEqual(rule, allV2([additionalSignerV2(publicKeyA), additionalSignerV2(publicKeyB)]));
});
