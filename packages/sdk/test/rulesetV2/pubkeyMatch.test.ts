import { Keypair } from '@solana/web3.js';
import test from 'ava';
import { deserializeRuleV2, pubkeyMatchV2, serializeRuleV2 } from '../../src';
import { serializeString32 } from '../../src/revisions/v2/helpers';

test('serialize', async (t) => {
  const publicKey = Keypair.generate().publicKey;
  const rule = pubkeyMatchV2('myAccount', publicKey);
  const serializedRule = serializeRuleV2(rule).toString('hex');
  t.is(
    serializedRule,
    '0f000000' + // Rule type
      '40000000' + // Rule length
      publicKey.toBuffer().toString('hex') + // PublicKey
      serializeString32('myAccount').toString('hex'), // Field
  );
});

test('deserialize', async (t) => {
  const publicKey = Keypair.generate().publicKey;
  const hexBuffer =
    '0f000000' + // Rule type
    '40000000' + // Rule length
    publicKey.toBuffer().toString('hex') + // PublicKey
    serializeString32('myAccount').toString('hex'); // Field
  const buffer = Buffer.from(hexBuffer, 'hex');
  const rule = deserializeRuleV2(buffer);
  t.deepEqual(rule, pubkeyMatchV2('myAccount', publicKey));
});
