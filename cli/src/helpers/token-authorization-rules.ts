import {
    Connection,
    Keypair,
    PublicKey,
    SystemProgram,
    SYSVAR_INSTRUCTIONS_PUBKEY,
    Transaction,
} from "@solana/web3.js";
import { createCreateInstruction, createValidateInstruction, Operation, Payload, PROGRAM_ID } from "../../../packages/sdk/src/token-authorization-rules";
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

    let op_type: Operation = Operation.Transfer;
    switch (operation) {
        case "Transfer":
            op_type = Operation.Transfer;
            break;
        case "Delegate":
            op_type = Operation.Delegate;
            break;
        case "SaleTransfer":
            op_type = Operation.SaleTransfer;
            break;
        case "MigrateClass":
            op_type = Operation.MigrateClass;
            break;
    }
    const ruleSetAddress = await findRuleSetPDA(payer.publicKey, name);
    let validateIX = createValidateInstruction(
        {
            payer: payer.publicKey,
            ruleset: ruleSetAddress[0],
            systemProgram: SystemProgram.programId,
        },
        {
            validateArgs: {
                name,
                operation: op_type,
                payload
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