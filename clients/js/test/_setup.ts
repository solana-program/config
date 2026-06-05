import path from 'node:path';

import { createClient, lamports } from '@solana/kit';
import { litesvm } from '@solana/kit-plugin-litesvm';
import { airdropSigner, generatedSigner } from '@solana/kit-plugin-signer';

import { SOLANA_CONFIG_PROGRAM_ADDRESS, solanaConfigProgram } from '../src';

const SOLANA_CONFIG_BINARY_PATH = path.resolve(
    __dirname,
    '..',
    '..',
    '..',
    'target',
    'deploy',
    'solana_config_program.so',
);

export const createTestClient = () => {
    return createClient()
        .use(generatedSigner())
        .use(litesvm())
        .use(airdropSigner(lamports(1_000_000_000n)))
        .use(client => {
            // Load the config program into the LiteSVM instance from its
            // compiled `.so` file. This must run after the `litesvm()` plugin
            // so that `client.svm` is available.
            client.svm.addProgramFromFile(SOLANA_CONFIG_PROGRAM_ADDRESS, SOLANA_CONFIG_BINARY_PATH);
            return client;
        })
        .use(solanaConfigProgram());
};
