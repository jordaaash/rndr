import { createSetEscrowOwnerInstruction } from 'rndr';
import { OWNER_KEYPAIR, OWNER_PUBKEY } from '../config';
import { sendTransaction } from '../util';

export const setEscrowOwner = async (): Promise<string> => {
    const setEscrowOwner = await createSetEscrowOwnerInstruction(OWNER_PUBKEY, OWNER_PUBKEY);
    return await sendTransaction([setEscrowOwner], [OWNER_KEYPAIR]);
};
