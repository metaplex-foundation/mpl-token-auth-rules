import {
    Connection,
    Keypair,
    PublicKey,
    SystemProgram,
    SYSVAR_INSTRUCTIONS_PUBKEY,
    Transaction,
} from "@solana/web3.js";
import { createCreateOrUpdateInstruction, createValidateInstruction, Payload, PROGRAM_ID } from "../../../packages/sdk/src/mpl-token-auth-rules";
import { findRuleSetPDA } from "./pda";
import { TokenMetadataProgram } from "@metaplex-foundation/js";

export const createTokenAuthorizationRules = async (
    connection: Connection,
    payer: Keypair,
    name: string,
    data: Uint8Array,
) => {
    const ruleSetAddress = await findRuleSetPDA(payer.publicKey, name);

    let createIX = createCreateOrUpdateInstruction(
        {
            payer: payer.publicKey,
            ruleSetPda: ruleSetAddress[0],
            systemProgram: SystemProgram.programId,
        },
        {
            createOrUpdateArgs: {__kind: "V1", serializedRuleSet: data },
        },
        PROGRAM_ID,
    )

    const tx = new Transaction().add(createIX);

    const { blockhash } = await connection.getLatestBlockhash();
    tx.recentBlockhash = blockhash;
    tx.feePayer = payer.publicKey;
    const sig = await connection.sendTransaction(tx, [payer], { skipPreflight: true });
    await connection.confirmTransaction(sig, "finalized");
    return ruleSetAddress[0];
}

export const validateOperation = async (
    connection: Connection,
    payer: Keypair,
    name: string,
    mint: PublicKey,
    operation: string,
    payload: Payload,
) => {

    const ruleSetAddress = await findRuleSetPDA(payer.publicKey, name);
    let validateIX = createValidateInstruction(
        {
            payer: payer.publicKey,
            mint,
            ruleSetPda: ruleSetAddress[0],
            systemProgram: SystemProgram.programId,
        },
        {
            validateArgs: {
                __kind: "V1",
                operation,
                payload,
                updateRuleState: true,
            },
        },
        PROGRAM_ID,
    );

    const tx = new Transaction().add(validateIX);

    const { blockhash } = await connection.getLatestBlockhash();
    tx.recentBlockhash = blockhash;
    tx.feePayer = payer.publicKey;
    const sig = await connection.sendTransaction(tx, [payer], { skipPreflight: true });
    await connection.confirmTransaction(sig, "finalized");
}