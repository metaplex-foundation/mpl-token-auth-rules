import { Keypair } from '@solana/web3.js';
import test from 'ava';
import { deserializeRuleV2, pdaMatchV2, serializeRuleV2 } from '../../src/mpl-token-auth-rules';
import { serializeString32 } from '../../src/ruleSetV2/helpers';

test('serialize', async (t) => {
  const program = Keypair.generate().publicKey;
  const rule = pdaMatchV2('myAccount', program, 'mySeeds');
  const serializedRule = serializeRuleV2(rule).toString('hex');
  t.is(
    serializedRule,
    '0a000000' + // Rule type (10)
      '60000000' + // Rule length (96 bytes)
      program.toBuffer().toString('hex') + // PublicKey
      serializeString32('myAccount').toString('hex') + // Pda Field
      serializeString32('mySeeds').toString('hex'), // Seeds Field
  );
});

test('deserialize', async (t) => {
  const program = Keypair.generate().publicKey;
  const hexBuffer =
    '0a000000' + // Rule type (10)
    '60000000' + // Rule length (96 bytes)
    program.toBuffer().toString('hex') + // PublicKey
    serializeString32('myAccount').toString('hex') + // Pda Field
    serializeString32('mySeeds').toString('hex'); // Seeds Field
  const buffer = Buffer.from(hexBuffer, 'hex');
  const rule = deserializeRuleV2(buffer);
  t.deepEqual(rule, pdaMatchV2('myAccount', program, 'mySeeds'));
});
