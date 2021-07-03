import { AccountInfo, PublicKey } from '@solana/web3.js';
import { struct, u8 } from 'buffer-layout';
import { Parser, publicKey, u64 } from '../util';
import { AccountType } from './accountType';

export interface Job {
    accountType: AccountType;
    amount: bigint;
    authority: PublicKey;
}

/** @internal */
export const JobLayout = struct<Job>([u8('accountType'), u64('amount'), publicKey('authority')]);

export const JOB_SIZE = JobLayout.span;

export const isJob = (info: AccountInfo<Buffer>): boolean => {
    return info.data.length === JOB_SIZE && info.data[0] === AccountType.JobV1;
};

export const parseJob: Parser<Job> = (pubkey: PublicKey, info: AccountInfo<Buffer>) => {
    if (!isJob(info)) return;
    const data = JobLayout.decode(info.data);
    return {
        pubkey,
        info,
        data,
    };
};
