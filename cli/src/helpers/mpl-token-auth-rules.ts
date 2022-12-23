import {
    Connection,
    Keypair,
    PublicKey,
    SystemProgram,
    SYSVAR_INSTRUCTIONS_PUBKEY,
    Transaction,
} from "@solana/web3.js";
import { createCreateInstruction, createValidateInstruction, Payload, PROGRAM_ID } from "../../../packages/sdk/src/mpl-token-auth-rules";
import { findRuleSetPDA } from "./pda";
import { TokenMetadataProgram } from "@metaplex-foundation/js";

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
            ruleSetPda: ruleSetAddress[0],
            systemProgram: SystemProgram.programId,
        },
        {
            createArgs: { serializedRuleSet: data },
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
    operation: string,
    payload: Payload,
) => {

    let op_type: number = 0;
    switch (operation) {
        case "Transfer":
            op_type = 0;
            break;
        case "Delegate":
            op_type = 1;
            break;
        case "SaleTransfer":
            op_type = 2;
            break;
        case "MigrateClass":
            op_type = 3;
            break;
    }
    const ruleSetAddress = await findRuleSetPDA(payer.publicKey, name);
    let validateIX = createValidateInstruction(
        {
            ruleSet: ruleSetAddress[0],
            systemProgram: SystemProgram.programId,
        },
        {
            validateArgs: {
                operation: op_type,
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