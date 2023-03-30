import { Keypair, PublicKey } from '@solana/web3.js';
import test from 'ava';
import {
  deserializeRuleV2,
  RuleTypeV2,
  serializeRuleV2,
  programOwnedListV2,
} from '../../src/mpl-token-auth-rules';
import { serializeString32 } from '../../src/ruleSetV2/helpers';

test('serialize', async (t) => {
  const programA = Keypair.generate().publicKey;
  const programB = Keypair.generate().publicKey;
  const programs: PublicKey[] = [programA, programB];
  const rule = programOwnedListV2('myTestField', programs);
  const serializedRule = serializeRuleV2(rule).toString('hex');
  t.is(
    serializedRule,
    '0c000000' + // Rule type
      '60000000' + // Rule length
      serializeString32('myTestField').toString('hex') + // Field
      programA.toBuffer().toString('hex') + // Program A
      programB.toBuffer().toString('hex'), // Program B
  );
});

test('deserialize', async (t) => {
  const programA = Keypair.generate().publicKey;
  const programB = Keypair.generate().publicKey;
  const programs: PublicKey[] = [programA, programB];
  const hexBuffer =
    '0c000000' + // Rule type
    '60000000' + // Rule length
    serializeString32('myTestField').toString('hex') + // Field
    programA.toBuffer().toString('hex') + // Program A
    programB.toBuffer().toString('hex'); // Program B
  const buffer = Buffer.from(hexBuffer, 'hex');
  const rule = deserializeRuleV2(buffer);
  t.deepEqual(rule, {
    type: RuleTypeV2.ProgramOwnedList,
    field: 'myTestField',
    programs,
  });
});
