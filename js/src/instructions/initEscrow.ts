import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { PublicKey, SYSVAR_RENT_PUBKEY, TransactionInstruction } from '@solana/web3.js';
import { struct, u8 } from 'buffer-layout';
import { RNDR_PROGRAM_ID } from '../constants';
import { publicKey } from '../util';
import { RNDRInstruction } from './instruction';

interface Data {
    instruction: number;
    owner: PublicKey;
}

const DataLayout = struct<Data>([u8('instruction'), publicKey('owner')]);

export const initEscrowInstruction = (
    owner: PublicKey,
    escrow: PublicKey,
    tokenAccount: PublicKey,
    tokenMint: PublicKey
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
        { pubkey: escrow, isSigner: false, isWritable: true },
        { pubkey: tokenAccount, isSigner: false, isWritable: true },
        { pubkey: tokenMint, isSigner: false, isWritable: false },
        { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ];

    return new TransactionInstruction({
        keys,
        programId: RNDR_PROGRAM_ID,
        data,
    });
};
