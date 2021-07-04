import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, TransactionInstruction } from '@solana/web3.js';
import { struct, u8 } from 'buffer-layout';
import { RNDR_PROGRAM_ID, RNDR_TOKEN_MINT } from '../constants';
import { findEscrowAddress, findEscrowAssociatedTokenAddress, findJobAddress, u64 } from '../util';
import { RNDRInstruction } from './instruction';

interface Data {
    instruction: number;
    amount: bigint;
}

const DataLayout = struct<Data>([u8('instruction'), u64('amount')]);

export const createFundJobInstruction = async (
    amount: number | bigint,
    funder: PublicKey,
    sourceToken: PublicKey,
    authority: PublicKey
): Promise<TransactionInstruction> => {
    const [escrow] = await findEscrowAddress(RNDR_TOKEN_MINT);
    const [escrowAssociatedToken] = await findEscrowAssociatedTokenAddress(escrow, RNDR_TOKEN_MINT);
    const [job] = await findJobAddress(escrow, authority);
    return fundJobInstruction(
        amount,
        RNDR_TOKEN_MINT,
        funder,
        sourceToken,
        authority,
        escrow,
        escrowAssociatedToken,
        job
    );
};

export const fundJobInstruction = (
    amount: number | bigint,
    tokenMint: PublicKey,
    funder: PublicKey,
    sourceToken: PublicKey,
    authority: PublicKey,
    escrow: PublicKey,
    escrowAssociatedToken: PublicKey,
    job: PublicKey
): TransactionInstruction => {
    const data = Buffer.alloc(DataLayout.span);
    DataLayout.encode(
        {
            instruction: RNDRInstruction.FundJob,
            amount: BigInt(amount),
        },
        data
    );

    const keys = [
        { pubkey: tokenMint, isSigner: false, isWritable: false },
        { pubkey: funder, isSigner: true, isWritable: true },
        { pubkey: sourceToken, isSigner: false, isWritable: true },
        { pubkey: authority, isSigner: true, isWritable: false },
        { pubkey: escrow, isSigner: false, isWritable: true },
        { pubkey: escrowAssociatedToken, isSigner: false, isWritable: true },
        { pubkey: job, isSigner: false, isWritable: true },
        { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ];

    return new TransactionInstruction({
        keys,
        programId: RNDR_PROGRAM_ID,
        data,
    });
};
