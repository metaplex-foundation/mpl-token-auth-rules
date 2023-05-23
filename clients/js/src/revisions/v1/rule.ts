import type { RuleV2 } from '../v2';
import type { AdditionalSignerRuleV1 } from './additionalSigner';
import type { AllRuleV1 } from './all';
import type { AmountRuleV1 } from './amount';
import type { AnyRuleV1 } from './any';
import type { NamespaceRuleV1 } from './namespace';
import type { NotRuleV1 } from './not';
import type { PassRuleV1 } from './pass';
import type { PdaMatchRuleV1 } from './pdaMatch';
import type { ProgramOwnedRuleV1 } from './programOwned';
import type { ProgramOwnedListRuleV1 } from './programOwnedList';
import type { ProgramOwnedTreeRuleV1 } from './programOwnedTree';
import type { PubkeyListMatchRuleV1 } from './pubkeyListMatch';
import type { PubkeyMatchRuleV1 } from './pubkeyMatch';
import type { PubkeyTreeMatchRuleV1 } from './pubkeyTreeMatch';

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
