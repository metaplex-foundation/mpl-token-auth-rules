import { Connection, Keypair, SystemProgram, Transaction } from '@solana/web3.js';
import { LOCALHOST } from '@metaplex-foundation/amman';
import {
  createCreateOrUpdateInstruction,
  findRuleSetPDA,
  PROGRAM_ID,
} from '../src/mpl-token-auth-rules';
import { amman } from './_amman';

export const getConnectionAndPayer = async () => {
  const payer = Keypair.generate();
  const connection = new Connection(LOCALHOST, 'confirmed');
  await amman.airdrop(connection, payer.publicKey, 2);
  const transactionHandler = amman.payerTransactionHandler(connection, payer);

  return {
    fstTxHandler: transactionHandler,
    connection,
    payer,
  };
};

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
  const sig = await connection.sendTransaction(tx, [payer]);
  await connection.confirmTransaction(sig);
  return ruleSetAddress[0];
};
