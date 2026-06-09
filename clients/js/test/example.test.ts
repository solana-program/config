import { expect, it } from 'vitest';

import { createTestClient } from './_setup';

it('sets up a LiteSVM client with the config program', async () => {
    // Given a test client whose payer is funded with SOL.
    const client = await createTestClient();

    // Then the client exposes the config program plugin.
    expect(client.config).toBeDefined();

    // And the payer was funded via LiteSVM.
    const { value: balance } = await client.rpc.getBalance(client.payer.address).send();
    expect(balance).toBe(1_000_000_000n);
});
