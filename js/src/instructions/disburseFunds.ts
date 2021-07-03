import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { PublicKey, TransactionInstruction } from '@solana/web3.js';
import { struct, u8 } from 'buffer-layout';
import { RNDR_PROGRAM_ID } from '../constants';
import { u64 } from '../util';
import { RNDRInstruction } from './instruction';

interface Data {
    instruction: number;
    amount: bigint;
}

const DataLayout = struct<Data>([u8('instruction'), u64('amount')]);

export const disburseFunds = (
    amount: number | bigint,
    sourceTokenAccount: PublicKey,
    destinationTokenAccount: PublicKey,
    escrow: PublicKey,
    job: PublicKey,
    owner: PublicKey
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
        { pubkey: sourceTokenAccount, isSigner: false, isWritable: true },
        { pubkey: destinationTokenAccount, isSigner: false, isWritable: true },
        { pubkey: escrow, isSigner: false, isWritable: true },
        { pubkey: job, isSigner: false, isWritable: true },
        { pubkey: owner, isSigner: true, isWritable: false },
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ];

    return new TransactionInstruction({
        keys,
        programId: RNDR_PROGRAM_ID,
        data,
    });
};
