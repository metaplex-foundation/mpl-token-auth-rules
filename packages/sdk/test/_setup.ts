import { Connection, Keypair, SystemProgram, Transaction } from '@solana/web3.js';
import {
  createCreateOrUpdateInstruction,
  findRuleSetPDA,
  PROGRAM_ID,
} from '../src/mpl-token-auth-rules';

export const createOrUpdateRuleset = async (
  connection: Connection,
  payer: Keypair,
  name: string,
  data: Uint8Array,
) => {
  const ruleSetAddress = await findRuleSetPDA(payer.publicKey, name);

  const createIX = createCreateOrUpdateInstruction(
    {
      payer: payer.publicKey,
      ruleSetPda: ruleSetAddress[0],
      systemProgram: SystemProgram.programId,
    },
    {
      createOrUpdateArgs: { __kind: 'V1', serializedRuleSet: data },
    },
    PROGRAM_ID,
  );

  const tx = new Transaction().add(createIX);

  const { blockhash } = await connection.getLatestBlockhash();
  tx.recentBlockhash = blockhash;
  tx.feePayer = payer.publicKey;
  const sig = await connection.sendTransaction(tx, [payer], { skipPreflight: true });
  await connection.confirmTransaction(sig, 'finalized');
  return ruleSetAddress[0];
};
