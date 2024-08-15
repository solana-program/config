#!/usr/bin/env zx
import 'zx/globals';
import * as k from "kinobi";
import { renderVisitor as renderJavaScriptVisitor } from '@kinobi-so/renderers-js';
import { renderVisitor as renderRustVisitor } from '@kinobi-so/renderers-rust';
import { getToolchainArgument, workingDirectory } from './utils.mjs';

// Instanciate Kinobi.
const kinobi = k.createFromRoot(
  require(path.join(workingDirectory, 'program', 'idl.json'))
);

// Add missing types from the IDL.
kinobi.update(
  k.bottomUpTransformerVisitor([
    {
      select: (node) => {
        const names = ["keys"];
        return (
          names.includes(node.name) &&
          (k.isNode(node, "instructionArgumentNode") || k.isNode(node, "structFieldTypeNode")) &&
          k.isNode(node.type, "arrayTypeNode")
        );
      },
      transform: (node) => {
        return {
          ...node,
          type: k.definedTypeLinkNode("configKeys", "hooked"),
        };
      },
    },
  ])
);

// Render JavaScript.
const jsClient = path.join(__dirname, '..', 'clients', 'js');
kinobi.accept(
  renderJavaScriptVisitor(path.join(jsClient, 'src', 'generated'), {
    prettier: require(path.join(jsClient, '.prettierrc.json')),
  })
);

// Render Rust.
const rustClient = path.join(__dirname, '..', 'clients', 'rust');
kinobi.accept(
  renderRustVisitor(path.join(rustClient, 'src', 'generated'), {
    formatCode: true,
    crateFolder: rustClient,
    toolchain: getToolchainArgument('format'),
  })
);