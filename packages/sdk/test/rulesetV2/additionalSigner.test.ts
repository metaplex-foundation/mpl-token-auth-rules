import { Keypair } from '@solana/web3.js';
import test from 'ava';
import {
  additionalSigner,
  deserializeRule,
  RuleType,
  serializeRule,
} from '../../src/mpl-token-auth-rules';

test('serialize', async (t) => {
  const publicKey = Keypair.generate().publicKey;
  const rule = additionalSigner(publicKey);
  const serializedRule = serializeRule(rule).toString('hex');
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
  const rule = deserializeRule(buffer);
  t.deepEqual(rule, {
    type: RuleType.AdditionalSigner,
    publicKey,
  });
});
