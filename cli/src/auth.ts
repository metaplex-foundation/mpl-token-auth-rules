import { encode, decode } from '@msgpack/msgpack';
import { createTokenAuthorizationRules, validateOperation } from './helpers/mpl-token-auth-rules';
import { Keypair, Connection, LAMPORTS_PER_SOL, PublicKey } from '@solana/web3.js';
import { Command, program } from "commander";
import log from "loglevel";
import * as fs from 'fs';
import { findRuleSetPDA } from './helpers/pda';
import { Payload, PayloadVecIndexErrorError } from '../../packages/sdk/src/generated';

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
    .option("-m, --mint <string>", "The mint of the token being operated on.")
    .option("-p, --payload [triplets...]", "Colon separated payload pairs.")
    .action(async (directory, cmd) => {
        const { keypair, env, rpc, name, operation, mint, payload } = cmd.opts();
        let payer = loadKeypair(keypair);
        const connection = new Connection(rpc, "finalized");

        console.log("Operation: " + operation);
        console.log("Triplets: " + JSON.stringify(payload, null, 2));

        let p: Payload = { map: new Map()};
        let additional_accounts: PublicKey[] = [];
        for (let pair of payload) {
            let [key, type, value] = pair.split(":");
            console.log("Key: ", key, "\nType: ", type, "\nValue: ", value);
            if (type === "pubkey")
            {
                let pubkey = new PublicKey(value);
                additional_accounts.push(pubkey);
                p.map.set(key, { __kind: "Pubkey", fields: [pubkey]});
            }
            else if (type === "number")
            {
                p.map.set(key, { __kind: "Number", fields: [parseInt(value)]});
            }
        }

        let result = await validateOperation(connection, payer, name, new PublicKey(mint), operation, p, additional_accounts);
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
    .option("-k, --keypair <path>", `Solana wallet location`)
    .option("-l, --log-level <string>", "log level", setLogLevel)
    .option("-c, --creator <string>", "The address of the ruleset creator.")
    .option("-n, --name <string>", "The name of the ruleset.")
    .action(async (directory, cmd) => {
        const { keypair, env, rpc, name, creator } = cmd.opts();
        let payer = loadKeypair(keypair);
        const connection = new Connection(rpc, "finalized");

        let rulesetPDA = await findRuleSetPDA(new PublicKey(creator), name);
        let rulesetData = await connection.getAccountInfo(rulesetPDA[0]);
        let rulesetDecoded = decode(rulesetData?.data);
        console.log("RuleSet Decoded: " + JSON.stringify(rulesetDecoded, null, 2));
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