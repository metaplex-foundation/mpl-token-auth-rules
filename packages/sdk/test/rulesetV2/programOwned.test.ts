import { Keypair } from '@solana/web3.js';
import test from 'ava';
import {
  deserializeRuleV2,
  RuleTypeV2,
  serializeRuleV2,
  programOwnedV2,
} from '../../src/mpl-token-auth-rules';
import { serializeString32 } from '../../src/ruleSetV2/helpers';

test('serialize', async (t) => {
  const program = Keypair.generate().publicKey;
  const rule = programOwnedV2(program, 'myTestField');
  const serializedRule = serializeRuleV2(rule).toString('hex');
  t.is(
    serializedRule,
    '0b000000' + // Rule type
      '40000000' + // Rule length
      program.toBuffer().toString('hex') + // PublicKey
      serializeString32('myTestField').toString('hex'), // Field
  );
});

test('deserialize', async (t) => {
  const program = Keypair.generate().publicKey;
  const hexBuffer =
    '0b000000' + // Rule type
    '40000000' + // Rule length
    program.toBuffer().toString('hex') + // PublicKey
    serializeString32('myTestField').toString('hex'); // Field
  const buffer = Buffer.from(hexBuffer, 'hex');
  const rule = deserializeRuleV2(buffer);
  t.deepEqual(rule, {
    type: RuleTypeV2.ProgramOwned,
    field: 'myTestField',
    program,
  });
});
