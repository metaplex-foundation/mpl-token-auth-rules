/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
import { ValidateArgs, validateArgsBeet } from '../types/ValidateArgs';

/**
 * @category Instructions
 * @category Validate
 * @category generated
 */
export type ValidateInstructionArgs = {
  validateArgs: ValidateArgs;
};
/**
 * @category Instructions
 * @category Validate
 * @category generated
 */
export const ValidateStruct = new beet.FixableBeetArgsStruct<
  ValidateInstructionArgs & {
    instructionDiscriminator: number;
  }
>(
  [
    ['instructionDiscriminator', beet.u8],
    ['validateArgs', validateArgsBeet],
  ],
  'ValidateInstructionArgs',
);
/**
 * Accounts required by the _Validate_ instruction
 *
 * @property [_writable_, **signer**] payer Payer and creator of the rule set
 * @property [_writable_] ruleset The PDA account where the ruleset is stored
 * @property [**signer**] optRuleSigner1 (optional) Optional rule validation signer 1
 * @property [**signer**] optRuleSigner2 (optional) Optional rule validation signer 2
 * @property [**signer**] optRuleSigner3 (optional) Optional rule validation signer 3
 * @property [**signer**] optRuleSigner4 (optional) Optional rule validation signer 4
 * @property [**signer**] optRuleSigner5 (optional) Optional rule validation signer 5
 * @property [] optRuleNonsigner1 (optional) Optional rule validation non-signer 1
 * @property [] optRuleNonsigner2 (optional) Optional rule validation non-signer 2
 * @property [] optRuleNonsigner3 (optional) Optional rule validation non-signer 3
 * @property [] optRuleNonsigner4 (optional) Optional rule validation non-signer 4
 * @property [] optRuleNonsigner5 (optional) Optional rule validation non-signer 5
 * @category Instructions
 * @category Validate
 * @category generated
 */
export type ValidateInstructionAccounts = {
  payer: web3.PublicKey;
  ruleset: web3.PublicKey;
  systemProgram?: web3.PublicKey;
  optRuleSigner1?: web3.PublicKey;
  optRuleSigner2?: web3.PublicKey;
  optRuleSigner3?: web3.PublicKey;
  optRuleSigner4?: web3.PublicKey;
  optRuleSigner5?: web3.PublicKey;
  optRuleNonsigner1?: web3.PublicKey;
  optRuleNonsigner2?: web3.PublicKey;
  optRuleNonsigner3?: web3.PublicKey;
  optRuleNonsigner4?: web3.PublicKey;
  optRuleNonsigner5?: web3.PublicKey;
};

export const validateInstructionDiscriminator = 1;

/**
 * Creates a _Validate_ instruction.
 *
 * Optional accounts that are not provided will be omitted from the accounts
 * array passed with the instruction.
 * An optional account that is set cannot follow an optional account that is unset.
 * Otherwise an Error is raised.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category Validate
 * @category generated
 */
export function createValidateInstruction(
  accounts: ValidateInstructionAccounts,
  args: ValidateInstructionArgs,
  programId = new web3.PublicKey('auth9SigNpDKz4sJJ1DfCTuZrZNSAgh9sFD3rboVmgg'),
) {
  const [data] = ValidateStruct.serialize({
    instructionDiscriminator: validateInstructionDiscriminator,
    ...args,
  });
  const keys: web3.AccountMeta[] = [
    {
      pubkey: accounts.payer,
      isWritable: true,
      isSigner: true,
    },
    {
      pubkey: accounts.ruleset,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.systemProgram ?? web3.SystemProgram.programId,
      isWritable: false,
      isSigner: false,
    },
  ];

  if (accounts.optRuleSigner1 != null) {
    keys.push({
      pubkey: accounts.optRuleSigner1,
      isWritable: false,
      isSigner: true,
    });
  }

  if (accounts.optRuleSigner2 != null) {
    if (accounts.optRuleSigner1 == null) {
      throw new Error(
        "When providing 'optRuleSigner2' then 'accounts.optRuleSigner1' need(s) to be provided as well.",
      );
    }
    keys.push({
      pubkey: accounts.optRuleSigner2,
      isWritable: false,
      isSigner: true,
    });
  }

  if (accounts.optRuleSigner3 != null) {
    if (accounts.optRuleSigner1 == null || accounts.optRuleSigner2 == null) {
      throw new Error(
        "When providing 'optRuleSigner3' then 'accounts.optRuleSigner1', 'accounts.optRuleSigner2' need(s) to be provided as well.",
      );
    }
    keys.push({
      pubkey: accounts.optRuleSigner3,
      isWritable: false,
      isSigner: true,
    });
  }

  if (accounts.optRuleSigner4 != null) {
    if (
      accounts.optRuleSigner1 == null ||
      accounts.optRuleSigner2 == null ||
      accounts.optRuleSigner3 == null
    ) {
      throw new Error(
        "When providing 'optRuleSigner4' then 'accounts.optRuleSigner1', 'accounts.optRuleSigner2', 'accounts.optRuleSigner3' need(s) to be provided as well.",
      );
    }
    keys.push({
      pubkey: accounts.optRuleSigner4,
      isWritable: false,
      isSigner: true,
    });
  }

  if (accounts.optRuleSigner5 != null) {
    if (
      accounts.optRuleSigner1 == null ||
      accounts.optRuleSigner2 == null ||
      accounts.optRuleSigner3 == null ||
      accounts.optRuleSigner4 == null
    ) {
      throw new Error(
        "When providing 'optRuleSigner5' then 'accounts.optRuleSigner1', 'accounts.optRuleSigner2', 'accounts.optRuleSigner3', 'accounts.optRuleSigner4' need(s) to be provided as well.",
      );
    }
    keys.push({
      pubkey: accounts.optRuleSigner5,
      isWritable: false,
      isSigner: true,
    });
  }

  if (accounts.optRuleNonsigner1 != null) {
    if (
      accounts.optRuleSigner1 == null ||
      accounts.optRuleSigner2 == null ||
      accounts.optRuleSigner3 == null ||
      accounts.optRuleSigner4 == null ||
      accounts.optRuleSigner5 == null
    ) {
      throw new Error(
        "When providing 'optRuleNonsigner1' then 'accounts.optRuleSigner1', 'accounts.optRuleSigner2', 'accounts.optRuleSigner3', 'accounts.optRuleSigner4', 'accounts.optRuleSigner5' need(s) to be provided as well.",
      );
    }
    keys.push({
      pubkey: accounts.optRuleNonsigner1,
      isWritable: false,
      isSigner: false,
    });
  }

  if (accounts.optRuleNonsigner2 != null) {
    if (
      accounts.optRuleSigner1 == null ||
      accounts.optRuleSigner2 == null ||
      accounts.optRuleSigner3 == null ||
      accounts.optRuleSigner4 == null ||
      accounts.optRuleSigner5 == null ||
      accounts.optRuleNonsigner1 == null
    ) {
      throw new Error(
        "When providing 'optRuleNonsigner2' then 'accounts.optRuleSigner1', 'accounts.optRuleSigner2', 'accounts.optRuleSigner3', 'accounts.optRuleSigner4', 'accounts.optRuleSigner5', 'accounts.optRuleNonsigner1' need(s) to be provided as well.",
      );
    }
    keys.push({
      pubkey: accounts.optRuleNonsigner2,
      isWritable: false,
      isSigner: false,
    });
  }

  if (accounts.optRuleNonsigner3 != null) {
    if (
      accounts.optRuleSigner1 == null ||
      accounts.optRuleSigner2 == null ||
      accounts.optRuleSigner3 == null ||
      accounts.optRuleSigner4 == null ||
      accounts.optRuleSigner5 == null ||
      accounts.optRuleNonsigner1 == null ||
      accounts.optRuleNonsigner2 == null
    ) {
      throw new Error(
        "When providing 'optRuleNonsigner3' then 'accounts.optRuleSigner1', 'accounts.optRuleSigner2', 'accounts.optRuleSigner3', 'accounts.optRuleSigner4', 'accounts.optRuleSigner5', 'accounts.optRuleNonsigner1', 'accounts.optRuleNonsigner2' need(s) to be provided as well.",
      );
    }
    keys.push({
      pubkey: accounts.optRuleNonsigner3,
      isWritable: false,
      isSigner: false,
    });
  }

  if (accounts.optRuleNonsigner4 != null) {
    if (
      accounts.optRuleSigner1 == null ||
      accounts.optRuleSigner2 == null ||
      accounts.optRuleSigner3 == null ||
      accounts.optRuleSigner4 == null ||
      accounts.optRuleSigner5 == null ||
      accounts.optRuleNonsigner1 == null ||
      accounts.optRuleNonsigner2 == null ||
      accounts.optRuleNonsigner3 == null
    ) {
      throw new Error(
        "When providing 'optRuleNonsigner4' then 'accounts.optRuleSigner1', 'accounts.optRuleSigner2', 'accounts.optRuleSigner3', 'accounts.optRuleSigner4', 'accounts.optRuleSigner5', 'accounts.optRuleNonsigner1', 'accounts.optRuleNonsigner2', 'accounts.optRuleNonsigner3' need(s) to be provided as well.",
      );
    }
    keys.push({
      pubkey: accounts.optRuleNonsigner4,
      isWritable: false,
      isSigner: false,
    });
  }

  if (accounts.optRuleNonsigner5 != null) {
    if (
      accounts.optRuleSigner1 == null ||
      accounts.optRuleSigner2 == null ||
      accounts.optRuleSigner3 == null ||
      accounts.optRuleSigner4 == null ||
      accounts.optRuleSigner5 == null ||
      accounts.optRuleNonsigner1 == null ||
      accounts.optRuleNonsigner2 == null ||
      accounts.optRuleNonsigner3 == null ||
      accounts.optRuleNonsigner4 == null
    ) {
      throw new Error(
        "When providing 'optRuleNonsigner5' then 'accounts.optRuleSigner1', 'accounts.optRuleSigner2', 'accounts.optRuleSigner3', 'accounts.optRuleSigner4', 'accounts.optRuleSigner5', 'accounts.optRuleNonsigner1', 'accounts.optRuleNonsigner2', 'accounts.optRuleNonsigner3', 'accounts.optRuleNonsigner4' need(s) to be provided as well.",
      );
    }
    keys.push({
      pubkey: accounts.optRuleNonsigner5,
      isWritable: false,
      isSigner: false,
    });
  }

  const ix = new web3.TransactionInstruction({
    programId,
    keys,
    data,
  });
  return ix;
}