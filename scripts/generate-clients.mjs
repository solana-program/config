#!/usr/bin/env zx
import 'zx/globals';
import * as c from 'codama';
import { renderVisitor as renderJavaScriptVisitor } from '@codama/renderers-js';
import { renderVisitor as renderRustVisitor } from '@codama/renderers-rust';
import { getToolchainArgument, workingDirectory } from './utils.mjs';

// Instanciate Codama.
const codama = c.createFromRoot(
  require(path.join(workingDirectory, 'program', 'idl.json'))
);

// Update programs.
codama.update(
  c.updateProgramsVisitor({
    solanaConfigProgram: { name: 'solanaConfig' },
  })
);

// Add missing types from the IDL.
codama.update(
  c.bottomUpTransformerVisitor([
    {
      select: (node) => {
        const names = ['keys'];
        return (
          names.includes(node.name) &&
          (c.isNode(node, 'instructionArgumentNode') ||
            c.isNode(node, 'structFieldTypeNode')) &&
          c.isNode(node.type, 'arrayTypeNode')
        );
      },
      transform: (node) => {
        return {
          ...node,
          type: c.definedTypeLinkNode('configKeys'),
        };
      },
    },
  ])
);

// Render JavaScript.
const jsClient = path.join(__dirname, '..', 'clients', 'js');
codama.accept(
  renderJavaScriptVisitor(path.join(jsClient, 'src', 'generated'), {
    prettier: require(path.join(jsClient, '.prettierrc.json')),
  })
);

// Render Rust.
const rustClient = path.join(__dirname, '..', 'clients', 'rust');
codama.accept(
  renderRustVisitor(path.join(rustClient, 'src', 'generated'), {
    formatCode: true,
    crateFolder: rustClient,
    toolchain: getToolchainArgument('format'),
  })
);
