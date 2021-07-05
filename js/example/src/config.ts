import { clusterApiUrl, Connection, Keypair } from '@solana/web3.js';
import authoritySecretKey from './keys/authority.json';
import ownerSecretKey from './keys/owner.json';

export const OWNER_KEYPAIR = Keypair.fromSecretKey(Uint8Array.from(ownerSecretKey));
export const OWNER_PUBKEY = OWNER_KEYPAIR.publicKey;

export const AUTHORITY_KEYPAIR = Keypair.fromSecretKey(Uint8Array.from(authoritySecretKey));
export const AUTHORITY_PUBKEY = AUTHORITY_KEYPAIR.publicKey;

export const CLUSTER = 'devnet';
export const CONNECTION = new Connection(clusterApiUrl(CLUSTER));

export const PORT = 3000;
