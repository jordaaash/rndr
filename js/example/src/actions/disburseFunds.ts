import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { PublicKey } from '@solana/web3.js';
import { createDisburseFundsInstruction, RNDR_TOKEN_MINT } from 'rndr';
import { AUTHORITY_PUBKEY, OWNER_KEYPAIR, OWNER_PUBKEY } from '../config';
import { sendTransaction } from '../util';

export const disburseFunds = async (amount: number | bigint): Promise<string> => {
    const [destinationTokenPubkey] = await PublicKey.findProgramAddress(
        [AUTHORITY_PUBKEY.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), RNDR_TOKEN_MINT.toBuffer()],
        ASSOCIATED_TOKEN_PROGRAM_ID
    );
    const disburseFunds = await createDisburseFundsInstruction(
        amount,
        OWNER_PUBKEY,
        destinationTokenPubkey,
        AUTHORITY_PUBKEY
    );
    return await sendTransaction([disburseFunds], [OWNER_KEYPAIR]);
};
