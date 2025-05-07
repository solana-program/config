import path from 'node:path';
import { config } from 'dotenv';

config({ path: path.resolve(process.cwd(), 'vars.env') });

const { default: prettierOptions } = await import(
  path.resolve('clients', 'js', '.prettierrc.json'),
  { with: { type: 'json' } }
);

export default {
  idl: 'program/idl.json',
  before: [
    {
      from: '@codama/renderers-js',
      args: ['clients/js/src/generated', { prettierOptions }],
    },
    {
      from: '@codama/visitors-core#deleteNodesVisitor',
      args: [['[definedTypeNode]configKeys']],
    },
  ],
  scripts: {
    rust: {
      from: '@codama/renderers-rust',
      args: [
        'clients/rust/src/generated',
        {
          anchorTraits: false,
          crateFolder: 'clients/rust',
          formatCode: true,
          linkOverrides: {
            definedTypes: {
              configKeys: 'hooked',
            },
          },
          toolchain: `+${process.env.RUST_TOOLCHAIN_NIGHTLY}`,
        },
      ],
    },
  },
};