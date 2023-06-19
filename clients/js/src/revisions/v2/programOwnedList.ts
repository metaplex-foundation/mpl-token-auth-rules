import {
  PublicKey,
  PublicKeyInput,
  publicKey as toPublicKey,
} from '@metaplex-foundation/umi';
import {
  Serializer,
  array,
  publicKey as publicKeySerializer,
  string,
  struct,
} from '@metaplex-foundation/umi/serializers';
import type { RuleV1 } from '../v1';
import { RuleV2, isRuleV2 } from './rule';
import { wrapSerializerInRuleHeaderV2 } from './ruleHeader';
import { RuleTypeV2 } from './ruleType';

export type ProgramOwnedListRuleV2 = {
  type: 'ProgramOwnedList';
  field: string;
  programs: PublicKey[];
};

export const programOwnedListV2 = (
  field: string,
  programs: PublicKeyInput[]
): ProgramOwnedListRuleV2 => ({
  type: 'ProgramOwnedList',
  field,
  programs: programs.map((program) => toPublicKey(program)),
});

export const getProgramOwnedListRuleV2Serializer =
  (): Serializer<ProgramOwnedListRuleV2> =>
    wrapSerializerInRuleHeaderV2(
      RuleTypeV2.ProgramOwnedList,
      struct([
        ['field', string({ size: 32 })],
        ['programs', array(publicKeySerializer(), { size: 'remainder' })],
      ])
    );

export const isProgramOwnedListRuleV2 = (
  rule: RuleV1 | RuleV2
): rule is ProgramOwnedListRuleV2 =>
  isRuleV2(rule) && rule.type === 'ProgramOwnedList';
