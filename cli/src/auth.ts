import { encode, decode } from '@msgpack/msgpack';
import { createTokenAuthorizationRules, validateOperation } from './helpers/mpl-token-auth-rules';
import { Keypair, Connection, LAMPORTS_PER_SOL, PublicKey } from '@solana/web3.js';
import { Command, program } from "commander";
import log from "loglevel";
import * as fs from 'fs';
import { findRuleSetPDA } from './helpers/pda';
import { Payload, RuleSetHeader, ruleSetHeaderBeet, ruleSetRevisionMapV1Beet } from '../../packages/sdk/src/generated';
import { getLatestRuleSet } from '../../packages/sdk/src/mpl-token-auth-rules';

program
    .command("create")
    .option(
        "-e, --env <string>",
        "Solana cluster env name",
        "devnet", //mainnet-beta, testnet, devnet
    )
    .option("-r, --rpc <string>", "The endpoint to connect to.")
    .option("-k, --keypair <path>", `Solana wallet location`)
    .option("-l, --log-level <string>", "log level", setLogLevel)
    .option("-n, --name <string>", "The name of the ruleset.")
    .option("-rs, --ruleset <string>", "The ruleset json file.")
    .action(async (directory, cmd) => {
        const { keypair, env, rpc, name, ruleset } = cmd.opts();
        let payer = loadKeypair(keypair);
        const connection = new Connection(rpc, "finalized");

        // Airdrop some Sol if we're on localnet.
        if (rpc == "http://localhost:8899") {
            const airdropSignature = await connection.requestAirdrop(
                payer.publicKey,
                10 * LAMPORTS_PER_SOL
            );
            await connection.confirmTransaction(airdropSignature);
        }

        const rulesetFile = JSON.parse(fs.readFileSync(ruleset, 'utf-8'));
        rulesetFile[1] = name;
        rulesetFile[2] = Array.from(payer.publicKey.toBytes());

        const encoded = encode(rulesetFile);
        console.log(encoded)
        let rulesetAddress = await createTokenAuthorizationRules(connection, payer, name, encoded);
        let rulesetData = await connection.getAccountInfo(rulesetAddress);
        let rulesetDecoded = decode(rulesetData?.data);
        console.log("RuleSet Decoded: " + JSON.stringify(rulesetDecoded, null, 2));
    });

program
    .command("validate")
    .option(
        "-e, --env <string>",
        "Solana cluster env name",
        "devnet", //mainnet-beta, testnet, devnet
    )
    .option("-r, --rpc <string>", "The endpoint to connect to.")
    .option("-k, --keypair <path>", `Solana wallet location`)
    .option("-l, --log-level <string>", "log level", setLogLevel)
    .option("-n, --name <string>", "The name of the ruleset.")
    .option("-op, --operation <string>", "The operation to validate.")
    .option("-da, --destination_address <string>", "The destination address.")
    .option("-ds, --derived_seeds <items>", "The derivation seeds as a comma-separated list.")
    .option("-am, --amount <int>", "The amount.")
    .option("-tl, --tree_leaf <string>", "The merkle tree leaf.")
    .option("-tp, --tree_proof <items>", "The merkle tree proof as a comma-separated list.")
    .action(async (directory, cmd) => {
        const { keypair, env, rpc, name,
            operation, destination_address, derived_seeds, amount, tree_leaf, tree_proof } = cmd.opts();
        let payer = loadKeypair(keypair);
        const connection = new Connection(rpc, "finalized");

        console.log("Operation: " + operation);
        console.log("Destination Address: " + destination_address);
        console.log("Derived Seeds: " + derived_seeds);
        console.log("Amount: " + amount);
        console.log("Tree Leaf: " + tree_leaf);
        console.log("Tree Proof: " + tree_proof);

        // let payload: Payload = {
        //     amount,
        //     destinationKey: new PublicKey(destination_address),
        //     derivedKeySeeds: null,
        //     treeMatchLeaf: null,
        // };
        // payload.amount = amount;
        // let result = await validateOperation(connection, payer, name, operation, payload);
        // console.log("Result: " + result);
    });
    
program
    .command("print")
    .option(
        "-e, --env <string>",
        "Solana cluster env name",
        "devnet", //mainnet-beta, testnet, devnet
    )
    .option("-r, --rpc <string>", "The endpoint to connect to.")
    .option("-l, --log-level <string>", "log level", setLogLevel)
    .option("-a, --address <string>", "The address of the ruleset.")
    .action(async (directory, cmd) => {
        const { keypair, env, rpc, address } = cmd.opts();
        const connection = new Connection(rpc, "finalized");

        let rulesetPDA = new PublicKey(address);
        let rulesetData = await connection.getAccountInfo(rulesetPDA);
        let data = rulesetData?.data as Buffer;
        const [header, header_num] = ruleSetHeaderBeet.deserialize(data.slice(0, 9));
        const [revmap, revmap_num] = ruleSetRevisionMapV1Beet.deserialize(data.slice(parseInt(header.revMapVersionLocation) + 1, data.length));
        let latestRevision = parseInt(revmap.ruleSetRevisions[revmap.ruleSetRevisions.length - 1]);
        let rulesetDecoded = decode(data.slice(latestRevision + 1, parseInt(header.revMapVersionLocation)));
        // console.log(rulesetDecoded);
        // console.log(JSON.stringify(rulesetDecoded, null, 2));
        console.log(getLatestRuleSet(data));
    });


// eslint-disable-next-line @typescript-eslint/no-unused-vars
function setLogLevel(value, prev) {
    if (value === undefined || value === null) {
        return;
    }
    log.info("setting the log value to: " + value);
    log.setLevel(value);
}

function loadKeypair(keypairPath) {
    const decodedKey = new Uint8Array(
        JSON.parse(
            fs.readFileSync(keypairPath).toString()
        ));

    return Keypair.fromSecretKey(decodedKey);
}

program
    .version("0.0.1")
    .description("CLI for controlling and managing RuleSets.")
    .parse(process.argv);