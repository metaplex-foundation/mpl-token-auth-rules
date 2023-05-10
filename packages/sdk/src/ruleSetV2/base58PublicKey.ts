import type { PublicKey } from '@solana/web3.js';

export type Base58PublicKey = string;

export const toBase58PublicKey = (publicKey: PublicKey | string): Base58PublicKey => {
  return typeof publicKey === 'string' ? publicKey : publicKey.toBase58();
};
