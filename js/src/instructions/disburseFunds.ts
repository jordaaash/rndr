import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { PublicKey, TransactionInstruction } from '@solana/web3.js';
import { struct, u8 } from 'buffer-layout';
import { RNDR_PROGRAM_ID, RNDR_TOKEN_MINT } from '../constants';
import { findEscrowAddress, findEscrowAssociatedTokenAddress, findJobAddress, u64 } from '../util';
import { RNDRInstruction } from './instruction';

interface Data {
    instruction: number;
    amount: bigint;
}

const DataLayout = struct<Data>([u8('instruction'), u64('amount')]);

export const createDisburseFundsInstruction = async (
    amount: number | bigint,
    owner: PublicKey,
    destinationToken: PublicKey,
    authority: PublicKey
): Promise<TransactionInstruction> => {
    const [escrow] = await findEscrowAddress(RNDR_TOKEN_MINT);
    const [escrowAssociatedToken] = await findEscrowAssociatedTokenAddress(escrow, RNDR_TOKEN_MINT);
    const [job] = await findJobAddress(escrow, authority);
    return disburseFunds(amount, RNDR_TOKEN_MINT, escrow, owner, escrowAssociatedToken, job, destinationToken);
};

export const disburseFunds = (
    amount: number | bigint,
    tokenMint: PublicKey,
    escrow: PublicKey,
    owner: PublicKey,
    escrowAssociatedToken: PublicKey,
    job: PublicKey,
    destinationToken: PublicKey
): TransactionInstruction => {
    const data = Buffer.alloc(DataLayout.span);
    DataLayout.encode(
        {
            instruction: RNDRInstruction.DisburseFunds,
            amount: BigInt(amount),
        },
        data
    );

    const keys = [
        { pubkey: tokenMint, isSigner: false, isWritable: false },
        { pubkey: escrow, isSigner: false, isWritable: true },
        { pubkey: owner, isSigner: true, isWritable: false },
        { pubkey: escrowAssociatedToken, isSigner: false, isWritable: true },
        { pubkey: job, isSigner: false, isWritable: true },
        { pubkey: destinationToken, isSigner: false, isWritable: true },
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ];

    return new TransactionInstruction({
        keys,
        programId: RNDR_PROGRAM_ID,
        data,
    });
};
