import { Keypair } from '@solana/web3.js';
import test from 'ava';
import {
  deserializeRuleV2,
  RuleTypeV2,
  serializeRuleV2,
  programOwnedV2,
} from '../../src/mpl-token-auth-rules';

test('serialize', async (t) => {
  const program = Keypair.generate().publicKey;
  const field = 'test';
  const rule = programOwnedV2(program, field);
  const serializedRule = serializeRuleV2(rule).toString('hex');
  t.is(
    serializedRule,
    '0b000000' + // Rule type
      '40000000' + // Rule length
      program.toBuffer().toString('hex') + // PublicKey
      Buffer.from(field.padEnd(32, '\0')).toString('hex'), // Field
  );
});

test('deserialize', async (t) => {
  const program = Keypair.generate().publicKey;
  const field = 'test';
  const hexBuffer =
    '0b000000' + // Rule type
    '40000000' + // Rule length
    program.toBuffer().toString('hex') + // PublicKey
    Buffer.from(field.padEnd(32, '\0')).toString('hex'); // Field
  const buffer = Buffer.from(hexBuffer, 'hex');
  const rule = deserializeRuleV2(buffer);
  t.deepEqual(rule, {
    type: RuleTypeV2.ProgramOwned,
    program,
    field,
  });
});
