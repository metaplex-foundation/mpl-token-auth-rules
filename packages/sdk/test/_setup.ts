import { Connection, Keypair, PublicKey, SystemProgram, Transaction } from '@solana/web3.js';
import { LOCALHOST } from '@metaplex-foundation/amman';
import {
  createCreateOrUpdateInstruction,
  createWriteToBufferInstruction,
  findRuleSetPDA,
  findRuleSetBufferPDA,
  PROGRAM_ID,
  createPuffRuleSetInstruction,
} from '../src/mpl-token-auth-rules';
import { amman } from './_amman';

const CHUNK_SIZE = 900;

export const getConnectionAndPayer = async () => {
  const payer = Keypair.generate();
  const connection = new Connection(LOCALHOST, 'confirmed');
  await amman.airdrop(connection, payer.publicKey, 20);
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
  data: Uint8Array | PublicKey,
) => {
  const ruleSetAddress = await findRuleSetPDA(payer.publicKey, name);

  const createIX = createCreateOrUpdateInstruction(
    {
      payer: payer.publicKey,
      ruleSetPda: ruleSetAddress[0],
      systemProgram: SystemProgram.programId,
      bufferPda: data instanceof PublicKey ? data : null,
    },
    {
      createOrUpdateArgs: {
        __kind: 'V1',
        serializedRuleSet: data instanceof PublicKey ? new Uint8Array() : data,
      },
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

export const createOrUpdateLargeRuleset = async (
  connection: Connection,
  payer: Keypair,
  name: string,
  data: Uint8Array,
) => {
  const chunks = Math.ceil(data.length / CHUNK_SIZE);

  for (let i = 0; i < chunks; i++) {
    const chunk = data.slice(i * CHUNK_SIZE, (i + 1) * CHUNK_SIZE);
    await writeAndPuff(connection, payer, name, chunk);
  }

  const bufferAddress = await findRuleSetBufferPDA(payer.publicKey);
  return createOrUpdateRuleset(connection, payer, name, bufferAddress[0]);
};

export const writeAndPuff = async (
  connection: Connection,
  payer: Keypair,
  name: string,
  data: Uint8Array,
  overwrite = false,
) => {
  const bufferAddress = await findRuleSetBufferPDA(payer.publicKey);

  const writeIX = createWriteToBufferInstruction(
    {
      payer: payer.publicKey,
      bufferPda: bufferAddress[0],
      systemProgram: SystemProgram.programId,
    },
    {
      writeToBufferArgs: { __kind: 'V1', serializedRuleSet: data, overwrite },
    },
    PROGRAM_ID,
  );

  const ruleSetAddress = await findRuleSetPDA(payer.publicKey, name);

  const puffIX = createPuffRuleSetInstruction(
    {
      payer: payer.publicKey,
      ruleSetPda: ruleSetAddress[0],
      systemProgram: SystemProgram.programId,
    },
    {
      puffRuleSetArgs: { __kind: 'V1', ruleSetName: name },
    },
    PROGRAM_ID,
  );

  const tx = new Transaction().add(writeIX, puffIX);

  const { blockhash } = await connection.getLatestBlockhash();
  tx.recentBlockhash = blockhash;
  tx.feePayer = payer.publicKey;
  const sig = await connection.sendTransaction(tx, [payer]);
  await connection.confirmTransaction(sig);
  return bufferAddress[0];
};
