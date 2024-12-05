#!/usr/bin/env zx

// Firedancer conformance testing of the Core BPF Config program against its
// original builtin implementation.
//
// Note: This script can only be run on Ubuntu.

import 'zx/globals';
import { getProgramId, getProgramSharedObjectPath, workingDirectory } from '../utils.mjs';

// Clone the conformance harness.
const harnessPath = path.join(workingDirectory, 'solana-conformance');
await $`git clone https://github.com/firedancer-io/solana-conformance.git`;

// Clone the test vectors.
const testVectorsPath = path.join(harnessPath, 'impl', 'test-vectors');
await $`git clone https://github.com/firedancer-io/test-vectors.git ${testVectorsPath}`;

// Add the Mollusk-generated fixtures to the test inputs.
const firedancerFixturesPath = path.join(testVectorsPath, 'instr', 'fixtures', 'config');
const molluskFixturesPath = path.join(workingDirectory, 'program', 'fuzz', 'blob');
await $`cp -a ${molluskFixturesPath}/. ${firedancerFixturesPath}/`;

// Clone the SolFuzz-Agave harness.
const solFuzzAgavePath = path.join(harnessPath, 'impl', 'solfuzz-agave');
await $`git clone -b agave-v2.1.3 http://github.com/firedancer-io/solfuzz-agave.git ${solFuzzAgavePath}`;

// Fetch protobuf files.
await $`make -j -C ${solFuzzAgavePath} fetch_proto`

// Move into the conformance harness.
cd(harnessPath);

// Build the environment.
await $`bash install_ubuntu_lite.sh`;

const solFuzzAgaveManifestPath = path.join(solFuzzAgavePath, 'Cargo.toml');
const solFuzzAgaveTargetPath = path.join(
    solFuzzAgavePath,
    'target',
    'x86_64-unknown-linux-gnu',
    'release',
    'libsolfuzz_agave.so',
);

const testTargetsDir = path.join(harnessPath, 'impl', 'lib');
await $`mkdir -p ${testTargetsDir}`;

// Build the Agave target with the builtin version.
const testTargetPathBuiltin = path.join(testTargetsDir, 'builtin.so');
await $`cargo build --manifest-path ${solFuzzAgaveManifestPath} \
        --lib --release --target x86_64-unknown-linux-gnu`;
await $`mv ${solFuzzAgaveTargetPath} ${testTargetPathBuiltin}`;

// Build the Agave target with the BPF version and special-casing enabled for
// conformance testing (`core-bpf-conformance` features).
const testTargetPathCoreBpf = path.join(testTargetsDir, 'core_bpf.so');
await $`CORE_BPF_PROGRAM_ID=${getProgramId('program')} \
        CORE_BPF_TARGET=${getProgramSharedObjectPath('program')} \
        FORCE_RECOMPILE=true \
        cargo build --manifest-path ${solFuzzAgaveManifestPath} \
        --lib --release --target x86_64-unknown-linux-gnu \
        --features core-bpf-conformance`;
await $`mv ${solFuzzAgaveTargetPath} ${testTargetPathCoreBpf}`;

// Remove any test results if they exist.
await $`rm -rf test_results`;

// Run the tests.
await $`source test_suite_env/bin/activate && \
        solana-test-suite run-tests \
        -i ${firedancerFixturesPath} -s ${testTargetPathBuiltin} -t ${testTargetPathCoreBpf} \
        --consensus-mode --save-failures`;

// Assert conformance.
// There should be no fixtures in the `failed_protobufs` directory.
if (fs.existsSync('test_results/failed_protobufs')) {
    if (fs.readdirSync('test_results/failed_protobufs').length > 0) {
        throw new Error(`Error: mismatches detected.`);
    }
}

console.log('All Firedancer conformance tests passed.');
