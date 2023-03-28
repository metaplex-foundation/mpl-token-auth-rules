import { Keypair } from '@solana/web3.js';
import test from 'ava';
import {
  additionalSignerV2,
  getLatestRuleSet,
  RuleSetV2,
  serializeRuleSetV2,
} from '../src/mpl-token-auth-rules';
import { createOrUpdateRuleset, getConnectionAndPayer } from './_setup';

test('it can create a ruleset v2', async (t) => {
  // Given a serialized ruleset v2 account data.
  const { connection, payer } = await getConnectionAndPayer();
  const publicKeyA = Keypair.generate().publicKey;
  const publicKeyB = Keypair.generate().publicKey;
  const name = 'My Rule Set';
  const ruleSet: RuleSetV2 = {
    name,
    owner: payer.publicKey,
    operations: {
      deposit: additionalSignerV2(publicKeyA),
      withdraw: additionalSignerV2(publicKeyB),
    },
  };
  const serializedRuleSet = serializeRuleSetV2(ruleSet);

  // When we create a new ruleset account using it.
  const ruleSetPda = await createOrUpdateRuleset(connection, payer, name, serializedRuleSet);

  // Then we can deserialize the account data and get the same ruleset.
  const rawRuleSetPdaAccount = await connection.getAccountInfo(ruleSetPda);
  const deserializedRuleSet = getLatestRuleSet(rawRuleSetPdaAccount?.data) as RuleSetV2;
  t.deepEqual(deserializedRuleSet, ruleSet);
});
