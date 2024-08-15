/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import { type Address } from '@solana/web3.js';
import { type ParsedStoreInstruction } from '../instructions';

export const CONFIG_PROGRAM_ADDRESS =
  'Config1111111111111111111111111111111111111' as Address<'Config1111111111111111111111111111111111111'>;

export enum ConfigAccount {
  Config,
}

export enum ConfigInstruction {
  Store,
}

export type ParsedConfigInstruction<
  TProgram extends string = 'Config1111111111111111111111111111111111111',
> = {
  instructionType: ConfigInstruction.Store;
} & ParsedStoreInstruction<TProgram>;
