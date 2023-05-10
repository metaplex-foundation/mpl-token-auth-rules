import { Keypair } from '@solana/web3.js';
import test from 'ava';
import {
  additionalSignerV2,
  deserializeRuleV2,
  serializeRuleV2,
} from '../../src/mpl-token-auth-rules';

test('serialize', async (t) => {
  const publicKey = Keypair.generate().publicKey;
  const rule = additionalSignerV2(publicKey);
  const serializedRule = serializeRuleV2(rule).toString('hex');
  t.is(
    serializedRule,
    '01000000' + // Rule type
      '20000000' + // Rule length
      publicKey.toBuffer().toString('hex'), // Rule version
  );
});

test('deserialize', async (t) => {
  const publicKey = Keypair.generate().publicKey;
  const hexBuffer =
    '01000000' + // Rule type
    '20000000' + // Rule length
    publicKey.toBuffer().toString('hex'); // Rule version
  const buffer = Buffer.from(hexBuffer, 'hex');
  const rule = deserializeRuleV2(buffer);
  t.deepEqual(rule, additionalSignerV2(publicKey));
});
