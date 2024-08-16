import {
  getShortU16Decoder,
  getShortU16Encoder,
  combineCodec,
  Decoder,
  Encoder,
  getAddressEncoder,
  getArrayDecoder,
  getArrayEncoder,
  getBooleanEncoder,
  getTupleEncoder,
  VariableSizeDecoder,
  VariableSizeEncoder,
  Address,
  getAddressDecoder,
  getBooleanDecoder,
  getTupleDecoder,
} from '@solana/web3.js';

/**
 * ShortVec generic type.
 */
export type ShortVec<T> = T[];
export type ShortVecArgs<T> = ShortVec<T>;

export const getShortVecEncoder = <T>(
  elementEncoder: Encoder<T>
): VariableSizeEncoder<ShortVec<T>> =>
  getArrayEncoder(elementEncoder, { size: getShortU16Encoder() });

export const getShortVecDecoder = <T>(
  elementDecoder: Decoder<T>
): VariableSizeDecoder<ShortVec<T>> =>
  getArrayDecoder(elementDecoder, { size: getShortU16Decoder() });

export const getShortVecCodec = <T>(
  elementEncoder: Encoder<T>,
  elementDecoder: Decoder<T>
) =>
  combineCodec(
    getShortVecEncoder(elementEncoder),
    getShortVecDecoder(elementDecoder)
  );

/**
 * ConfigKeys type - uses short vec.
 */
export type ConfigKeys = ShortVec<[Address, boolean]>;
export type ConfigKeysArgs = ConfigKeys;

export const getConfigKeysEncoder = (): VariableSizeEncoder<
  ShortVec<[Address, boolean]>
> =>
  getShortVecEncoder(
    getTupleEncoder([getAddressEncoder(), getBooleanEncoder()])
  );

export const getConfigKeysDecoder = (): VariableSizeDecoder<
  ShortVec<[Address, boolean]>
> =>
  getShortVecDecoder(
    getTupleDecoder([getAddressDecoder(), getBooleanDecoder()])
  ) as VariableSizeDecoder<ShortVec<[Address, boolean]>>;

export const getConfigKeysCodec = combineCodec(
  getConfigKeysEncoder(),
  getConfigKeysDecoder()
);
