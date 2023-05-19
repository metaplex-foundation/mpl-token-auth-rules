import { LOCALHOST } from '@metaplex-foundation/amman';
import { Connection, Keypair } from '@solana/web3.js';
import { amman } from './_amman';

export const getConnectionAndPayer = async () => {
  const payer = Keypair.generate();
  const connection = new Connection(LOCALHOST, 'confirmed');
  await amman.airdrop(connection, payer.publicKey, 20);
  const transactionHandler = amman.payerTransactionHandler(connection, payer);

  return {
    fstTxHandler: transactionHandler,
    connection,
    payer,
  };
};
