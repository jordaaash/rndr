import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { PublicKey } from '@solana/web3.js';
import { createFundJobInstruction, RNDR_TOKEN_MINT } from 'rndr';
import { AUTHORITY_KEYPAIR, AUTHORITY_PUBKEY } from '../config';
import { sendTransaction } from '../util';

export const fundJob = async (amount: number | bigint): Promise<string> => {
    const [sourceTokenPubkey] = await PublicKey.findProgramAddress(
        [AUTHORITY_PUBKEY.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), RNDR_TOKEN_MINT.toBuffer()],
        ASSOCIATED_TOKEN_PROGRAM_ID
    );
    const fundJob = await createFundJobInstruction(amount, AUTHORITY_PUBKEY, sourceTokenPubkey, AUTHORITY_PUBKEY);
    return await sendTransaction([fundJob], [AUTHORITY_KEYPAIR]);
};
