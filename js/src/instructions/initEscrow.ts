import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, TransactionInstruction } from '@solana/web3.js';
import { struct, u8 } from 'buffer-layout';
import { RNDR_PROGRAM_ID, RNDR_TOKEN_MINT } from '../constants';
import { findEscrowAddress, findEscrowAssociatedTokenAddress, publicKey } from '../util';
import { RNDRInstruction } from './instruction';

interface Data {
    instruction: number;
    owner: PublicKey;
}

const DataLayout = struct<Data>([u8('instruction'), publicKey('owner')]);

export const createInitEscrowInstruction = async (
    owner: PublicKey,
    funder: PublicKey
): Promise<TransactionInstruction> => {
    const [escrow] = await findEscrowAddress(RNDR_TOKEN_MINT);
    const [escrowAssociatedToken] = await findEscrowAssociatedTokenAddress(escrow, RNDR_TOKEN_MINT);
    return initEscrowInstruction(owner, RNDR_TOKEN_MINT, funder, escrow, escrowAssociatedToken);
};

export const initEscrowInstruction = (
    owner: PublicKey,
    tokenMint: PublicKey,
    funder: PublicKey,
    escrow: PublicKey,
    escrowAssociatedToken: PublicKey
): TransactionInstruction => {
    const data = Buffer.alloc(DataLayout.span);
    DataLayout.encode(
        {
            instruction: RNDRInstruction.InitEscrow,
            owner,
        },
        data
    );

    const keys = [
        { pubkey: tokenMint, isSigner: false, isWritable: false },
        { pubkey: funder, isSigner: true, isWritable: true },
        { pubkey: escrow, isSigner: false, isWritable: true },
        { pubkey: escrowAssociatedToken, isSigner: false, isWritable: true },
        { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
        { pubkey: ASSOCIATED_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ];

    return new TransactionInstruction({
        keys,
        programId: RNDR_PROGRAM_ID,
        data,
    });
};
