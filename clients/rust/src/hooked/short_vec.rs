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
        let mut value: u16 = 0;
        let mut shift = 0;

        loop {
            let mut byte = [0u8; 1];
            reader.read_exact(&mut byte)?;
            let part = (byte[0] & 0x7F) as u16;

            if shift >= 16 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Overflow while decoding",
                ));
            }

            value |= part << shift;
            shift += 7;

            // If the top bit is not set, this is the last byte.
            if byte[0] & 0x80 == 0 {
                break;
            }
        }

        Ok(ShortU16(value))
    }
}

/// ShortVec generic type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShortVec<T>(pub Vec<T>);

fn borsh_serialize_as_short_vec<T: BorshSerialize, W: std::io::Write>(
    vec: &Vec<T>,
    writer: &mut W,
) -> std::io::Result<()> {
    ShortU16(vec.len() as u16).serialize(writer)?;
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
}
