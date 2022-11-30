"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.showTrifle = exports.showModel = exports.transferOut = exports.transferIn = exports.createTrifle = exports.addTokensConstraint = exports.addCollectionConstraint = exports.addNoneConstraint = exports.createConstraintModel = void 0;
const web3_js_1 = require("@solana/web3.js");
const generated_1 = require("../../../js/src/generated");
const pdas_1 = require("./pdas");
const spl_token_1 = require("@solana/spl-token");
const js_1 = require("@metaplex-foundation/js");
const mpl_token_metadata_1 = require("@metaplex-foundation/mpl-token-metadata");
const utils_1 = require("./utils");
const createConstraintModel = async (connection, keypair, name, schema) => {
    const escrowConstraintModel = await (0, pdas_1.findEscrowConstraintModelPda)(keypair.publicKey, name);
    const createIX = (0, generated_1.createCreateEscrowConstraintModelAccountInstruction)({
        escrowConstraintModel: escrowConstraintModel[0],
        payer: keypair.publicKey,
        updateAuthority: keypair.publicKey,
    }, {
        createEscrowConstraintModelAccountArgs: {
            name,
            schemaUri: schema,
        },
    });
    const tx = new web3_js_1.Transaction().add(createIX);
    const { blockhash } = await connection.getLatestBlockhash();
    tx.recentBlockhash = blockhash;
    tx.feePayer = keypair.publicKey;
    const sig = await connection.sendTransaction(tx, [keypair]);
    // await connection.sendTransaction(tx, [keypair]);
    await connection.confirmTransaction(sig, "finalized");
    return escrowConstraintModel[0];
};
exports.createConstraintModel = createConstraintModel;
const addNoneConstraint = async (connection, keypair, name, tokenLimit, transferEffects, model) => {
    const addIX = (0, generated_1.createAddNoneConstraintToEscrowConstraintModelInstruction)({
        constraintModel: model,
        payer: keypair.publicKey,
        updateAuthority: keypair.publicKey,
        systemProgram: web3_js_1.SystemProgram.programId,
        sysvarInstructions: web3_js_1.SYSVAR_INSTRUCTIONS_PUBKEY,
    }, {
        addNoneConstraintToEscrowConstraintModelArgs: {
            constraintName: name,
            tokenLimit: tokenLimit,
            transferEffects,
        },
    });
    const tx = new web3_js_1.Transaction().add(addIX);
    const { blockhash } = await connection.getLatestBlockhash();
    tx.recentBlockhash = blockhash;
    tx.feePayer = keypair.publicKey;
    const sig = await connection.sendTransaction(tx, [keypair], { skipPreflight: true });
    await connection.confirmTransaction(sig, "finalized");
};
exports.addNoneConstraint = addNoneConstraint;
const addCollectionConstraint = async (connection, keypair, name, tokenLimit, collection, transferEffects, model) => {
    const collectionMintMetadata = await (0, js_1.findMetadataPda)(collection);
    const addIX = (0, generated_1.createAddCollectionConstraintToEscrowConstraintModelInstruction)({
        constraintModel: model,
        payer: keypair.publicKey,
        updateAuthority: keypair.publicKey,
        collectionMint: collection,
        collectionMintMetadata,
        sysvarInstructions: web3_js_1.SYSVAR_INSTRUCTIONS_PUBKEY,
    }, {
        addCollectionConstraintToEscrowConstraintModelArgs: {
            constraintName: name,
            tokenLimit: tokenLimit,
            transferEffects,
        },
    });
    const tx = new web3_js_1.Transaction().add(addIX);
    const { blockhash } = await connection.getLatestBlockhash();
    tx.recentBlockhash = blockhash;
    tx.feePayer = keypair.publicKey;
    const sig = await connection.sendTransaction(tx, [keypair], {
        skipPreflight: true,
    });
    await connection.confirmTransaction(sig, "finalized");
};
exports.addCollectionConstraint = addCollectionConstraint;
const addTokensConstraint = async (connection, keypair, name, tokenLimit, tokens, transferEffects, model) => {
    const addIX = (0, generated_1.createAddTokensConstraintToEscrowConstraintModelInstruction)({
        constraintModel: model,
        payer: keypair.publicKey,
        updateAuthority: keypair.publicKey,
        sysvarInstructions: web3_js_1.SYSVAR_INSTRUCTIONS_PUBKEY,
    }, {
        addTokensConstraintToEscrowConstraintModelArgs: {
            constraintName: name,
            tokenLimit: tokenLimit,
            tokens,
            transferEffects,
        },
    });
    const tx = new web3_js_1.Transaction().add(addIX);
    const { blockhash } = await connection.getLatestBlockhash();
    tx.recentBlockhash = blockhash;
    tx.feePayer = keypair.publicKey;
    const sig = await connection.sendTransaction(tx, [keypair], {
        skipPreflight: true,
    });
    await connection.confirmTransaction(sig, "finalized");
};
exports.addTokensConstraint = addTokensConstraint;
const createTrifle = async (connection, nft, keypair, model_name) => {
    const escrowConstraintModel = await (0, pdas_1.findEscrowConstraintModelPda)(keypair.publicKey, model_name);
    const trifleAddress = await (0, pdas_1.findTriflePda)(nft.mint.address, keypair.publicKey);
    const escrowAccountAddress = await (0, pdas_1.findEscrowPda)(nft.mint.address, utils_1.EscrowAuthority.Creator, trifleAddress[0]);
    const createIX = (0, generated_1.createCreateTrifleAccountInstruction)({
        escrow: escrowAccountAddress[0],
        metadata: nft.metadataAddress,
        mint: nft.mint.address,
        tokenAccount: nft.token.address,
        edition: nft.edition.address,
        trifleAccount: trifleAddress[0],
        trifleAuthority: keypair.publicKey,
        constraintModel: escrowConstraintModel[0],
        payer: keypair.publicKey,
        tokenMetadataProgram: new web3_js_1.PublicKey(mpl_token_metadata_1.PROGRAM_ADDRESS),
        sysvarInstructions: web3_js_1.SYSVAR_INSTRUCTIONS_PUBKEY,
    });
    const tx = new web3_js_1.Transaction().add(createIX);
    const { blockhash } = await connection.getLatestBlockhash();
    tx.recentBlockhash = blockhash;
    tx.feePayer = keypair.publicKey;
    const sig = await connection.sendTransaction(tx, [keypair], {
        skipPreflight: false,
    });
    await connection.confirmTransaction(sig, "finalized");
    return trifleAddress[0];
};
exports.createTrifle = createTrifle;
const transferIn = async (connection, escrowNft, escrowAccountAddress, nft, keypair, slot) => {
    const escrowConstraintModel = await (0, pdas_1.findEscrowConstraintModelPda)(keypair.publicKey, "test");
    const trifleAddress = await (0, pdas_1.findTriflePda)(escrowNft.mint.address, keypair.publicKey);
    const dst = await (0, spl_token_1.getAssociatedTokenAddress)(nft.mint.address, escrowAccountAddress, true);
    // trifle: web3.PublicKey;
    // trifleAuthority: web3.PublicKey;
    // payer: web3.PublicKey;
    // constraintModel: web3.PublicKey;
    // escrow: web3.PublicKey;
    // escrowMint?: web3.PublicKey;
    // escrowToken?: web3.PublicKey;
    // escrowEdition?: web3.PublicKey;
    // attributeMint?: web3.PublicKey;
    // attributeSrcToken?: web3.PublicKey;
    // attributeDstToken?: web3.PublicKey;
    // attributeMetadata?: web3.PublicKey;
    // attributeEdition?: web3.PublicKey;
    // attributeCollectionMetadata?: web3.PublicKey;
    const transferIX = (0, generated_1.createTransferInInstruction)({
        trifle: trifleAddress[0],
        constraintModel: escrowConstraintModel[0],
        escrow: escrowAccountAddress,
        payer: keypair.publicKey,
        trifleAuthority: keypair.publicKey,
        attributeMint: nft.mint.address,
        attributeSrcToken: nft.token.address,
        attributeDstToken: dst,
        attributeMetadata: nft.metadataAddress,
        escrowMint: escrowNft.mint.address,
        escrowToken: escrowNft.token.address,
        splToken: new web3_js_1.PublicKey(spl_token_1.TOKEN_PROGRAM_ID),
        splAssociatedTokenAccount: new web3_js_1.PublicKey(spl_token_1.ASSOCIATED_TOKEN_PROGRAM_ID),
        tokenMetadataProgram: new web3_js_1.PublicKey(mpl_token_metadata_1.PROGRAM_ADDRESS),
    }, {
        transferInArgs: { amount: 1, slot },
    });
    const tx = new web3_js_1.Transaction().add(transferIX);
    const { blockhash } = await connection.getLatestBlockhash();
    tx.recentBlockhash = blockhash;
    tx.feePayer = keypair.publicKey;
    // console.log(tx);
    const sig = await connection.sendTransaction(tx, [keypair], {
        skipPreflight: true,
    });
    await connection.confirmTransaction(sig, "finalized");
};
exports.transferIn = transferIn;
const transferOut = async (connection, escrowNft, escrowAccountAddress, nft, keypair, slot) => {
    const escrowConstraintModel = await (0, pdas_1.findEscrowConstraintModelPda)(keypair.publicKey, "test");
    const trifleAddress = await (0, pdas_1.findTriflePda)(escrowNft.mint.address, keypair.publicKey);
    const dst = await (0, spl_token_1.getAssociatedTokenAddress)(nft.mint.address, keypair.publicKey, true);
    const transferIX = (0, generated_1.createTransferOutInstruction)({
        trifleAccount: trifleAddress[0],
        constraintModel: escrowConstraintModel[0],
        escrowAccount: escrowAccountAddress,
        escrowTokenAccount: escrowNft.token.address,
        escrowMint: escrowNft.mint.address,
        escrowMetadata: escrowNft.metadataAddress,
        payer: keypair.publicKey,
        trifleAuthority: keypair.publicKey,
        attributeMint: nft.mint.address,
        attributeSrcTokenAccount: nft.token.address,
        attributeDstTokenAccount: dst,
        attributeMetadata: nft.metadataAddress,
        splAssociatedTokenAccount: new web3_js_1.PublicKey(spl_token_1.ASSOCIATED_TOKEN_PROGRAM_ID),
        splToken: new web3_js_1.PublicKey(spl_token_1.TOKEN_PROGRAM_ID),
        tokenMetadataProgram: new web3_js_1.PublicKey(mpl_token_metadata_1.PROGRAM_ADDRESS),
        sysvarInstructions: web3_js_1.SYSVAR_INSTRUCTIONS_PUBKEY,
    }, {
        transferOutArgs: { amount: 1, slot },
    });
    const tx = new web3_js_1.Transaction().add(transferIX);
    const { blockhash } = await connection.getLatestBlockhash();
    tx.recentBlockhash = blockhash;
    tx.feePayer = keypair.publicKey;
    // console.log(tx);
    const sig = await connection.sendTransaction(tx, [keypair], {
        skipPreflight: true,
    });
    await connection.confirmTransaction(sig, "finalized");
};
exports.transferOut = transferOut;
const showModel = async (connection, modelAddress) => {
    // console.log("Fetching " + modelAddress.toString());
    const accountInfo = await connection.getAccountInfo(modelAddress);
    if (accountInfo) {
        const account = generated_1.EscrowConstraintModel.fromAccountInfo(accountInfo)[0];
        console.log(JSON.stringify(account.pretty(), utils_1.map_replacer));
    }
    else {
        console.log("Unable to fetch account");
    }
};
exports.showModel = showModel;
const showTrifle = async (connection, trifleAddress) => {
    // console.log("Fetching " + trifleAddress.toString());
    const accountInfo = await connection.getAccountInfo(trifleAddress);
    if (accountInfo) {
        const account = generated_1.Trifle.fromAccountInfo(accountInfo)[0];
        console.log(JSON.stringify(account.pretty(), utils_1.map_replacer));
    }
    else {
        console.log("Unable to fetch account");
    }
};
exports.showTrifle = showTrifle;
