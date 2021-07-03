import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { PublicKey } from '@solana/web3.js';
import { RNDR_PROGRAM_ID, RNDR_TOKEN_ACCOUNT } from '../constants';

export const findEscrowAddress = async (
    tokenAccount: PublicKey = RNDR_TOKEN_ACCOUNT,
    tokenProgramId: PublicKey = TOKEN_PROGRAM_ID
): Promise<[PublicKey, number]> => {
    return await PublicKey.findProgramAddress(
        [Buffer.from('escrow', 'utf8'), tokenAccount.toBuffer(), tokenProgramId.toBuffer()],
        RNDR_PROGRAM_ID
    );
};

export const findJobAddress = async (authority: PublicKey, escrow?: PublicKey): Promise<[PublicKey, number]> => {
    if (!escrow) {
        [escrow] = await findEscrowAddress();
    }
    return await PublicKey.findProgramAddress(
        [Buffer.from('escrow', 'utf8'), escrow.toBuffer(), authority.toBuffer()],
        RNDR_PROGRAM_ID
    );
};
