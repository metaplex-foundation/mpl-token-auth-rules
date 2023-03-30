import test from 'ava';
import {
  deserializeRuleV2,
  isWalletV2,
  RuleTypeV2,
  serializeRuleV2,
} from '../../src/mpl-token-auth-rules';
import { serializeString32 } from '../../src/ruleSetV2/helpers';

test('serialize', async (t) => {
  const rule = isWalletV2('myField');
  const serializedRule = serializeRuleV2(rule).toString('hex');
  t.is(
    serializedRule,
    '06000000' + // Rule type (6)
      '20000000' + // Rule length (32 bytes)
      serializeString32('myField').toString('hex'), // Field
  );
});

test('deserialize', async (t) => {
  const hexBuffer =
    '06000000' + // Rule type (6)
    '20000000' + // Rule length (32 bytes)
    serializeString32('myField').toString('hex'); // Field
  const buffer = Buffer.from(hexBuffer, 'hex');
  const rule = deserializeRuleV2(buffer);
  t.deepEqual(rule, {
    type: RuleTypeV2.IsWallet,
    field: 'myField',
  });
});
