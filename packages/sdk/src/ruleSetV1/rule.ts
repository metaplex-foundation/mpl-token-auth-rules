import { AmountOperator } from '../shared';
import { PublicKeyAsArrayOfBytes } from './publicKey';

export type RuleV1 =
  | AdditionalSignerRuleV1
  | AllRuleV1
  | AmountRuleV1
  | AnyRuleV1
  | NamespaceRuleV1
  | NotRuleV1
  | PassRuleV1
  | PdaMatchRuleV1
  | ProgramOwnedRuleV1
  | ProgramOwnedListRuleV1
  | ProgramOwnedTreeRuleV1
  | PubkeyListMatchRuleV1
  | PubkeyMatchRuleV1
  | PubkeyTreeMatchRuleV1;

export type AdditionalSignerRuleV1 = {
  AdditionalSigner: {
    account: PublicKeyAsArrayOfBytes;
  };
};

export type AllRuleV1 = {
  All: {
    rules: RuleV1[];
  };
};

export type AmountRuleV1 = {
  Amount: {
    amount: number;
    operator: AmountOperator;
    field: string;
  };
};

export type AnyRuleV1 = {
  Any: {
    rules: RuleV1[];
  };
};

export type NamespaceRuleV1 = 'Namespace';

export type NotRuleV1 = {
  Not: {
    rule: RuleV1;
  };
};

export type PassRuleV1 = 'Pass';

export type PdaMatchRuleV1 = {
  PDAMatch: {
    program: PublicKeyAsArrayOfBytes;
    pda_field: string;
    seeds_field: string;
  };
};

export type ProgramOwnedRuleV1 = {
  ProgramOwned: {
    program: PublicKeyAsArrayOfBytes;
    field: string;
  };
};

export type ProgramOwnedListRuleV1 = {
  ProgramOwnedList: {
    programs: PublicKeyAsArrayOfBytes[];
    field: string;
  };
};

export type ProgramOwnedTreeRuleV1 = {
  ProgramOwnedTree: {
    root: number[];
    pubkey_field: string;
    proof_field: string;
  };
};

export type PubkeyListMatchRuleV1 = {
  PubkeyListMatch: {
    pubkeys: PublicKeyAsArrayOfBytes[];
    field: string;
  };
};

export type PubkeyMatchRuleV1 = {
  PubkeyMatch: {
    pubkey: PublicKeyAsArrayOfBytes;
    field: string;
  };
};

export type PubkeyTreeMatchRuleV1 = {
  PubkeyTreeMatch: {
    root: number[];
    pubkey_field: string;
    proof_field: string;
  };
};
