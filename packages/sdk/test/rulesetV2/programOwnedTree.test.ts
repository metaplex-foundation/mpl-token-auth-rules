import test from 'ava';
import { deserializeRuleV2, programOwnedTreeV2, serializeRuleV2 } from '../../src';
import { serializeString32 } from '../../src/revisions/v2/helpers';

test('serialize', async (t) => {
  const root = new Uint8Array([...Array(32)].map(() => Math.floor(Math.random() * 40)));
  const rule = programOwnedTreeV2('myAccount', 'myProof', root);
  const serializedRule = serializeRuleV2(rule).toString('hex');
  t.is(
    serializedRule,
    '0d000000' + // Rule type
      '60000000' + // Rule length
      serializeString32('myAccount').toString('hex') + // pubkeyField
      serializeString32('myProof').toString('hex') + // proofField
      Buffer.from(root).toString('hex'), // root
  );
});

test('deserialize', async (t) => {
  const root = new Uint8Array([...Array(32)].map(() => Math.floor(Math.random() * 40)));
  const hexBuffer =
    '0d000000' + // Rule type
    '60000000' + // Rule length
    serializeString32('myAccount').toString('hex') + // pubkeyField
    serializeString32('myProof').toString('hex') + // proofField
    Buffer.from(root).toString('hex'); // root
  const buffer = Buffer.from(hexBuffer, 'hex');
  const rule = deserializeRuleV2(buffer);
  t.deepEqual(rule, programOwnedTreeV2('myAccount', 'myProof', root));
});
