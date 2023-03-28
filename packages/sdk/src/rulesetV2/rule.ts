import * as beet from '@metaplex-foundation/beet';
import {
  AdditionalSignerRule,
  deserializeAdditionalSigner,
  serializeAdditionalSigner,
} from './additionalSigner';
import { AllRule, deserializeAll, serializeAll } from './all';
import { AnyRule } from './any';
import { RuleType } from './ruleType';

export type Rule =
  | AdditionalSignerRule
  | AllRule
  // | AmountRule
  | AnyRule;
// | FrequencyRule
// | IsWalletRule
// | NamespaceRule
// | NotRule
// | PassRule
// | PDAMatchRule
// | ProgramOwnedListRule
// | ProgramOwnedTreeRule
// | ProgramOwnedRule
// | PubkeyListMatchRule
// | PubkeyMatchRule
// | PubkeyTreeMatchRule;

export const serializeRule = (rule: Rule): Buffer => {
  switch (rule.type) {
    case RuleType.AdditionalSigner:
      return serializeAdditionalSigner(rule);
    case RuleType.All:
      return serializeAll(rule);
    default:
      throw new Error('Unknown rule type: ' + rule.type);
  }
};

export const deserializeRule = (buffer: Buffer, offset = 0): Rule => {
  const type = beet.u32.read(buffer, offset) as RuleType;
  switch (type) {
    case RuleType.AdditionalSigner:
      return deserializeAdditionalSigner(buffer);
    case RuleType.All:
      return deserializeAll(buffer);
    default:
      throw new Error('Unknown rule type: ' + type);
  }
};

export const serializeRuleHeader = (ruleType: RuleType, length: number): Buffer => {
  const buffer = Buffer.alloc(8);
  beet.u32.write(buffer, 0, ruleType);
  beet.u32.write(buffer, 4, length);
  return buffer;
};
