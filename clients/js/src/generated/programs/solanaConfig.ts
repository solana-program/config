/**
 * This code was AUTOGENERATED using the codama library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun codama to update it.
 *
 * @see https://github.com/codama-idl/codama
 */

import { type Address } from '@solana/kit';
import { type ParsedStoreInstruction } from '../instructions';

export const SOLANA_CONFIG_PROGRAM_ADDRESS =
  'Config1111111111111111111111111111111111111' as Address<'Config1111111111111111111111111111111111111'>;

export enum SolanaConfigAccount {
  Config,
}

export enum SolanaConfigInstruction {
  Store,
}

export type ParsedSolanaConfigInstruction<
  TProgram extends string = 'Config1111111111111111111111111111111111111',
> = {
  instructionType: SolanaConfigInstruction.Store;
} & ParsedStoreInstruction<TProgram>;
