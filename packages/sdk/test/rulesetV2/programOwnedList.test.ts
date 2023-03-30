import { Keypair, PublicKey } from '@solana/web3.js';
import test from 'ava';
import {
  deserializeRuleV2,
  RuleTypeV2,
  serializeRuleV2,
  programOwnedListV2,
} from '../../src/mpl-token-auth-rules';

test('serialize', async (t) => {
  const program = Keypair.generate().publicKey;
  const field = 'test';
  const programs: PublicKey[] = [program, program];
  const rule = programOwnedListV2(field, programs);
  const serializedRule = serializeRuleV2(rule).toString('hex');
  t.is(
    serializedRule,
    '0c000000' + // Rule type
      '60000000' + // Rule length
      Buffer.from(field.padEnd(32, '\0')).toString('hex') + // Field
      program.toBuffer().toString('hex') + // Program 1
      program.toBuffer().toString('hex'), // Program 2
  );
});

test('deserialize', async (t) => {
  const program = Keypair.generate().publicKey;
  const field = 'test';
  const programs: PublicKey[] = [program, program];
  const hexBuffer =
    '0c000000' + // Rule type
    '60000000' + // Rule length
    Buffer.from(field.padEnd(32, '\0')).toString('hex') + // Field
    program.toBuffer().toString('hex') + // Program 1
    program.toBuffer().toString('hex'); // Program 2
  const buffer = Buffer.from(hexBuffer, 'hex');
  const rule = deserializeRuleV2(buffer);
  t.deepEqual(rule, {
    type: RuleTypeV2.ProgramOwnedList,
    field,
    programs,
  });
});
