import {
  Serializer,
  string,
  struct,
} from '@metaplex-foundation/umi/serializers';
import type { RuleV1 } from '../v1';
import { RuleV2, isRuleV2 } from './rule';
import { wrapSerializerInRuleHeaderV2 } from './ruleHeader';
import { RuleTypeV2 } from './ruleType';

export type IsWalletRuleV2 = {
  type: 'IsWallet';
  field: string;
};

export const isWalletV2 = (
  field: string,
): IsWalletRuleV2 => ({
  type: 'IsWallet',
  field,
});

export const getIsWalletRuleV2Serializer =
  (): Serializer<IsWalletRuleV2> =>
    wrapSerializerInRuleHeaderV2(
      RuleTypeV2.IsWallet,
      struct([
        ['field', string({ size: 32 })],
      ])
    );

export const isIsWalletRuleV2 = (
  rule: RuleV1 | RuleV2
): rule is IsWalletRuleV2 => isRuleV2(rule) && rule.type === 'IsWallet';
