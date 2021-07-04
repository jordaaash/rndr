import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { PublicKey } from '@solana/web3.js';
import { RNDR_PROGRAM_ID, RNDR_TOKEN_MINT } from '../constants';

export const findEscrowAddress = async (tokenMint: PublicKey = RNDR_TOKEN_MINT): Promise<[PublicKey, number]> => {
    return await PublicKey.findProgramAddress(
        [Buffer.from('escrow', 'utf8'), tokenMint.toBuffer(), TOKEN_PROGRAM_ID.toBuffer()],
        RNDR_PROGRAM_ID
    );
};

export const findEscrowAssociatedTokenAddress = async (
    escrow: PublicKey,
    tokenMint: PublicKey = RNDR_TOKEN_MINT
): Promise<[PublicKey, number]> => {
    return await PublicKey.findProgramAddress(
        [escrow.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), tokenMint.toBuffer()],
        ASSOCIATED_TOKEN_PROGRAM_ID
    );
};

export const findJobAddress = async (escrow: PublicKey, authority: PublicKey): Promise<[PublicKey, number]> => {
    return await PublicKey.findProgramAddress(
        [Buffer.from('job', 'utf8'), escrow.toBuffer(), authority.toBuffer()],
        RNDR_PROGRAM_ID
    );
};
