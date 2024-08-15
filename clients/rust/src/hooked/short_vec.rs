#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::pubkey::Pubkey,
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

impl<T: BorshSerialize> BorshSerialize for ShortVec<T> {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        ShortU16(self.0.len() as u16).serialize(writer)?;
        for item in &self.0 {
            item.serialize(writer)?;
        }
        Ok(())
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

#[cfg(feature = "serde")]
impl<T: Serialize> Serialize for ShortVec<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        solana_program::short_vec::serialize(&self.0, serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Deserialize<'de>> Deserialize<'de> for ShortVec<T> {
    fn deserialize<D>(deserializer: D) -> Result<ShortVec<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        solana_program::short_vec::deserialize(deserializer).map(ShortVec)
    }
}

/// ConfigKeys type - uses short vec.
pub type ConfigKeys = ShortVec<(Pubkey, bool)>;

#[cfg(test)]
mod tests {
    #[cfg(feature = "serde")]
    use {super::*, assert_matches::assert_matches, bincode::serialize};
    use {
        bincode::deserialize,
        solana_program::short_vec::{decode_shortu16_len, ShortU16},
    };

    /// Return the serialized length.
    fn encode_len(len: u16) -> Vec<u8> {
        bincode::serialize(&ShortU16(len)).unwrap()
    }

    fn assert_len_encoding(len: u16, bytes: &[u8]) {
        assert_eq!(encode_len(len), bytes, "unexpected usize encoding");
        assert_eq!(
            decode_shortu16_len(bytes).unwrap(),
            (usize::from(len), bytes.len()),
            "unexpected usize decoding"
        );
    }

    #[test]
    fn test_short_vec_encode_len() {
        assert_len_encoding(0x0, &[0x0]);
        assert_len_encoding(0x7f, &[0x7f]);
        assert_len_encoding(0x80, &[0x80, 0x01]);
        assert_len_encoding(0xff, &[0xff, 0x01]);
        assert_len_encoding(0x100, &[0x80, 0x02]);
        assert_len_encoding(0x7fff, &[0xff, 0xff, 0x01]);
        assert_len_encoding(0xffff, &[0xff, 0xff, 0x03]);
    }

    fn assert_good_deserialized_value(value: u16, bytes: &[u8]) {
        assert_eq!(value, deserialize::<ShortU16>(bytes).unwrap().0);
    }

    fn assert_bad_deserialized_value(bytes: &[u8]) {
        assert!(deserialize::<ShortU16>(bytes).is_err());
    }

    #[test]
    fn test_deserialize() {
        assert_good_deserialized_value(0x0000, &[0x00]);
        assert_good_deserialized_value(0x007f, &[0x7f]);
        assert_good_deserialized_value(0x0080, &[0x80, 0x01]);
        assert_good_deserialized_value(0x00ff, &[0xff, 0x01]);
        assert_good_deserialized_value(0x0100, &[0x80, 0x02]);
        assert_good_deserialized_value(0x07ff, &[0xff, 0x0f]);
        assert_good_deserialized_value(0x3fff, &[0xff, 0x7f]);
        assert_good_deserialized_value(0x4000, &[0x80, 0x80, 0x01]);
        assert_good_deserialized_value(0xffff, &[0xff, 0xff, 0x03]);

        // aliases
        // 0x0000
        assert_bad_deserialized_value(&[0x80, 0x00]);
        assert_bad_deserialized_value(&[0x80, 0x80, 0x00]);
        // 0x007f
        assert_bad_deserialized_value(&[0xff, 0x00]);
        assert_bad_deserialized_value(&[0xff, 0x80, 0x00]);
        // 0x0080
        assert_bad_deserialized_value(&[0x80, 0x81, 0x00]);
        // 0x00ff
        assert_bad_deserialized_value(&[0xff, 0x81, 0x00]);
        // 0x0100
        assert_bad_deserialized_value(&[0x80, 0x82, 0x00]);
        // 0x07ff
        assert_bad_deserialized_value(&[0xff, 0x8f, 0x00]);
        // 0x3fff
        assert_bad_deserialized_value(&[0xff, 0xff, 0x00]);

        // too short
        assert_bad_deserialized_value(&[]);
        assert_bad_deserialized_value(&[0x80]);

        // too long
        assert_bad_deserialized_value(&[0x80, 0x80, 0x80, 0x00]);

        // too large
        // 0x0001_0000
        assert_bad_deserialized_value(&[0x80, 0x80, 0x04]);
        // 0x0001_8000
        assert_bad_deserialized_value(&[0x80, 0x80, 0x06]);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_short_vec_u8() {
        let vec = ShortVec(vec![4u8; 32]);
        let bytes = serialize(&vec).unwrap();
        assert_eq!(bytes.len(), vec.0.len() + 1);

        let vec1: ShortVec<u8> = deserialize(&bytes).unwrap();
        assert_eq!(vec.0, vec1.0);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_short_vec_u8_too_long() {
        let vec = ShortVec(vec![4u8; u16::MAX as usize]);
        assert_matches!(serialize(&vec), Ok(_));

        let vec = ShortVec(vec![4u8; u16::MAX as usize + 1]);
        assert_matches!(serialize(&vec), Err(_));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_short_vec_aliased_length() {
        let bytes = [
            0x81, 0x80, 0x00, // 3-byte alias of 1
            0x00,
        ];
        assert!(deserialize::<ShortVec<u8>>(&bytes).is_err());
    }
}
