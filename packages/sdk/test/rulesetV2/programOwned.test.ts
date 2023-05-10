import { Keypair } from '@solana/web3.js';
import test from 'ava';
import { deserializeRuleV2, programOwnedV2, serializeRuleV2 } from '../../src';
import { serializeString32 } from '../../src/revisions/v2/helpers';

test('serialize', async (t) => {
  const program = Keypair.generate().publicKey;
  const rule = programOwnedV2('myAccount', program);
  const serializedRule = serializeRuleV2(rule).toString('hex');
  t.is(
    serializedRule,
    '0b000000' + // Rule type
      '40000000' + // Rule length
      program.toBuffer().toString('hex') + // PublicKey
      serializeString32('myAccount').toString('hex'), // Field
  );
});

test('deserialize', async (t) => {
  const program = Keypair.generate().publicKey;
  const hexBuffer =
    '0b000000' + // Rule type
    '40000000' + // Rule length
    program.toBuffer().toString('hex') + // PublicKey
    serializeString32('myAccount').toString('hex'); // Field
  const buffer = Buffer.from(hexBuffer, 'hex');
  const rule = deserializeRuleV2(buffer);
  t.deepEqual(rule, programOwnedV2('myAccount', program));
});
