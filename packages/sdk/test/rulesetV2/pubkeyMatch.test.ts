import { Keypair } from '@solana/web3.js';
import test from 'ava';
import {
  deserializeRuleV2,
  RuleTypeV2,
  serializeRuleV2,
  pubkeyMatchV2,
} from '../../src/mpl-token-auth-rules';

test('serialize', async (t) => {
  const publicKey = Keypair.generate().publicKey;
  const field = 'test';
  const rule = pubkeyMatchV2(publicKey, field);
  const serializedRule = serializeRuleV2(rule).toString('hex');
  t.is(
    serializedRule,
    '0f000000' + // Rule type
      '40000000' + // Rule length
      publicKey.toBuffer().toString('hex') + // PublicKey
      Buffer.from(field.padEnd(32, '\0')).toString('hex'), // Field
  );
});

test('deserialize', async (t) => {
  const publicKey = Keypair.generate().publicKey;
  const field = 'test';
  const hexBuffer =
    '0f000000' + // Rule type
    '40000000' + // Rule length
    publicKey.toBuffer().toString('hex') + // PublicKey
    Buffer.from(field.padEnd(32, '\0')).toString('hex'); // Field
  const buffer = Buffer.from(hexBuffer, 'hex');
  const rule = deserializeRuleV2(buffer);
  t.deepEqual(rule, {
    type: RuleTypeV2.PubkeyMatch,
    publicKey,
    field,
  });
});
