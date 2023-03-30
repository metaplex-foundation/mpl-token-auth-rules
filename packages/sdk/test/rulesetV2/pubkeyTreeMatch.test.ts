import test from 'ava';
import {
  pubkeyTreeMatchV2,
  deserializeRuleV2,
  RuleTypeV2,
  serializeRuleV2,
} from '../../src/mpl-token-auth-rules';

test('serialize', async (t) => {
  const pubkeyField = 'publickKey';
  const proofField = 'proof';
  const root = new Uint8Array([...Array(32)].map((e) => Math.floor(Math.random() * 40)));
  const rule = pubkeyTreeMatchV2(pubkeyField, proofField, root);
  const serializedRule = serializeRuleV2(rule).toString('hex');
  t.is(
    serializedRule,
    '10000000' + // Rule type
      '60000000' + // Rule length
      Buffer.from(pubkeyField.padEnd(32, '\0')).toString('hex') + // pubkeyField
      Buffer.from(proofField.padEnd(32, '\0')).toString('hex') + // proofField
      Buffer.from(root).toString('hex'), // root
  );
});

test('deserialize', async (t) => {
  const pubkeyField = 'publickKey';
  const proofField = 'proof';
  const root = new Uint8Array([...Array(32)].map((e) => Math.floor(Math.random() * 40)));
  const hexBuffer =
    '10000000' + // Rule type
    '60000000' + // Rule length
    Buffer.from(pubkeyField.padEnd(32, '\0')).toString('hex') + // pubkeyField
    Buffer.from(proofField.padEnd(32, '\0')).toString('hex') + // proofField
    Buffer.from(root).toString('hex'); // root
  const buffer = Buffer.from(hexBuffer, 'hex');
  const rule = deserializeRuleV2(buffer);
  t.deepEqual(rule, {
    type: RuleTypeV2.PubkeyTreeMatch,
    pubkeyField,
    proofField,
    root,
  });
});
