/* eslint-disable import/no-extraneous-dependencies */
import {
  createUmi as baseTestCreateUmi,
  testPlugins,
} from '@metaplex-foundation/umi-bundle-tests';
import { mplTokenMetadata } from '@metaplex-foundation/mpl-token-metadata';
import {
  Context,
  Pda,
  PublicKey,
  assertAccountExists,
  base16,
  createUmi as baseCreateUmi,
  publicKey,
  publicKeyBytes,
} from '@metaplex-foundation/umi';
import { mplTokenAuthRules, RuleV2, getRuleV2Serializer } from '../src';

export const createUmi = async () =>
  (await baseTestCreateUmi()).use(mplTokenAuthRules()).use(mplTokenMetadata());

export const createUmiSync = () =>
  baseCreateUmi()
    .use(testPlugins())
    .use(mplTokenAuthRules())
    .use(mplTokenMetadata());

export const serializeRuleV2AsHex = (
  context: Pick<Context, 'serializer'>,
  rule: RuleV2
): string => {
  const serializedRule = getRuleV2Serializer(context).serialize(rule);
  return toHex(serializedRule);
};

export const deserializeRuleV2FromHex = (
  context: Pick<Context, 'serializer'>,
  bufferAsHex: string
): RuleV2 => {
  const buffer = base16.serialize(bufferAsHex);
  return getRuleV2Serializer(context).deserialize(buffer)[0];
};

export const toHex = (buffer: Uint8Array | PublicKey): string => {
  if (typeof buffer === 'string') buffer = publicKeyBytes(buffer);
  return base16.deserialize(buffer)[0];
};

export const toString32Hex = (
  context: Pick<Context, 'serializer'>,
  value: string
): string => {
  const buffer = context.serializer.string({ size: 32 }).serialize(value);
  return toHex(buffer);
};

export const fetchRuleSetSize = async (
  context: Pick<Context, 'rpc'>,
  ruleSetPda: PublicKey | Pda
) => {
  const rawAccount = await context.rpc.getAccount(publicKey(ruleSetPda, false));
  assertAccountExists(rawAccount);
  return rawAccount.data.length;
};
