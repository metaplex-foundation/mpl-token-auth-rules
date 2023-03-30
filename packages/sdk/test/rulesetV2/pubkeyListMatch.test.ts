import { Keypair, PublicKey } from '@solana/web3.js';
import test from 'ava';
import {
  deserializeRuleV2,
  RuleTypeV2,
  serializeRuleV2,
  pubkeyListMatchV2,
} from '../../src/mpl-token-auth-rules';

test('serialize', async (t) => {
  const publicKey = Keypair.generate().publicKey;
  const field = 'test';
  const publicKeys: PublicKey[] = [publicKey, publicKey, publicKey];
  const rule = pubkeyListMatchV2(field, publicKeys);
  const serializedRule = serializeRuleV2(rule).toString('hex');
  t.is(
    serializedRule,
    '0e000000' + // Rule type
      '80000000' + // Rule length
      Buffer.from(field.padEnd(32, '\0')).toString('hex') + // Field
      publicKey.toBuffer().toString('hex') + // PublicKey 1
      publicKey.toBuffer().toString('hex') + // PublicKey 2
      publicKey.toBuffer().toString('hex'), // PublicKey 3
  );
});

test('deserialize', async (t) => {
  const publicKey = Keypair.generate().publicKey;
  const field = 'test';
  const publicKeys: PublicKey[] = [publicKey, publicKey, publicKey];
  const hexBuffer =
    '0e000000' + // Rule type
    '80000000' + // Rule length
    Buffer.from(field.padEnd(32, '\0')).toString('hex') + // Field
    publicKey.toBuffer().toString('hex') + // PublicKey 1
    publicKey.toBuffer().toString('hex') + // PublicKey 2
    publicKey.toBuffer().toString('hex'); // PublicKey 3
  const buffer = Buffer.from(hexBuffer, 'hex');
  const rule = deserializeRuleV2(buffer);
  t.deepEqual(rule, {
    type: RuleTypeV2.PubkeyListMatch,
    field,
    publicKeys,
  });
});
