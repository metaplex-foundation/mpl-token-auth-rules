import { Keypair } from '@solana/web3.js';
import test from 'ava';
import {
  additionalSignerV2,
  deserializeRuleSetV2,
  RuleSetV2,
  serializeRuleSetV2,
} from '../../src/mpl-token-auth-rules';

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

  const expectedName = Buffer.alloc(32);
  expectedName.write('My Rule Set');
  const expectedDepositOperation = Buffer.alloc(32);
  expectedDepositOperation.write('deposit');
  const expectedWithdrawOperation = Buffer.alloc(32);
  expectedWithdrawOperation.write('withdraw');
  const expectedRuleA = '0100000020000000' + publicKeyA.toBuffer().toString('hex');
  const expectedRuleB = '0100000020000000' + publicKeyB.toBuffer().toString('hex');
  t.is(
    serializedRuleSet,
    '02000000' + // Rule Set Version
      '02000000' + // Number of operations/rules
      owner.toBuffer().toString('hex') + // Owner
      expectedName.toString('hex') + // Name
      expectedDepositOperation.toString('hex') + // Deposit operation
      expectedWithdrawOperation.toString('hex') + // Withdraw operation
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
  const name = Buffer.alloc(32);
  name.write('My Rule Set');
  const depositOperation = Buffer.alloc(32);
  depositOperation.write('deposit');
  const withdrawOperation = Buffer.alloc(32);
  withdrawOperation.write('withdraw');
  const hexBuffer =
    '02000000' + // Rule Set Version
    '02000000' + // Number of operations/rules
    owner.toBuffer().toString('hex') + // Owner
    name.toString('hex') + // Name
    depositOperation.toString('hex') + // Deposit operation
    withdrawOperation.toString('hex') + // Withdraw operation
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
