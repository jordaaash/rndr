import { createInitEscrowInstruction } from 'rndr';
import { OWNER_KEYPAIR, OWNER_PUBKEY } from '../config';
import { sendTransaction } from '../util';

export const initEscrow = async (): Promise<string> => {
    const initEscrow = await createInitEscrowInstruction(OWNER_PUBKEY, OWNER_PUBKEY);
    return await sendTransaction([initEscrow], [OWNER_KEYPAIR]);
};
