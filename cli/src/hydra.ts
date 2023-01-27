import { encode, decode } from '@msgpack/msgpack';
import { createCreateOrUpdateInstruction, PREFIX, PROGRAM_ID } from './helpers/mpl-token-auth-rules';

import {
    Fanout,
    FanoutClient,
    MembershipModel,
  } from "../../../metaplex-program-library/hydra/js/src";
import { Keypair, Connection, PublicKey, Transaction, SystemProgram, Account } from '@solana/web3.js';
import {NodeWallet} from '@project-serum/common' //TODO: remove iykyk
import fs from 'fs'
import bs58 from 'bs58'
const name = new Date().getTime().toString() 
export const findRuleSetPDA = async (payer: PublicKey, name: string) => {
    return await PublicKey.findProgramAddress(
        [
            Buffer.from(PREFIX),
            payer.toBuffer(),
            Buffer.from(name),
        ],
        PROGRAM_ID,
    );
}

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
// @ts-ignore
const connection = new Connection(process.env.SOLANA_RPC, "finalized");
let payer = Keypair.fromSecretKey(new Uint8Array( JSON.parse(fs.readFileSync('/home/gitpod/.config/solana/id.json').toString())))
let authorityWallet: Keypair;
let fanoutSdk: FanoutClient;
  authorityWallet = payer;
  //await airdrop(connection, authorityWallet.publicKey, LAMPORTS_PER_SOL * 10);
  fanoutSdk = new FanoutClient(
    connection,
    // @ts-ignore
    new NodeWallet(payer)
  );

setTimeout(async function(){
/*
    const { fanout: nfteeze } = await fanoutSdk.initializeFanout({
        totalShares: 100000,
        name: name,
        membershipModel: MembershipModel.NFT,
      });  */
      const nfteeze = new PublicKey("ADih34mBjvzp5kw6nLvfX5pVdnYAcxGy8arJ4qdGojqW")
       const nfteezeFanout = await fanoutSdk.fetch<Fanout>(nfteeze, Fanout);
      let [holdingAccount, bump] = await FanoutClient.nativeAccount(nfteeze)
      console.log('nfteeze fanout: ' + nfteeze.toBase58())
      console.log('nfteeze fanout sol hodling account: ' + holdingAccount.toBase58())
      console.log('nfteeze fanout many details: ' + nfteezeFanout)
      console.log('lol')
      console.log(bs58.decode (holdingAccount.toBase58()))
      console.log(nfteezeFanout)
 /* await fanoutSdk.initializeFanoutForMint({
    fanout: nfteeze,
    mint: new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), //USDC if y'all weak
  }); */
// Encode the file using msgpack so the pre-encoded data can be written directly to a Solana program account
// todo how to do this properly sers ie how pubkey to uint8array
const encoded = encode(JSON.parse(fs.readFileSync("./cli/examples/pubkey_match.json").toString()));

// Create the ruleset
await createTokenAuthorizationRules(connection, payer, name, encoded);
})