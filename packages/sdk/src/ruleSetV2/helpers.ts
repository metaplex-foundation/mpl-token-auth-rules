import * as beetSolana from '@metaplex-foundation/beet-solana';
import { PublicKey } from '@solana/web3.js';
import { Base58PublicKey, toBase58PublicKey } from './base58PublicKey';

export const serializeString32 = (str: string): Buffer => {
  const buffer = Buffer.alloc(32);
  buffer.write(str);
  return buffer;
};

export const deserializeString32 = (buffer: Buffer, offset = 0): string => {
  return buffer
    .subarray(offset, offset + 32)
    .toString()
    .replace(/\u0000/g, '');
};

export const serializePublicKey = (publicKey: PublicKey | Base58PublicKey): Buffer => {
  const buffer = Buffer.alloc(32);
  const web3JsPublicKey = typeof publicKey === 'string' ? new PublicKey(publicKey) : publicKey;
  beetSolana.publicKey.write(buffer, 0, web3JsPublicKey);
  return buffer;
};

export const deserializePublicKey = (buffer: Buffer, offset = 0): Base58PublicKey => {
  return toBase58PublicKey(beetSolana.publicKey.read(buffer, offset));
};
