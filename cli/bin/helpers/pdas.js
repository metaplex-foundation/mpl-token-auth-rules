"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.findEscrowPda = exports.findTriflePda = exports.findEscrowConstraintModelPda = void 0;
const web3_js_1 = require("@solana/web3.js");
const mpl_token_metadata_1 = require("@metaplex-foundation/mpl-token-metadata");
const generated_1 = require("../../../js/src/generated");
const utils_1 = require("./utils");
const findEscrowConstraintModelPda = async (creator, name) => {
    return await web3_js_1.PublicKey.findProgramAddress([Buffer.from("escrow"), creator.toBuffer(), Buffer.from(name)], new web3_js_1.PublicKey(generated_1.PROGRAM_ADDRESS));
};
exports.findEscrowConstraintModelPda = findEscrowConstraintModelPda;
const findTriflePda = async (mint, authority) => {
    return await web3_js_1.PublicKey.findProgramAddress([Buffer.from("trifle"), mint.toBuffer(), authority.toBuffer()], new web3_js_1.PublicKey(generated_1.PROGRAM_ADDRESS));
};
exports.findTriflePda = findTriflePda;
const findEscrowPda = async (mint, authority, creator) => {
    const seeds = [
        Buffer.from("metadata"),
        new web3_js_1.PublicKey(mpl_token_metadata_1.PROGRAM_ADDRESS).toBuffer(),
        mint.toBuffer(),
        Uint8Array.from([authority]),
    ];
    if (authority == utils_1.EscrowAuthority.Creator) {
        if (creator) {
            seeds.push(creator.toBuffer());
        }
        else {
            throw new Error("Creator is required");
        }
    }
    seeds.push(Buffer.from("escrow"));
    return await web3_js_1.PublicKey.findProgramAddress(seeds, new web3_js_1.PublicKey(mpl_token_metadata_1.PROGRAM_ADDRESS));
};
exports.findEscrowPda = findEscrowPda;
