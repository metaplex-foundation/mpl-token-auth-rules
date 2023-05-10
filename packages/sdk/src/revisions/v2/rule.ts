import * as beet from '@metaplex-foundation/beet';
import BN from 'bn.js';
import {
  AdditionalSignerRuleV2,
  deserializeAdditionalSignerV2,
  serializeAdditionalSignerV2,
} from './additionalSigner';
import { AllRuleV2, deserializeAllV2, serializeAllV2 } from './all';
import { AmountRuleV2, deserializeAmountV2, serializeAmountV2 } from './amount';
import { AnyRuleV2, deserializeAnyV2, serializeAnyV2 } from './any';
import { NamespaceRuleV2, deserializeNamespaceV2, serializeNamespaceV2 } from './namespace';
import { NotRuleV2, deserializeNotV2, serializeNotV2 } from './not';
import { PassRuleV2, deserializePassV2, serializePassV2 } from './pass';
import { PdaMatchRuleV2, deserializePdaMatchV2, serializePdaMatchV2 } from './pdaMatch';
import {
  ProgramOwnedRuleV2,
  deserializeProgramOwnedV2,
  serializeProgramOwnedV2,
} from './programOwned';
import {
  ProgramOwnedListRuleV2,
  deserializeProgramOwnedListV2,
  serializeProgramOwnedListV2,
} from './programOwnedList';
import {
  ProgramOwnedTreeRuleV2,
  deserializeProgramOwnedTreeV2,
  serializeProgramOwnedTreeV2,
} from './programOwnedTree';
import {
  PubkeyListMatchRuleV2,
  deserializePubkeyListMatchV2,
  serializePubkeyListMatchV2,
} from './pubkeyListMatch';
import { PubkeyMatchRuleV2, deserializePubkeyMatchV2, serializePubkeyMatchV2 } from './pubkeyMatch';
import {
  PubkeyTreeMatchRuleV2,
  deserializePubkeyTreeMatchV2,
  serializePubkeyTreeMatchV2,
} from './pubkeyTreeMatch';
import { RuleTypeV2 } from './ruleType';

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

export const serializeRuleV2 = (rule: RuleV2): Buffer => {
  const type = rule.type;
  switch (type) {
    case 'AdditionalSigner':
      return serializeAdditionalSignerV2(rule);
    case 'All':
      return serializeAllV2(rule);
    case 'Amount':
      return serializeAmountV2(rule);
    case 'Any':
      return serializeAnyV2(rule);
    // case 'Frequency':
    //   return serializeFrequencyV2(rule);
    // case 'IsWallet':
    //   return serializeIsWalletV2(rule);
    case 'Namespace':
      return serializeNamespaceV2(rule);
    case 'Not':
      return serializeNotV2(rule);
    case 'Pass':
      return serializePassV2(rule);
    case 'PdaMatch':
      return serializePdaMatchV2(rule);
    case 'ProgramOwned':
      return serializeProgramOwnedV2(rule);
    case 'ProgramOwnedList':
      return serializeProgramOwnedListV2(rule);
    case 'ProgramOwnedTree':
      return serializeProgramOwnedTreeV2(rule);
    case 'PubkeyListMatch':
      return serializePubkeyListMatchV2(rule);
    case 'PubkeyMatch':
      return serializePubkeyMatchV2(rule);
    case 'PubkeyTreeMatch':
      return serializePubkeyTreeMatchV2(rule);
    default:
      // Ensures all cases are handled.
      const neverType: never = type;
      throw new Error('Unknown rule type: ' + neverType);
  }
};

export const serializeRulesV2 = (rules: RuleV2[]): Buffer => {
  return Buffer.concat(rules.map(serializeRuleV2));
};

export const deserializeRuleV2 = (buffer: Buffer, offset = 0): RuleV2 => {
  const type = beet.u32.read(buffer, offset) as RuleTypeV2;
  switch (type) {
    case RuleTypeV2.AdditionalSigner:
      return deserializeAdditionalSignerV2(buffer, offset);
    case RuleTypeV2.All:
      return deserializeAllV2(buffer, offset);
    case RuleTypeV2.Amount:
      return deserializeAmountV2(buffer, offset);
    case RuleTypeV2.Any:
      return deserializeAnyV2(buffer, offset);
    // case RuleTypeV2.Frequency:
    //   return deserializeFrequencyV2(buffer, offset);
    // case RuleTypeV2.IsWallet:
    //   return deserializeIsWalletV2(buffer, offset);
    case RuleTypeV2.Namespace:
      return deserializeNamespaceV2(buffer, offset);
    case RuleTypeV2.Not:
      return deserializeNotV2(buffer, offset);
    case RuleTypeV2.Pass:
      return deserializePassV2(buffer, offset);
    case RuleTypeV2.PdaMatch:
      return deserializePdaMatchV2(buffer, offset);
    case RuleTypeV2.ProgramOwned:
      return deserializeProgramOwnedV2(buffer, offset);
    case RuleTypeV2.ProgramOwnedList:
      return deserializeProgramOwnedListV2(buffer, offset);
    case RuleTypeV2.ProgramOwnedTree:
      return deserializeProgramOwnedTreeV2(buffer, offset);
    case RuleTypeV2.PubkeyListMatch:
      return deserializePubkeyListMatchV2(buffer, offset);
    case RuleTypeV2.PubkeyMatch:
      return deserializePubkeyMatchV2(buffer, offset);
    case RuleTypeV2.PubkeyTreeMatch:
      return deserializePubkeyTreeMatchV2(buffer, offset);
    default:
      throw new Error('Unknown rule type: ' + type);
  }
};

export const deserializeRulesV2 = (buffer: Buffer, size: number | BN, offset = 0): RuleV2[] => {
  const rules: RuleV2[] = [];
  const sizeAsNumber = new BN(size).toNumber();

  for (let index = 0; index < sizeAsNumber; index++) {
    const length = beet.u32.read(buffer, offset + 4);
    rules.push(deserializeRuleV2(buffer, offset));
    offset += 8 + length;
  }

  return rules;
};

export const serializeRuleHeaderV2 = (ruleType: RuleTypeV2, length: number): Buffer => {
  const buffer = Buffer.alloc(8);
  beet.u32.write(buffer, 0, ruleType);
  beet.u32.write(buffer, 4, length);
  return buffer;
};
