import test from 'ava';
import {
  pubkeyTreeMatchV2,
  deserializeRuleV2,
  RuleTypeV2,
  serializeRuleV2,
} from '../../src/mpl-token-auth-rules';
import { serializeString32 } from '../../src/ruleSetV2/helpers';

test('serialize', async (t) => {
  const root = new Uint8Array([...Array(32)].map(() => Math.floor(Math.random() * 40)));
  const rule = pubkeyTreeMatchV2('myPublicKey', 'myProof', root);
  const serializedRule = serializeRuleV2(rule).toString('hex');
  t.is(
    serializedRule,
    '10000000' + // Rule type
      '60000000' + // Rule length
      serializeString32('myPublicKey').toString('hex') + // pubkeyField
      serializeString32('myProof').toString('hex') + // proofField
      Buffer.from(root).toString('hex'), // root
  );
});

test('deserialize', async (t) => {
  const root = new Uint8Array([...Array(32)].map(() => Math.floor(Math.random() * 40)));
  const hexBuffer =
    '10000000' + // Rule type
    '60000000' + // Rule length
    serializeString32('myPublicKey').toString('hex') + // pubkeyField
    serializeString32('myProof').toString('hex') + // proofField
    Buffer.from(root).toString('hex'); // root

  const buffer = Buffer.from(hexBuffer, 'hex');
  const rule = deserializeRuleV2(buffer);
  t.deepEqual(rule, {
    type: RuleTypeV2.PubkeyTreeMatch,
    pubkeyField: 'myPublicKey',
    proofField: 'myProof',
    root,
  });
});
