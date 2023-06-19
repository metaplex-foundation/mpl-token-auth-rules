import { Serializer } from '@metaplex-foundation/umi/serializers';
import type { RuleV1 } from '../v1';
import {
  AdditionalSignerRuleV2,
  getAdditionalSignerRuleV2Serializer,
} from './additionalSigner';
import { AllRuleV2, getAllRuleV2Serializer } from './all';
import { AmountRuleV2, getAmountRuleV2Serializer } from './amount';
import { AnyRuleV2, getAnyRuleV2Serializer } from './any';
import { NamespaceRuleV2, getNamespaceRuleV2Serializer } from './namespace';
import { NotRuleV2, getNotRuleV2Serializer } from './not';
import { PassRuleV2, getPassRuleV2Serializer } from './pass';
import { PdaMatchRuleV2, getPdaMatchRuleV2Serializer } from './pdaMatch';
import {
  ProgramOwnedRuleV2,
  getProgramOwnedRuleV2Serializer,
} from './programOwned';
import {
  ProgramOwnedListRuleV2,
  getProgramOwnedListRuleV2Serializer,
} from './programOwnedList';
import {
  ProgramOwnedTreeRuleV2,
  getProgramOwnedTreeRuleV2Serializer,
} from './programOwnedTree';
import {
  PubkeyListMatchRuleV2,
  getPubkeyListMatchRuleV2Serializer,
} from './pubkeyListMatch';
import {
  PubkeyMatchRuleV2,
  getPubkeyMatchRuleV2Serializer,
} from './pubkeyMatch';
import {
  PubkeyTreeMatchRuleV2,
  getPubkeyTreeMatchRuleV2Serializer,
} from './pubkeyTreeMatch';
import { RuleTypeV2, getRuleTypeV2AsString } from './ruleType';

export type RuleV2 =
  | AdditionalSignerRuleV2
  | AllRuleV2
  | AmountRuleV2
  | AnyRuleV2
  // | FrequencyRuleV2
  // | IsWalletRuleV2
  | NamespaceRuleV2
  | NotRuleV2
  | PassRuleV2
  | PdaMatchRuleV2
  | ProgramOwnedRuleV2
  | ProgramOwnedListRuleV2
  | ProgramOwnedTreeRuleV2
  | PubkeyListMatchRuleV2
  | PubkeyMatchRuleV2
  | PubkeyTreeMatchRuleV2;

export const getRuleV2Serializer = (): Serializer<RuleV2> => ({
  description: 'RuleV2',
  fixedSize: null,
  maxSize: null,
  serialize: (rule: RuleV2) =>
    getRuleV2SerializerFromType(rule.type).serialize(rule),
  deserialize: (buffer, offset = 0) => {
    const type = buffer[offset] as RuleTypeV2;
    const typeAsString = getRuleTypeV2AsString(type);
    return getRuleV2SerializerFromType(typeAsString).deserialize(
      buffer,
      offset
    );
  },
});

export const getRuleV2SerializerFromType = <T extends RuleV2>(
  type: T['type']
): Serializer<T> =>
  ((): Serializer<any> => {
    switch (type) {
      case 'AdditionalSigner':
        return getAdditionalSignerRuleV2Serializer();
      case 'All':
        return getAllRuleV2Serializer();
      case 'Amount':
        return getAmountRuleV2Serializer();
      case 'Any':
        return getAnyRuleV2Serializer();
      // case 'Frequency':
      //   return getFrequencyRuleV2Serializer();
      // case 'IsWallet':
      //   return getIsWalletRuleV2Serializer();
      case 'Namespace':
        return getNamespaceRuleV2Serializer();
      case 'Not':
        return getNotRuleV2Serializer();
      case 'Pass':
        return getPassRuleV2Serializer();
      case 'PdaMatch':
        return getPdaMatchRuleV2Serializer();
      case 'ProgramOwned':
        return getProgramOwnedRuleV2Serializer();
      case 'ProgramOwnedList':
        return getProgramOwnedListRuleV2Serializer();
      case 'ProgramOwnedTree':
        return getProgramOwnedTreeRuleV2Serializer();
      case 'PubkeyListMatch':
        return getPubkeyListMatchRuleV2Serializer();
      case 'PubkeyMatch':
        return getPubkeyMatchRuleV2Serializer();
      case 'PubkeyTreeMatch':
        return getPubkeyTreeMatchRuleV2Serializer();
      default:
        throw new Error(`Unknown rule type: ${type}`);
    }
  })() as Serializer<T>;

export const isRuleV2 = (rule: RuleV1 | RuleV2): rule is RuleV2 =>
  'type' in (rule as object);
