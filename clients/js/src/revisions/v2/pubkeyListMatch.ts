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

export type PubkeyListMatchRuleV2 = {
  type: 'PubkeyListMatch';
  field: string;
  publicKeys: PublicKey[];
};

export const pubkeyListMatchV2 = (
  field: string,
  publicKeys: PublicKeyInput[]
): PubkeyListMatchRuleV2 => ({
  type: 'PubkeyListMatch',
  field,
  publicKeys: publicKeys.map((program) => toPublicKey(program)),
});

export const getPubkeyListMatchRuleV2Serializer =
  (): Serializer<PubkeyListMatchRuleV2> =>
    wrapSerializerInRuleHeaderV2(
      RuleTypeV2.PubkeyListMatch,
      struct([
        ['field', string({ size: 32 })],
        ['publicKeys', array(publicKeySerializer(), { size: 'remainder' })],
      ])
    );

export const isPubkeyListMatchRuleV2 = (
  rule: RuleV1 | RuleV2
): rule is PubkeyListMatchRuleV2 =>
  isRuleV2(rule) && rule.type === 'PubkeyListMatch';
