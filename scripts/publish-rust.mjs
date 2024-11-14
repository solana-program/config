#!/usr/bin/env zx
import 'zx/globals';
import { getCargo, workingDirectory } from './utils.mjs';

class Publisher {
  libType;

  constructor() {
    const lib = process.argv[3];
    if (!lib) {
      throw new Error('A library type must be provided.');
    }
    if (lib !== 'client' && lib !== 'program') {
      throw new Error('Invalid library type. Allowed values are "client" or "program".');
    }
    this.libType = lib;
  }

  path() {
    if (this.libType === 'client') {
      return 'clients/rust';
    } else {
      return 'program';
    }
  }

  message() {
    if (this.libType === 'client') {
      return 'Rust client';
    } else {
      return 'Program';
    }
  }

  tagPrefix() {
    if (this.libType === 'client') {
      return 'rust';
    } else {
      return 'program';
    }
  }
}

const publisher = new Publisher();
const dryRun = argv['dry-run'] ?? false;
const [level] = process.argv.slice(4);
if (!level) {
  throw new Error('A version level — e.g. "path" — must be provided.');
}

// Go to the directory and install the dependencies.
cd(path.join(workingDirectory, publisher.path()));

// Publish the new version.
const releaseArgs = dryRun
  ? []
  : ['--no-push', '--no-tag', '--no-confirm', '--execute'];
await $`cargo release ${level} ${releaseArgs}`;

// Stop here if this is a dry run.
if (dryRun) {
  process.exit(0);
}

// Get the new version.
const newVersion = getCargo(publisher.path()).package.version;

// Expose the new version to CI if needed.
if (process.env.CI) {
  await $`echo "new_version=${newVersion}" >> $GITHUB_OUTPUT`;
}

// Soft reset the last commit so we can create our own commit and tag.
await $`git reset --soft HEAD~1`;

// Commit the new version.
await $`git commit -am "Publish ${publisher.message()} v${newVersion}"`;

// Tag the new version.
await $`git tag -a ${publisher.tagPrefix()}@v${newVersion} -m "${publisher.message()} v${newVersion}"`;
