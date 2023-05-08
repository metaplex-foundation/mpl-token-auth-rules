import type { PublicKeyAsArrayOfBytes } from './publicKey';
import type { RuleV1 } from './rule';

export type RuleSetV1 = {
  /** The version of the ruleset. */
  version: 1;
  /** The name of the ruleset. */
  name: string;
  /** The owner of the ruleset as an array of 32 bytes. */
  owner: PublicKeyAsArrayOfBytes;
  /** The operations of the ruleset. */
  operations: Record<string, RuleV1>;
};
