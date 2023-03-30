import { Keypair } from '@solana/web3.js';
import test from 'ava';
import {
  additionalSignerV2,
  deserializeRuleSetV2,
  RuleSetV2,
  serializeRuleSetV2,
} from '../../src/mpl-token-auth-rules';
import { serializeString32 } from '../../src/ruleSetV2/helpers';

test('serialize', async (t) => {
  const owner = Keypair.generate().publicKey;
  const publicKeyA = Keypair.generate().publicKey;
  const publicKeyB = Keypair.generate().publicKey;
  const ruleSet: RuleSetV2 = {
    name: 'My Rule Set',
    owner,
    operations: {
      deposit: additionalSignerV2(publicKeyA),
      withdraw: additionalSignerV2(publicKeyB),
    },
  };
  const serializedRuleSet = serializeRuleSetV2(ruleSet).toString('hex');

  const expectedRuleA = '0100000020000000' + publicKeyA.toBuffer().toString('hex');
  const expectedRuleB = '0100000020000000' + publicKeyB.toBuffer().toString('hex');
  t.is(
    serializedRuleSet,
    '02000000' + // Rule Set Version
      '02000000' + // Number of operations/rules
      owner.toBuffer().toString('hex') + // Owner
      serializeString32('My Rule Set').toString('hex') + // Name
      serializeString32('deposit').toString('hex') + // Deposit operation
      serializeString32('withdraw').toString('hex') + // Withdraw operation
      expectedRuleA +
      expectedRuleB,
  );
});

test('deserialize', async (t) => {
  const owner = Keypair.generate().publicKey;
  const publicKeyA = Keypair.generate().publicKey;
  const publicKeyB = Keypair.generate().publicKey;
  const ruleA = '0100000020000000' + publicKeyA.toBuffer().toString('hex');
  const ruleB = '0100000020000000' + publicKeyB.toBuffer().toString('hex');
  const hexBuffer =
    '02000000' + // Rule Set Version
    '02000000' + // Number of operations/rules
    owner.toBuffer().toString('hex') + // Owner
    serializeString32('My Rule Set').toString('hex') + // Name
    serializeString32('deposit').toString('hex') + // Deposit operation
    serializeString32('withdraw').toString('hex') + // Withdraw operation
    ruleA +
    ruleB;
  const buffer = Buffer.from(hexBuffer, 'hex');
  const ruleSet = deserializeRuleSetV2(buffer);
  t.deepEqual(ruleSet, {
    name: 'My Rule Set',
    owner,
    operations: {
      deposit: additionalSignerV2(publicKeyA),
      withdraw: additionalSignerV2(publicKeyB),
    },
  });
});
