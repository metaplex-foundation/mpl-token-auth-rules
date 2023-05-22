import type { RuleV2 } from '../v2';
import { AdditionalSignerRuleV1 } from './additionalSigner';
import { AllRuleV1 } from './all';
import { AmountRuleV1 } from './amount';
import { AnyRuleV1 } from './any';
import { NamespaceRuleV1 } from './namespace';
import { NotRuleV1 } from './not';
import { PassRuleV1 } from './pass';
import { PdaMatchRuleV1 } from './pdaMatch';
import { ProgramOwnedRuleV1 } from './programOwned';
import { ProgramOwnedListRuleV1 } from './programOwnedList';
import { ProgramOwnedTreeRuleV1 } from './programOwnedTree';
import { PubkeyListMatchRuleV1 } from './pubkeyListMatch';
import { PubkeyMatchRuleV1 } from './pubkeyMatch';
import { PubkeyTreeMatchRuleV1 } from './pubkeyTreeMatch';

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

export const isRuleV1 = (rule: RuleV2 | RuleV1): rule is RuleV1 =>
  (typeof rule === 'string' && (rule === 'Namespace' || rule === 'Pass')) ||
  !('type' in (rule as object));
