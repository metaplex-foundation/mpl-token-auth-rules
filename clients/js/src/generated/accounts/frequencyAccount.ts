/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/metaplex-foundation/kinobi
 */

import {
  Account,
  Context,
  Pda,
  PublicKey,
  RpcAccount,
  RpcGetAccountOptions,
  RpcGetAccountsOptions,
  assertAccountExists,
  deserializeAccount,
  gpaBuilder,
  publicKey as toPublicKey,
} from '@metaplex-foundation/umi';
import { Serializer, i64, struct } from '@metaplex-foundation/umi/serializers';
import { Key, KeyArgs, getKeySerializer } from '../types';

export type FrequencyAccount = Account<FrequencyAccountAccountData>;

export type FrequencyAccountAccountData = {
  key: Key;
  lastUpdate: bigint;
  period: bigint;
};

export type FrequencyAccountAccountDataArgs = {
  key: KeyArgs;
  lastUpdate: number | bigint;
  period: number | bigint;
};

export function getFrequencyAccountAccountDataSerializer(): Serializer<
  FrequencyAccountAccountDataArgs,
  FrequencyAccountAccountData
> {
  return struct<FrequencyAccountAccountData>(
    [
      ['key', getKeySerializer()],
      ['lastUpdate', i64()],
      ['period', i64()],
    ],
    { description: 'FrequencyAccountAccountData' }
  ) as Serializer<FrequencyAccountAccountDataArgs, FrequencyAccountAccountData>;
}

export function deserializeFrequencyAccount(
  rawAccount: RpcAccount
): FrequencyAccount {
  return deserializeAccount(
    rawAccount,
    getFrequencyAccountAccountDataSerializer()
  );
}

export async function fetchFrequencyAccount(
  context: Pick<Context, 'rpc'>,
  publicKey: PublicKey | Pda,
  options?: RpcGetAccountOptions
): Promise<FrequencyAccount> {
  const maybeAccount = await context.rpc.getAccount(
    toPublicKey(publicKey, false),
    options
  );
  assertAccountExists(maybeAccount, 'FrequencyAccount');
  return deserializeFrequencyAccount(maybeAccount);
}

export async function safeFetchFrequencyAccount(
  context: Pick<Context, 'rpc'>,
  publicKey: PublicKey | Pda,
  options?: RpcGetAccountOptions
): Promise<FrequencyAccount | null> {
  const maybeAccount = await context.rpc.getAccount(
    toPublicKey(publicKey, false),
    options
  );
  return maybeAccount.exists ? deserializeFrequencyAccount(maybeAccount) : null;
}

export async function fetchAllFrequencyAccount(
  context: Pick<Context, 'rpc'>,
  publicKeys: Array<PublicKey | Pda>,
  options?: RpcGetAccountsOptions
): Promise<FrequencyAccount[]> {
  const maybeAccounts = await context.rpc.getAccounts(
    publicKeys.map((key) => toPublicKey(key, false)),
    options
  );
  return maybeAccounts.map((maybeAccount) => {
    assertAccountExists(maybeAccount, 'FrequencyAccount');
    return deserializeFrequencyAccount(maybeAccount);
  });
}

export async function safeFetchAllFrequencyAccount(
  context: Pick<Context, 'rpc'>,
  publicKeys: Array<PublicKey | Pda>,
  options?: RpcGetAccountsOptions
): Promise<FrequencyAccount[]> {
  const maybeAccounts = await context.rpc.getAccounts(
    publicKeys.map((key) => toPublicKey(key, false)),
    options
  );
  return maybeAccounts
    .filter((maybeAccount) => maybeAccount.exists)
    .map((maybeAccount) =>
      deserializeFrequencyAccount(maybeAccount as RpcAccount)
    );
}

export function getFrequencyAccountGpaBuilder(
  context: Pick<Context, 'rpc' | 'programs'>
) {
  const programId = context.programs.getPublicKey(
    'mplTokenAuthRules',
    'auth9SigNpDKz4sJJ1DfCTuZrZNSAgh9sFD3rboVmgg'
  );
  return gpaBuilder(context, programId)
    .registerFields<{
      key: KeyArgs;
      lastUpdate: number | bigint;
      period: number | bigint;
    }>({
      key: [0, getKeySerializer()],
      lastUpdate: [1, i64()],
      period: [9, i64()],
    })
    .deserializeUsing<FrequencyAccount>((account) =>
      deserializeFrequencyAccount(account)
    );
}

export function getFrequencyAccountSize(): number {
  return 17;
}
