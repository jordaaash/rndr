import { Signer, Transaction, TransactionInstruction } from '@solana/web3.js';
import { CLUSTER, CONNECTION } from './config';

export const sendTransaction = async (instructions: TransactionInstruction[], signers: Signer[]): Promise<string> => {
    const { blockhash } = await CONNECTION.getRecentBlockhash('max');

    const tx = new Transaction({ recentBlockhash: blockhash });
    tx.add(...instructions);

    const signature = await CONNECTION.sendTransaction(tx, signers, {
        skipPreflight: true,
        preflightCommitment: 'singleGossip',
    });

    await CONNECTION.confirmTransaction(signature, 'singleGossip');

    return signature;
};

export const getExplorerLink = (signature: string) => `https://explorer.solana.com/tx/${signature}?cluster=${CLUSTER}`;
