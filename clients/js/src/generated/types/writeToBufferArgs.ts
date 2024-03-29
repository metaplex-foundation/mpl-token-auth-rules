/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/metaplex-foundation/kinobi
 */

import {
  GetDataEnumKind,
  GetDataEnumKindContent,
  Serializer,
  bool,
  bytes,
  dataEnum,
  struct,
  u32,
} from '@metaplex-foundation/umi/serializers';

export type WriteToBufferArgs = {
  __kind: 'V1';
  data: Uint8Array;
  overwrite: boolean;
};

export type WriteToBufferArgsArgs = WriteToBufferArgs;

export function getWriteToBufferArgsSerializer(): Serializer<
  WriteToBufferArgsArgs,
  WriteToBufferArgs
> {
  return dataEnum<WriteToBufferArgs>(
    [
      [
        'V1',
        struct<GetDataEnumKindContent<WriteToBufferArgs, 'V1'>>([
          ['data', bytes({ size: u32() })],
          ['overwrite', bool()],
        ]),
      ],
    ],
    { description: 'WriteToBufferArgs' }
  ) as Serializer<WriteToBufferArgsArgs, WriteToBufferArgs>;
}

// Data Enum Helpers.
export function writeToBufferArgs(
  kind: 'V1',
  data: GetDataEnumKindContent<WriteToBufferArgsArgs, 'V1'>
): GetDataEnumKind<WriteToBufferArgsArgs, 'V1'>;
export function writeToBufferArgs<K extends WriteToBufferArgsArgs['__kind']>(
  kind: K,
  data?: any
): Extract<WriteToBufferArgsArgs, { __kind: K }> {
  return Array.isArray(data)
    ? { __kind: kind, fields: data }
    : { __kind: kind, ...(data ?? {}) };
}
export function isWriteToBufferArgs<K extends WriteToBufferArgs['__kind']>(
  kind: K,
  value: WriteToBufferArgs
): value is WriteToBufferArgs & { __kind: K } {
  return value.__kind === kind;
}
