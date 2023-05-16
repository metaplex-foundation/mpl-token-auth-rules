import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_INSTRUCTIONS_PUBKEY,
  Transaction,
} from '@solana/web3.js';
import { findRuleSetPDA } from './pda';
import {
  createCreateOrUpdateInstruction,
  createPuffRuleSetInstruction,
  createValidateInstruction,
  createWriteToBufferInstruction,
  findRuleSetBufferPDA,
  Payload,
  PROGRAM_ID,
} from '@metaplex-foundation/mpl-token-auth-rules';

const PUFF_CHUNK_SIZE = 10_000;
const CHUNK_SIZE = 900;

export const createOrUpdateRuleset = async (
  connection: Connection,
  payer: Keypair,
  name: string,
  data: Uint8Array | PublicKey,
) => {
  const [ruleSetAddress] = await findRuleSetPDA(payer.publicKey, name);

  const createIX = createCreateOrUpdateInstruction(
    {
      payer: payer.publicKey,
      ruleSetPda: ruleSetAddress,
      systemProgram: SystemProgram.programId,
      bufferPda: data instanceof PublicKey ? data : undefined,
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

  return ruleSetAddress;
};

export const createOrUpdateLargeRuleset = async (
  connection: Connection,
  payer: Keypair,
  name: string,
  data: Uint8Array,
) => {
  if (data.length <= CHUNK_SIZE) {
    return createOrUpdateRuleset(connection, payer, name, data);
  }

  // we first write the buffer
  const chunks = Math.ceil(data.length / CHUNK_SIZE);

  for (let i = 0; i < chunks; i++) {
    const chunk = data.slice(i * CHUNK_SIZE, Math.min((i + 1) * CHUNK_SIZE, data.length));
    console.log(`   + writing data slice ${i + 1} of ${chunks}: ${chunk.length} bytes`);
    await write(connection, payer, name, chunk, i == 0);
  }

  // then puff the rule set account
  const puffs = Math.ceil(data.length / PUFF_CHUNK_SIZE) - 1;

  if (puffs > 0) {
    for (let i = 0; i < puffs; i++) {
      await puff(connection, payer, name);
    }
  }

  const [bufferAddress] = await findRuleSetBufferPDA(payer.publicKey);
  return createOrUpdateRuleset(connection, payer, name, bufferAddress);
};

export const write = async (
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

  const tx = new Transaction().add(writeIX);

  const { blockhash } = await connection.getLatestBlockhash();
  tx.recentBlockhash = blockhash;
  tx.feePayer = payer.publicKey;
  const sig = await connection.sendTransaction(tx, [payer]);
  await connection.confirmTransaction(sig);
  return bufferAddress[0];
};

export const puff = async (
  connection: Connection,
  payer: Keypair,
  name: string,
  overwrite = false,
) => {
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

  const tx = new Transaction().add(puffIX);

  const { blockhash } = await connection.getLatestBlockhash();
  tx.recentBlockhash = blockhash;
  tx.feePayer = payer.publicKey;
  const sig = await connection.sendTransaction(tx, [payer]);
  await connection.confirmTransaction(sig);
};
