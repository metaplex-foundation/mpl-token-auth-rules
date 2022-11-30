import { encode, decode } from '@msgpack/msgpack';
import { createTokenAuthorizationRules } from './helpers/tar';
import { Keypair, Connection, LAMPORTS_PER_SOL, PublicKey } from '@solana/web3.js';

const encoded: Uint8Array = encode({ foo: 'bar' });
console.log(encoded);
const decoded: any = decode(encoded);
console.log(decoded);

const ruleset = [
    {
        "Transfer": {
            "All": [
                [
                    {
                        "All": [
                            [
                                {
                                    "AdditionalSigner": [
                                        PublicKey.decode(Buffer.from([42, 157, 245, 156, 21, 37, 147, 96, 183, 190, 206, 14, 24, 1, 106, 49, 167, 236, 38, 73, 98, 53, 60, 9, 154, 164, 240, 126, 210, 197, 76, 235]))
                                    ]
                                },
                                {
                                    "AdditionalSigner": [
                                        [42, 157, 245, 156, 21, 37, 147, 96, 183, 190, 206, 14, 24, 1, 106, 49, 167, 236, 38, 73, 98, 53, 60, 9, 154, 164, 240, 126, 210, 197, 76, 235]
                                    ]
                                }
                            ]
                        ]
                    },
                    {
                        "Amount": [1]
                    }
                ]
            ]
        }
    }
]

async function main() {
    const connection = new Connection("http://localhost:8899", "finalized");
    const payer = Keypair.generate();

    const airdropSignature = await connection.requestAirdrop(
        payer.publicKey,
        10 * LAMPORTS_PER_SOL
    );

    await connection.confirmTransaction(airdropSignature);

    const encoded = encode(ruleset);
    let rulesetAddress = await createTokenAuthorizationRules(connection, payer, 'test', encoded);
    console.log(JSON.stringify(rulesetAddress/*, null, 2*/));
    let rulesetData = await connection.getAccountInfo(rulesetAddress);
    console.log(JSON.stringify(rulesetData?.data/*, null, 2*/));
    let rulesetDecoded = decode(rulesetData?.data);
    console.log(JSON.stringify(rulesetDecoded/*, null, 2*/));
}

main().then(() => console.log('done')).catch(e => console.error(e));