import {
    Connection,
    Keypair,
    PublicKey,
    SystemProgram,
    SYSVAR_INSTRUCTIONS_PUBKEY,
    Transaction,
} from "@solana/web3.js";
import { createCreateInstruction, PROGRAM_ID } from "../../../packages/sdk/src/token-authorization-rules";
import { findRuleSetPDA } from "./pda";

export const createTokenAuthorizationRules = async (
    connection: Connection,
    payer: Keypair,
    name: string,
    data: Uint8Array,
) => {
    const ruleSetAddress = await findRuleSetPDA(payer.publicKey, name);

    let createIX = createCreateInstruction(
        {
            payer: payer.publicKey,
            rulesetPda: ruleSetAddress[0],
            systemProgram: SystemProgram.programId,
        },
        {
            createArgs: { name, serializedRuleSet: data },
        },
        PROGRAM_ID,
    )

    const tx = new Transaction().add(createIX);

    const { blockhash } = await connection.getLatestBlockhash();
    tx.recentBlockhash = blockhash;
    tx.feePayer = payer.publicKey;
    const sig = await connection.sendTransaction(tx, [payer], { skipPreflight: true });
    // await connection.sendTransaction(tx, [payer]);
    await connection.confirmTransaction(sig, "finalized");
    return ruleSetAddress[0];
}