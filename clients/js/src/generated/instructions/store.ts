/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import {
  AccountRole,
  combineCodec,
  getBytesDecoder,
  getBytesEncoder,
  getStructDecoder,
  getStructEncoder,
  type Address,
  type Codec,
  type Decoder,
  type Encoder,
  type IAccountMeta,
  type IAccountSignerMeta,
  type IInstruction,
  type IInstructionWithAccounts,
  type IInstructionWithData,
  type ReadonlyUint8Array,
  type TransactionSigner,
  type WritableAccount,
  type WritableSignerAccount,
} from '@solana/web3.js';
import {
  getConfigKeysDecoder,
  getConfigKeysEncoder,
  type ConfigKeys,
  type ConfigKeysArgs,
} from '../../hooked';
import { CONFIG_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';

export type StoreInstruction<
  TProgram extends string = typeof CONFIG_PROGRAM_ADDRESS,
  TAccountConfigAccount extends string | IAccountMeta<string> = string,
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountConfigAccount extends string
        ? WritableAccount<TAccountConfigAccount>
        : TAccountConfigAccount,
      ...TRemainingAccounts,
    ]
  >;

export type StoreInstructionData = {
  /**
   * List of pubkeys to store in the config account,
   * and whether each pubkey needs to sign subsequent calls to `store`.
   * Non-signer pubkeys do not need to be passed to the program as accounts.
   */
  keys: ConfigKeys;
  /** Arbitrary data to store in the config account. */
  data: ReadonlyUint8Array;
};

export type StoreInstructionDataArgs = {
  /**
   * List of pubkeys to store in the config account,
   * and whether each pubkey needs to sign subsequent calls to `store`.
   * Non-signer pubkeys do not need to be passed to the program as accounts.
   */
  keys: ConfigKeysArgs;
  /** Arbitrary data to store in the config account. */
  data: ReadonlyUint8Array;
};

export function getStoreInstructionDataEncoder(): Encoder<StoreInstructionDataArgs> {
  return getStructEncoder([
    ['keys', getConfigKeysEncoder()],
    ['data', getBytesEncoder()],
  ]);
}

export function getStoreInstructionDataDecoder(): Decoder<StoreInstructionData> {
  return getStructDecoder([
    ['keys', getConfigKeysDecoder()],
    ['data', getBytesDecoder()],
  ]);
}

export function getStoreInstructionDataCodec(): Codec<
  StoreInstructionDataArgs,
  StoreInstructionData
> {
  return combineCodec(
    getStoreInstructionDataEncoder(),
    getStoreInstructionDataDecoder()
  );
}

export type StoreInput<TAccountConfigAccount extends string = string> = {
  /**
   * The config account to be modified.
   * Must sign during the first call to `store` to initialize the account,
   * or if no signers are configured in the config data.
   */
  configAccount:
    | Address<TAccountConfigAccount>
    | TransactionSigner<TAccountConfigAccount>;
  keys: StoreInstructionDataArgs['keys'];
  data: StoreInstructionDataArgs['data'];
  signers?: Array<TransactionSigner>;
};

export function getStoreInstruction<TAccountConfigAccount extends string>(
  input: StoreInput<TAccountConfigAccount>
): StoreInstruction<
  typeof CONFIG_PROGRAM_ADDRESS,
  (typeof input)['configAccount'] extends TransactionSigner<TAccountConfigAccount>
    ? WritableSignerAccount<TAccountConfigAccount> &
        IAccountSignerMeta<TAccountConfigAccount>
    : TAccountConfigAccount
> {
  // Program address.
  const programAddress = CONFIG_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    configAccount: { value: input.configAccount ?? null, isWritable: true },
  };
  const accounts = originalAccounts as Record<
    keyof typeof originalAccounts,
    ResolvedAccount
  >;

  // Original args.
  const args = { ...input };

  // Remaining accounts.
  const remainingAccounts: IAccountMeta[] = (args.signers ?? []).map(
    (signer) => ({
      address: signer.address,
      role: AccountRole.READONLY_SIGNER,
      signer,
    })
  );

  const getAccountMeta = getAccountMetaFactory(programAddress, 'omitted');
  const instruction = {
    accounts: [getAccountMeta(accounts.configAccount), ...remainingAccounts],
    programAddress,
    data: getStoreInstructionDataEncoder().encode(
      args as StoreInstructionDataArgs
    ),
  } as StoreInstruction<
    typeof CONFIG_PROGRAM_ADDRESS,
    (typeof input)['configAccount'] extends TransactionSigner<TAccountConfigAccount>
      ? WritableSignerAccount<TAccountConfigAccount> &
          IAccountSignerMeta<TAccountConfigAccount>
      : TAccountConfigAccount
  >;

  return instruction;
}

export type ParsedStoreInstruction<
  TProgram extends string = typeof CONFIG_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /**
     * The config account to be modified.
     * Must sign during the first call to `store` to initialize the account,
     * or if no signers are configured in the config data.
     */

    configAccount: TAccountMetas[0];
  };
  data: StoreInstructionData;
};

export function parseStoreInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedStoreInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 1) {
    // TODO: Coded error.
    throw new Error('Not enough accounts');
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts![accountIndex]!;
    accountIndex += 1;
    return accountMeta;
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      configAccount: getNextAccount(),
    },
    data: getStoreInstructionDataDecoder().decode(instruction.data),
  };
}