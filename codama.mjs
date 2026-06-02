import path from 'node:path';
import { config } from 'dotenv';

config({ path: path.resolve(process.cwd(), 'vars.env') });

export default {
    idl: 'program/idl.json',
    before: [],
    scripts: {
        js: {
            from: '@codama/renderers-js',
            args: [
                'clients/js',
                {
                    kitImportStrategy: 'rootOnly',
                    syncPackageJson: true,
                },
            ],
        },
        rust: [
            {
                from: '@codama/visitors-core#deleteNodesVisitor',
                args: [['[definedTypeNode]configKeys']],
            },
            {
                from: '@codama/renderers-rust',
                args: [
                    'clients/rust',
                    {
                        anchorTraits: false,
                        formatCode: true,
                        linkOverrides: {
                            definedTypes: { configKeys: 'hooked' },
                        },
                        toolchain: `+${process.env.RUST_TOOLCHAIN_NIGHTLY}`,
                    },
                ],
            },
        ],
    },
};
