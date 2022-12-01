import { encode, decode } from '@msgpack/msgpack';
import { createTokenAuthorizationRules } from './helpers/tar';
import { Keypair, Connection, LAMPORTS_PER_SOL, PublicKey } from '@solana/web3.js';
import { Command, program } from "commander";
import log from "loglevel";
import * as fs from 'fs';

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

        const encoded = encode(rulesetFile);
        let rulesetAddress = await createTokenAuthorizationRules(connection, payer, name, encoded);
        let rulesetData = await connection.getAccountInfo(rulesetAddress);
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