import {
  PublicKey,
  PublicKeyInput,
  publicKey as toPublicKey,
} from '@metaplex-foundation/umi';
import {
  Serializer,
  publicKey as publicKeySerializer,
  string,
  struct,
} from '@metaplex-foundation/umi/serializers';
import type { RuleV1 } from '../v1';
import { RuleV2, isRuleV2 } from './rule';
import { wrapSerializerInRuleHeaderV2 } from './ruleHeader';
import { RuleTypeV2 } from './ruleType';

export type ProgramOwnedRuleV2 = {
  type: 'ProgramOwned';
  field: string;
  program: PublicKey;
};

export const programOwnedV2 = (
  field: string,
  program: PublicKeyInput
): ProgramOwnedRuleV2 => ({
  type: 'ProgramOwned',
  program: toPublicKey(program),
  field,
});

export const getProgramOwnedRuleV2Serializer =
  (): Serializer<ProgramOwnedRuleV2> =>
    wrapSerializerInRuleHeaderV2(
      RuleTypeV2.ProgramOwned,
      struct([
        ['program', publicKeySerializer()],
        ['field', string({ size: 32 })],
      ])
    );

export const isProgramOwnedRuleV2 = (
  rule: RuleV1 | RuleV2
): rule is ProgramOwnedRuleV2 => isRuleV2(rule) && rule.type === 'ProgramOwned';
