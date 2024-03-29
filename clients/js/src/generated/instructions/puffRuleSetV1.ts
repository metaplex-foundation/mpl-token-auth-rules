/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/metaplex-foundation/kinobi
 */

import {
  Context,
  Pda,
  PublicKey,
  Signer,
  TransactionBuilder,
  transactionBuilder,
} from '@metaplex-foundation/umi';
import {
  Serializer,
  mapSerializer,
  string,
  struct,
  u8,
} from '@metaplex-foundation/umi/serializers';
import {
  ResolvedAccount,
  ResolvedAccountsWithIndices,
  getAccountMetasAndSigners,
} from '../shared';

// Accounts.
export type PuffRuleSetV1InstructionAccounts = {
  /** Payer and creator of the RuleSet */
  payer?: Signer;
  /** The PDA account where the RuleSet is stored */
  ruleSetPda: PublicKey | Pda;
  /** System program */
  systemProgram?: PublicKey | Pda;
};

// Data.
export type PuffRuleSetV1InstructionData = {
  discriminator: number;
  puffRuleSetV1Discriminator: number;
  ruleSetName: string;
};

export type PuffRuleSetV1InstructionDataArgs = { ruleSetName: string };

export function getPuffRuleSetV1InstructionDataSerializer(): Serializer<
  PuffRuleSetV1InstructionDataArgs,
  PuffRuleSetV1InstructionData
> {
  return mapSerializer<
    PuffRuleSetV1InstructionDataArgs,
    any,
    PuffRuleSetV1InstructionData
  >(
    struct<PuffRuleSetV1InstructionData>(
      [
        ['discriminator', u8()],
        ['puffRuleSetV1Discriminator', u8()],
        ['ruleSetName', string()],
      ],
      { description: 'PuffRuleSetV1InstructionData' }
    ),
    (value) => ({ ...value, discriminator: 3, puffRuleSetV1Discriminator: 0 })
  ) as Serializer<
    PuffRuleSetV1InstructionDataArgs,
    PuffRuleSetV1InstructionData
  >;
}

// Args.
export type PuffRuleSetV1InstructionArgs = PuffRuleSetV1InstructionDataArgs;

// Instruction.
export function puffRuleSetV1(
  context: Pick<Context, 'payer' | 'programs'>,
  input: PuffRuleSetV1InstructionAccounts & PuffRuleSetV1InstructionArgs
): TransactionBuilder {
  // Program ID.
  const programId = context.programs.getPublicKey(
    'mplTokenAuthRules',
    'auth9SigNpDKz4sJJ1DfCTuZrZNSAgh9sFD3rboVmgg'
  );

  // Accounts.
  const resolvedAccounts: ResolvedAccountsWithIndices = {
    payer: { index: 0, isWritable: true, value: input.payer ?? null },
    ruleSetPda: { index: 1, isWritable: true, value: input.ruleSetPda ?? null },
    systemProgram: {
      index: 2,
      isWritable: false,
      value: input.systemProgram ?? null,
    },
  };

  // Arguments.
  const resolvedArgs: PuffRuleSetV1InstructionArgs = { ...input };

  // Default values.
  if (!resolvedAccounts.payer.value) {
    resolvedAccounts.payer.value = context.payer;
  }
  if (!resolvedAccounts.systemProgram.value) {
    resolvedAccounts.systemProgram.value = context.programs.getPublicKey(
      'splSystem',
      '11111111111111111111111111111111'
    );
    resolvedAccounts.systemProgram.isWritable = false;
  }

  // Accounts in order.
  const orderedAccounts: ResolvedAccount[] = Object.values(
    resolvedAccounts
  ).sort((a, b) => a.index - b.index);

  // Keys and Signers.
  const [keys, signers] = getAccountMetasAndSigners(
    orderedAccounts,
    'programId',
    programId
  );

  // Data.
  const data = getPuffRuleSetV1InstructionDataSerializer().serialize(
    resolvedArgs as PuffRuleSetV1InstructionDataArgs
  );

  // Bytes Created On Chain.
  const bytesCreatedOnChain = 0;

  return transactionBuilder([
    { instruction: { keys, programId, data }, signers, bytesCreatedOnChain },
  ]);
}
