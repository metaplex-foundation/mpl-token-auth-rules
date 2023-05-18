import { RuleV2 } from './rule';

export enum RuleTypeV2 {
  Uninitialized, // 0
  AdditionalSigner, // 1
  All, // 2
  Amount, // 3
  Any, // 4
  Frequency, // 5
  IsWallet, // 6
  Namespace, // 7
  Not, // 8
  Pass, // 9
  PdaMatch, // 10
  ProgramOwned, // 11
  ProgramOwnedList, // 12
  ProgramOwnedTree, // 13
  PubkeyListMatch, // 14
  PubkeyMatch, // 15
  PubkeyTreeMatch, // 16
}

export const getRuleTypeV2AsString = (type: RuleTypeV2): RuleV2['type'] => {
  switch (type) {
    case RuleTypeV2.AdditionalSigner: // 1
      return 'AdditionalSigner';
    case RuleTypeV2.All: // 2
      return 'All';
    case RuleTypeV2.Amount: // 3
      return 'Amount';
    case RuleTypeV2.Any: // 4
      return 'Any';
    case RuleTypeV2.Frequency: // 5
      // return 'Frequency';
      throw new Error('Frequency Rule is not supported yet');
    case RuleTypeV2.IsWallet: // 6
      // return 'IsWallet';
      throw new Error('IsWallet Rule is not supported yet');
    case RuleTypeV2.Namespace: // 7
      return 'Namespace';
    case RuleTypeV2.Not: // 8
      return 'Not';
    case RuleTypeV2.Pass: // 9
      return 'Pass';
    case RuleTypeV2.PdaMatch: // 10
      return 'PdaMatch';
    case RuleTypeV2.ProgramOwned: // 11
      return 'ProgramOwned';
    case RuleTypeV2.ProgramOwnedList: // 12
      return 'ProgramOwnedList';
    case RuleTypeV2.ProgramOwnedTree: // 13
      return 'ProgramOwnedTree';
    case RuleTypeV2.PubkeyListMatch: // 14
      return 'PubkeyListMatch';
    case RuleTypeV2.PubkeyMatch: // 15
      return 'PubkeyMatch';
    case RuleTypeV2.PubkeyTreeMatch: // 16
      return 'PubkeyTreeMatch';
    case RuleTypeV2.Uninitialized: // 0
    default:
      throw new Error(`Unknown rule type: ${type}`);
  }
};
