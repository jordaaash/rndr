import { PublicKey, TransactionInstruction } from '@solana/web3.js';
import { struct, u8 } from 'buffer-layout';
import { RNDR_PROGRAM_ID, RNDR_TOKEN_MINT } from '../constants';
import { findEscrowAddress, publicKey } from '../util';
import { RNDRInstruction } from './instruction';

interface Data {
    instruction: number;
    newOwner: PublicKey;
}

const DataLayout = struct<Data>([u8('instruction'), publicKey('newOwner')]);

export const createSetEscrowOwnerInstruction = async (
    newOwner: PublicKey,
    currentOwner: PublicKey
): Promise<TransactionInstruction> => {
    const [escrow] = await findEscrowAddress(RNDR_TOKEN_MINT);
    return setEscrowOwnerInstruction(newOwner, escrow, currentOwner);
};

export const setEscrowOwnerInstruction = (
    newOwner: PublicKey,
    escrow: PublicKey,
    currentOwner: PublicKey
): TransactionInstruction => {
    const data = Buffer.alloc(DataLayout.span);
    DataLayout.encode(
        {
            instruction: RNDRInstruction.SetEscrowOwner,
            newOwner,
        },
        data
    );

    const keys = [
        { pubkey: escrow, isSigner: false, isWritable: true },
        { pubkey: currentOwner, isSigner: true, isWritable: false },
    ];

    return new TransactionInstruction({
        keys,
        programId: RNDR_PROGRAM_ID,
        data,
    });
};
