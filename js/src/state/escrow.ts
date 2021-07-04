import { AccountInfo, PublicKey } from '@solana/web3.js';
import { struct, u8 } from 'buffer-layout';
import { Parser, publicKey, u64 } from '../util';
import { AccountType } from './accountType';

export interface Escrow {
    accountType: AccountType;
    amount: bigint;
    owner: PublicKey;
}

/** @internal */
export const EscrowLayout = struct<Escrow>([u8('accountType'), u64('amount'), publicKey('owner')]);

export const ESCROW_SIZE = EscrowLayout.span;

export const isEscrow = (info: AccountInfo<Buffer>): boolean => {
    return info.data.length === ESCROW_SIZE && info.data.readUIntLE(0, 1) === AccountType.EscrowV1;
};

export const parseEscrow: Parser<Escrow> = (pubkey: PublicKey, info: AccountInfo<Buffer>) => {
    if (!isEscrow(info)) return;
    const data = EscrowLayout.decode(info.data);
    return {
        pubkey,
        info,
        data,
    };
};
