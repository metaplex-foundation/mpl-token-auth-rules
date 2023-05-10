import { PublicKey } from '@solana/web3.js';
import { PREFIX } from './constants';
import { PROGRAM_ID } from './generated';

export const findRuleSetPDA = async (payer: PublicKey, name: string) => {
  return await PublicKey.findProgramAddress(
    [Buffer.from(PREFIX), payer.toBuffer(), Buffer.from(name)],
    PROGRAM_ID,
  );
};

export const findRuleSetBufferPDA = async (payer: PublicKey) => {
  return await PublicKey.findProgramAddress([Buffer.from(PREFIX), payer.toBuffer()], PROGRAM_ID);
};
