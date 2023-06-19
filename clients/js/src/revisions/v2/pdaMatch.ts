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

export type PdaMatchRuleV2 = {
  type: 'PdaMatch';
  pdaField: string;
  program: PublicKey;
  seedsField: string;
};

export const pdaMatchV2 = (
  pdaField: string,
  program: PublicKeyInput,
  seedsField: string
): PdaMatchRuleV2 => ({
  type: 'PdaMatch',
  pdaField,
  program: toPublicKey(program),
  seedsField,
});

export const getPdaMatchRuleV2Serializer = (): Serializer<PdaMatchRuleV2> =>
  wrapSerializerInRuleHeaderV2(
    RuleTypeV2.PdaMatch,
    struct([
      ['program', publicKeySerializer()],
      ['pdaField', string({ size: 32 })],
      ['seedsField', string({ size: 32 })],
    ])
  );

export const isPdaMatchRuleV2 = (
  rule: RuleV1 | RuleV2
): rule is PdaMatchRuleV2 => isRuleV2(rule) && rule.type === 'PdaMatch';
