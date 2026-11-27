import { AccountRole, generateKeyPairSigner, lamports } from '@solana/kit';
import { expect, it } from 'vitest';

import {
    CONFIG_ERROR__READONLY_DATA_MODIFIED,
    CONFIG_PROGRAM_ADDRESS,
    getConfigErrorMessage,
    getStoreInstruction,
    isConfigError,
} from '../src';
import { createTestClient } from './_setup';

it('recognises the error returned for a read-only config account', async () => {
    // Given an empty config account.
    const client = await createTestClient();
    const config = await generateKeyPairSigner();
    const data = new Uint8Array([1, 2, 3]);
    client.svm.setAccount({
        address: config.address,
        data: new Uint8Array(1 + data.length),
        executable: false,
        lamports: lamports(1_000_000_000n),
        programAddress: CONFIG_PROGRAM_ADDRESS,
        space: BigInt(1 + data.length),
    });

    // When we store data with the config account passed as read-only.
    const store = getStoreInstruction({ configAccount: config, keys: [], data });
    const instruction = { ...store, accounts: [{ ...store.accounts[0], role: AccountRole.READONLY_SIGNER }] };
    const error = await client.sendTransaction([instruction]).catch((e: unknown) => e);

    // Then the program error is recognised as `ReadonlyDataModified`.
    const cause = (error as Error).cause;
    expect(isConfigError(cause, { instructions: [instruction] }, CONFIG_ERROR__READONLY_DATA_MODIFIED)).toBe(true);
    expect(getConfigErrorMessage(CONFIG_ERROR__READONLY_DATA_MODIFIED)).toBe(
        'Instruction modified data of a read-only account',
    );
});
