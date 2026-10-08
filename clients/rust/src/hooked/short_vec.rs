use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_address::Address,
};

struct ShortU16(u16);

impl BorshSerialize for ShortU16 {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let mut value = self.0;
        while value > 0x7F {
            writer.write_all(&[(value as u8) | 0x80])?;
            value >>= 7;
        }
        writer.write_all(&[value as u8])
    }
}

impl BorshDeserialize for ShortU16 {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        // Mirrors `solana_short_vec::decode_shortu16_len`, which is what the
        // program uses: at most 3 bytes, no alias (trailing zero byte)
        // encodings, and the value must fit in a `u16`.
        let mut value: u32 = 0;

        for shift in [0, 7, 14] {
            let mut byte = [0u8; 1];
            reader.read_exact(&mut byte)?;

            if byte[0] == 0 && shift != 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Alias encoding while decoding",
                ));
            }

            value |= u32::from(byte[0] & 0x7F) << shift;

            // If the top bit is not set, this is the last byte.
            if byte[0] & 0x80 == 0 {
                return u16::try_from(value).map(ShortU16).map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Overflow while decoding")
                });
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Overflow while decoding",
        ))
    }
}

/// ShortVec generic type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShortVec<T>(pub Vec<T>);

fn borsh_serialize_as_short_vec<T: BorshSerialize, W: std::io::Write>(
    vec: &Vec<T>,
    writer: &mut W,
) -> std::io::Result<()> {
    let len = u16::try_from(vec.len()).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "length larger than u16")
    })?;
    ShortU16(len).serialize(writer)?;
    for item in vec {
        item.serialize(writer)?;
    }
    Ok(())
}

impl<T: BorshSerialize> BorshSerialize for ShortVec<T> {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        borsh_serialize_as_short_vec(&self.0, writer)
    }
}

impl<T: BorshDeserialize> BorshDeserialize for ShortVec<T> {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let ShortU16(len) = ShortU16::deserialize_reader(reader)?;
        let mut vec = Vec::with_capacity(len as usize);
        for _ in 0..len {
            vec.push(T::deserialize_reader(reader)?);
        }
        Ok(ShortVec(vec))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigKeys {
    /// Each key tuple comprises a unique `Address` identifier,
    /// and `bool` whether that key is a signer of the data.
    pub keys: Vec<(Address, bool)>,
}

impl BorshSerialize for ConfigKeys {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        borsh_serialize_as_short_vec(&self.keys, writer)
    }
}

impl BorshDeserialize for ConfigKeys {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        Ok(ConfigKeys {
            keys: ShortVec::deserialize_reader(reader)?.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use {super::*, solana_address::Address};

    fn address(seed: u8) -> Address {
        Address::from([seed; 32])
    }

    #[test]
    fn test_serialization_borsh() {
        fn test_serialization(data: ConfigKeys) {
            let bytes = borsh::to_vec(&data).unwrap();
            let data1 = ConfigKeys::try_from_slice(&bytes).unwrap();
            assert_eq!(data, data1);
        }

        test_serialization(ConfigKeys { keys: vec![] });

        test_serialization(ConfigKeys {
            keys: vec![(address(1), false)],
        });

        test_serialization(ConfigKeys {
            keys: vec![(address(1), true), (address(2), false)],
        });

        test_serialization(ConfigKeys {
            keys: vec![
                (address(1), true),
                (address(2), false),
                (address(3), true),
                (address(4), true),
                (address(5), false),
                (address(6), true),
            ],
        });
    }

    #[test]
    fn test_short_u16_deserialize() {
        // Canonical encodings, as produced by `solana-short-vec`.
        for (bytes, value) in [
            (&[0x00][..], 0),
            (&[0x7f], 0x7f),
            (&[0x80, 0x01], 0x80),
            (&[0xff, 0x7f], 0x3fff),
            (&[0x80, 0x80, 0x01], 0x4000),
            (&[0xff, 0xff, 0x03], u16::MAX),
        ] {
            assert_eq!(ShortU16::try_from_slice(bytes).unwrap().0, value);
        }

        // Encodings rejected by `solana-short-vec` (and so by the program).
        for bytes in [
            // Alias: trailing zero byte.
            &[0x80, 0x00][..],
            &[0xff, 0x80, 0x00],
            // Overflow: value larger than `u16::MAX`.
            &[0x80, 0x80, 0x04],
            &[0xff, 0xff, 0x7f],
            // Third byte has the continuation bit set.
            &[0x80, 0x80, 0x81],
        ] {
            assert!(ShortU16::try_from_slice(bytes).is_err(), "{bytes:?}");
        }
    }

    #[test]
    fn test_short_vec_serialize_too_long() {
        let max = ShortVec(vec![0u8; u16::MAX as usize]);
        let bytes = borsh::to_vec(&max).unwrap();
        assert_eq!(&bytes[..3], &[0xff, 0xff, 0x03]);

        let too_long = ShortVec(vec![0u8; u16::MAX as usize + 1]);
        assert!(borsh::to_vec(&too_long).is_err());
    }
}
