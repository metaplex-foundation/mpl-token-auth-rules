import { PublicKey } from "@solana/web3.js";
import { PREFIX, PROGRAM_ID } from "../../../packages/sdk/src/mpl-token-auth-rules";

export const findRuleSetPDA = async (payer: PublicKey, name: string) => {
    return await PublicKey.findProgramAddress(
        [
            Buffer.from("rule_set"),
            payer.toBuffer(),
            Buffer.from(name),
        ],
        PROGRAM_ID,
    );
}