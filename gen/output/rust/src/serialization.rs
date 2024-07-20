// same as cbor_event::de::Deserialize but with our DeserializeError
pub trait Deserialize {
    fn from_cbor_bytes(data: &[u8]) -> Result<Self, DeserializeError>
    where
        Self: Sized,
    {
        let mut raw = Deserializer::from(std::io::Cursor::new(data));
        Self::deserialize(&mut raw)
    }

    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError>
    where
        Self: Sized;
}

impl<T: cbor_event::de::Deserialize> Deserialize for T {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<T, DeserializeError> {
        T::deserialize(raw).map_err(DeserializeError::from)
    }
}
pub struct CBORReadLen {
    deser_len: cbor_event::Len,
    read: u64,
}

impl CBORReadLen {
    pub fn new(len: cbor_event::Len) -> Self {
        Self {
            deser_len: len,
            read: 0,
        }
    }

    pub fn read(&self) -> u64 {
        self.read
    }

    // Marks {n} values as being read, and if we go past the available definite length
    // given by the CBOR, we return an error.
    pub fn read_elems(&mut self, count: usize) -> Result<(), DeserializeFailure> {
        match self.deser_len {
            cbor_event::Len::Len(n) => {
                self.read += count as u64;
                if self.read > n {
                    Err(DeserializeFailure::DefiniteLenMismatch(n, None))
                } else {
                    Ok(())
                }
            }
            cbor_event::Len::Indefinite => Ok(()),
        }
    }

    pub fn finish(&self) -> Result<(), DeserializeFailure> {
        match self.deser_len {
            cbor_event::Len::Len(n) => {
                if self.read == n {
                    Ok(())
                } else {
                    Err(DeserializeFailure::DefiniteLenMismatch(n, Some(self.read)))
                }
            }
            cbor_event::Len::Indefinite => Ok(()),
        }
    }
}

pub trait DeserializeEmbeddedGroup {
    fn deserialize_as_embedded_group<R: BufRead + Seek>(
        raw: &mut Deserializer<R>,
        read_len: &mut CBORReadLen,
        len: cbor_event::Len,
    ) -> Result<Self, DeserializeError>
    where
        Self: Sized;
}
pub trait SerializeEmbeddedGroup {
    fn serialize_as_embedded_group<'a, W: Write + Sized>(
        &self,
        serializer: &'a mut Serializer<W>,
    ) -> cbor_event::Result<&'a mut Serializer<W>>;
}

pub trait ToCBORBytes {
    fn to_cbor_bytes(&self) -> Vec<u8>;
}

impl<T: cbor_event::se::Serialize> ToCBORBytes for T {
    fn to_cbor_bytes(&self) -> Vec<u8> {
        let mut buf = Serializer::new_vec();
        self.serialize(&mut buf).unwrap();
        buf.finalize()
    }
}

// This file was code-generated using an experimental CDDL to rust tool:
// https://github.com/dcSpark/cddl-codegen

use super::*;
use crate::error::*;
use cbor_event::de::Deserializer;
use cbor_event::se::{Serialize, Serializer};
use std::io::{BufRead, Seek, SeekFrom, Write};

impl cbor_event::se::Serialize for Anyy {
    fn serialize<'se, W: Write>(
        &self,
        serializer: &'se mut Serializer<W>,
    ) -> cbor_event::Result<&'se mut Serializer<W>> {
        serializer.write_array(cbor_event::Len::Len(1))?;
        serializer.write_unsigned_integer(0u64)?;
        Ok(serializer)
    }
}

impl Deserialize for Anyy {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        let len = raw.array()?;
        let mut read_len = CBORReadLen::new(len);
        read_len.read_elems(1)?;
        read_len.finish()?;
        let index_0_value = raw.unsigned_integer()?;
        if index_0_value != 0 {
            return Err(DeserializeFailure::FixedValueMismatch {
                found: Key::Uint(index_0_value),
                expected: Key::Uint(0),
            }
            .into());
        }
        match len {
            cbor_event::Len::Len(_) => (),
            cbor_event::Len::Indefinite => match raw.special()? {
                cbor_event::Special::Break => (),
                _ => return Err(DeserializeFailure::EndingBreakMissing.into()),
            },
        }
        Ok(Anyy {})
    }
}

impl cbor_event::se::Serialize for Int {
    fn serialize<'se, W: Write>(
        &self,
        serializer: &'se mut Serializer<W>,
    ) -> cbor_event::Result<&'se mut Serializer<W>> {
        match self {
            Self::Uint(x) => serializer.write_unsigned_integer(*x),
            Self::Nint(x) => serializer
                .write_negative_integer_sz(-((*x as i128) + 1), cbor_event::Sz::canonical(*x)),
        }
    }
}

impl Deserialize for Int {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        (|| -> Result<_, DeserializeError> {
            match raw.cbor_type()? {
                cbor_event::Type::UnsignedInteger => Ok(Self::Uint(raw.unsigned_integer()?)),
                cbor_event::Type::NegativeInteger => Ok(Self::Nint(
                    (-1 - raw.negative_integer_sz().map(|(x, _enc)| x)?) as u64,
                )),
                _ => Err(DeserializeFailure::NoVariantMatched.into()),
            }
        })()
        .map_err(|e| e.annotate("Int"))
    }
}

impl cbor_event::se::Serialize for IntOrText {
    fn serialize<'se, W: Write>(
        &self,
        serializer: &'se mut Serializer<W>,
    ) -> cbor_event::Result<&'se mut Serializer<W>> {
        match self {
            IntOrText::Int(int) => int.serialize(serializer),
            IntOrText::Text(text) => serializer.write_text(&text),
        }
    }
}

impl Deserialize for IntOrText {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        match raw.cbor_type()? {
            cbor_event::Type::Array | cbor_event::Type::Map => {
                Ok(IntOrText::Int(Int::deserialize(raw)?))
            }
            cbor_event::Type::Text => Ok(IntOrText::Text(raw.text()? as String)),
            _ => Err(DeserializeError::new(
                "IntOrText",
                DeserializeFailure::NoVariantMatched,
            )),
        }
    }
}

impl cbor_event::se::Serialize for Key {
    fn serialize<'se, W: Write>(
        &self,
        serializer: &'se mut Serializer<W>,
    ) -> cbor_event::Result<&'se mut Serializer<W>> {
        match self {
            Key::Int(int) => int.serialize(serializer),
            Key::Text(text) => serializer.write_text(&text),
        }
    }
}

impl Deserialize for Key {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        match raw.cbor_type()? {
            cbor_event::Type::Array | cbor_event::Type::Map => Ok(Key::Int(Int::deserialize(raw)?)),
            cbor_event::Type::Text => Ok(Key::Text(raw.text()? as String)),
            _ => Err(DeserializeError::new(
                "Key",
                DeserializeFailure::NoVariantMatched,
            )),
        }
    }
}

impl cbor_event::se::Serialize for Salted {
    fn serialize<'se, W: Write>(
        &self,
        serializer: &'se mut Serializer<W>,
    ) -> cbor_event::Result<&'se mut Serializer<W>> {
        match self {
            Salted::SaltedClaim(salted_claim) => {
                let mut salted_claim_inner_se = Serializer::new_vec();
                salted_claim.serialize(&mut salted_claim_inner_se)?;
                let salted_claim_bytes = salted_claim_inner_se.finalize();
                serializer.write_bytes(&salted_claim_bytes)
            }
            Salted::SaltedElement(salted_element) => {
                let mut salted_element_inner_se = Serializer::new_vec();
                salted_element.serialize(&mut salted_element_inner_se)?;
                let salted_element_bytes = salted_element_inner_se.finalize();
                serializer.write_bytes(&salted_element_bytes)
            }
        }
    }
}

impl Deserialize for Salted {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        let initial_position = raw.as_mut_ref().stream_position().unwrap();
        let mut errs = Vec::new();
        let deser_variant = (|raw: &mut Deserializer<_>| -> Result<_, DeserializeError> {
            let salted_claim_bytes = raw.bytes()?;
            let inner_de = &mut Deserializer::from(std::io::Cursor::new(salted_claim_bytes));
            SaltedClaimItem::deserialize(inner_de)
        })(raw);
        match deser_variant {
            Ok(salted_claim) => return Ok(Self::SaltedClaim(salted_claim)),
            Err(e) => {
                errs.push(e.annotate("SaltedClaim"));
                raw.as_mut_ref()
                    .seek(SeekFrom::Start(initial_position))
                    .unwrap();
            }
        };
        let deser_variant = (|raw: &mut Deserializer<_>| -> Result<_, DeserializeError> {
            let salted_element_bytes = raw.bytes()?;
            let inner_de = &mut Deserializer::from(std::io::Cursor::new(salted_element_bytes));
            SaltedElementItem::deserialize(inner_de)
        })(raw);
        match deser_variant {
            Ok(salted_element) => return Ok(Self::SaltedElement(salted_element)),
            Err(e) => {
                errs.push(e.annotate("SaltedElement"));
                raw.as_mut_ref()
                    .seek(SeekFrom::Start(initial_position))
                    .unwrap();
            }
        };
        Err(DeserializeError::new(
            "Salted",
            DeserializeFailure::NoVariantMatchedWithCauses(errs),
        ))
    }
}

impl cbor_event::se::Serialize for SaltedClaimItem {
    fn serialize<'se, W: Write>(
        &self,
        serializer: &'se mut Serializer<W>,
    ) -> cbor_event::Result<&'se mut Serializer<W>> {
        serializer.write_array(cbor_event::Len::Len(3))?;
        serializer.write_bytes(&self.salt)?;
        self.index_1.serialize(serializer)?;
        self.value.serialize(serializer)?;
        Ok(serializer)
    }
}

impl Deserialize for SaltedClaimItem {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        let len = raw.array()?;
        let mut read_len = CBORReadLen::new(len);
        read_len.read_elems(3)?;
        read_len.finish()?;
        let salt = raw
            .bytes()
            .map_err(Into::<DeserializeError>::into)
            .and_then(|bytes| {
                if bytes.len() < 16 || bytes.len() > 16 {
                    Err(DeserializeFailure::RangeCheck {
                        found: bytes.len() as isize,
                        min: Some(16),
                        max: Some(16),
                    }
                    .into())
                } else {
                    Ok(bytes)
                }
            })? as Vec<u8>;
        let index_1 = IntOrText::deserialize(raw)?;
        let value = Anyy::deserialize(raw)?;
        match len {
            cbor_event::Len::Len(_) => (),
            cbor_event::Len::Indefinite => match raw.special()? {
                cbor_event::Special::Break => (),
                _ => return Err(DeserializeFailure::EndingBreakMissing.into()),
            },
        }
        Ok(SaltedClaimItem {
            salt,
            index_1,
            value,
        })
    }
}

impl cbor_event::se::Serialize for SaltedElementItem {
    fn serialize<'se, W: Write>(
        &self,
        serializer: &'se mut Serializer<W>,
    ) -> cbor_event::Result<&'se mut Serializer<W>> {
        serializer.write_array(cbor_event::Len::Len(2))?;
        serializer.write_bytes(&self.salt)?;
        self.value.serialize(serializer)?;
        Ok(serializer)
    }
}

impl Deserialize for SaltedElementItem {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        let len = raw.array()?;
        let mut read_len = CBORReadLen::new(len);
        read_len.read_elems(2)?;
        read_len.finish()?;
        let salt = raw
            .bytes()
            .map_err(Into::<DeserializeError>::into)
            .and_then(|bytes| {
                if bytes.len() < 16 || bytes.len() > 16 {
                    Err(DeserializeFailure::RangeCheck {
                        found: bytes.len() as isize,
                        min: Some(16),
                        max: Some(16),
                    }
                    .into())
                } else {
                    Ok(bytes)
                }
            })? as Vec<u8>;
        let value = Anyy::deserialize(raw)?;
        match len {
            cbor_event::Len::Len(_) => (),
            cbor_event::Len::Indefinite => match raw.special()? {
                cbor_event::Special::Break => (),
                _ => return Err(DeserializeFailure::EndingBreakMissing.into()),
            },
        }
        Ok(SaltedElementItem { salt, value })
    }
}

impl cbor_event::se::Serialize for SdCwt {
    fn serialize<'se, W: Write>(
        &self,
        serializer: &'se mut Serializer<W>,
    ) -> cbor_event::Result<&'se mut Serializer<W>> {
        serializer.write_tag(18u64)?;
        serializer.write_array(cbor_event::Len::Len(4))?;
        let mut protected_inner_se = Serializer::new_vec();
        self.protected.serialize(&mut protected_inner_se)?;
        let protected_bytes = protected_inner_se.finalize();
        serializer.write_bytes(&protected_bytes)?;
        self.unprotected.serialize(serializer)?;
        let mut payload_inner_se = Serializer::new_vec();
        self.payload.serialize(&mut payload_inner_se)?;
        let payload_bytes = payload_inner_se.finalize();
        serializer.write_bytes(&payload_bytes)?;
        serializer.write_bytes(&self.signature)?;
        Ok(serializer)
    }
}

impl Deserialize for SdCwt {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        let tag = raw.tag()?;
        if tag != 18 {
            return Err(DeserializeError::new(
                "SdCwt",
                DeserializeFailure::TagMismatch {
                    found: tag,
                    expected: 18,
                },
            ));
        }
        let len = raw.array()?;
        let mut read_len = CBORReadLen::new(len);
        read_len.read_elems(4)?;
        read_len.finish()?;
        let protected_bytes = raw.bytes()?;
        let inner_de = &mut Deserializer::from(std::io::Cursor::new(protected_bytes));
        let protected = SdProtected::deserialize(inner_de)?;
        let unprotected = Unprotected::deserialize(raw)?;
        let payload_bytes = raw.bytes()?;
        let inner_de = &mut Deserializer::from(std::io::Cursor::new(payload_bytes));
        let payload = SdPayload::deserialize(inner_de)?;
        let signature = raw.bytes()? as Vec<u8>;
        match len {
            cbor_event::Len::Len(_) => (),
            cbor_event::Len::Indefinite => match raw.special()? {
                cbor_event::Special::Break => (),
                _ => return Err(DeserializeFailure::EndingBreakMissing.into()),
            },
        }
        Ok(SdCwt {
            protected,
            unprotected,
            payload,
            signature,
        })
    }
}

impl cbor_event::se::Serialize for SdPayload {
    fn serialize<'se, W: Write>(
        &self,
        serializer: &'se mut Serializer<W>,
    ) -> cbor_event::Result<&'se mut Serializer<W>> {
        serializer.write_map(cbor_event::Len::Len(
            3 + match &self.iss {
                Some(_) => 1,
                None => 0,
            } + match &self.sub {
                Some(_) => 1,
                None => 0,
            } + match &self.key_4 {
                Some(_) => 1,
                None => 0,
            } + match &self.key_5 {
                Some(_) => 1,
                None => 0,
            } + match &self.cnonce {
                Some(_) => 1,
                None => 0,
            } + match &self.cnf {
                Some(_) => 1,
                None => 0,
            } + match &self.sd_hash {
                Some(_) => 1,
                None => 0,
            } + match &self.sd_alg {
                Some(_) => 1,
                None => 0,
            } + match &self.redacted_keys {
                Some(_) => 1,
                None => 0,
            },
        ))?;
        if let Some(field) = &self.iss {
            serializer.write_unsigned_integer(1u64)?;
            serializer.write_text(&field)?;
        }
        if let Some(field) = &self.sub {
            serializer.write_unsigned_integer(2u64)?;
            serializer.write_text(&field)?;
        }
        serializer.write_unsigned_integer(3u64)?;
        serializer.write_text(&self.aud)?;
        if let Some(field) = &self.key_4 {
            serializer.write_unsigned_integer(4u64)?;
            field.serialize(serializer)?;
        }
        if let Some(field) = &self.key_5 {
            serializer.write_unsigned_integer(5u64)?;
            field.serialize(serializer)?;
        }
        serializer.write_unsigned_integer(6u64)?;
        self.key_6.serialize(serializer)?;
        if let Some(field) = &self.cnf {
            serializer.write_unsigned_integer(8u64)?;
            serializer.write_map(cbor_event::Len::Len(field.len() as u64))?;
            for (key, value) in field.iter() {
                key.serialize(serializer)?;
                value.serialize(serializer)?;
            }
        }
        if let Some(field) = &self.cnonce {
            serializer.write_unsigned_integer(39u64)?;
            serializer.write_bytes(&field)?;
        }
        if let Some(field) = &self.sd_hash {
            serializer.write_unsigned_integer(1113u64)?;
            serializer.write_bytes(&field)?;
        }
        if let Some(field) = &self.sd_alg {
            serializer.write_unsigned_integer(1114u64)?;
            field.serialize(serializer)?;
        }
        if let Some(field) = &self.redacted_keys {
            serializer.write_unsigned_integer(1115u64)?;
            serializer.write_array(cbor_event::Len::Len(field.len() as u64))?;
            for element in field.iter() {
                serializer.write_bytes(&element)?;
            }
        }
        serializer.write_text("custom")?;
        serializer.write_map(cbor_event::Len::Len(self.custom.len() as u64))?;
        for (key, value) in self.custom.iter() {
            key.serialize(serializer)?;
            value.serialize(serializer)?;
        }
        Ok(serializer)
    }
}

impl Deserialize for SdPayload {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        let len = raw.map()?;
        let mut read_len = CBORReadLen::new(len);
        read_len.read_elems(3)?;
        let mut iss = None;
        let mut sub = None;
        let mut aud = None;
        let mut key_4 = None;
        let mut key_5 = None;
        let mut key_6 = None;
        let mut cnf = None;
        let mut cnonce = None;
        let mut sd_hash = None;
        let mut sd_alg = None;
        let mut redacted_keys = None;
        let mut custom = None;
        let mut read = 0;
        while match len {
            cbor_event::Len::Len(n) => read < n,
            cbor_event::Len::Indefinite => true,
        } {
            match raw.cbor_type()? {
                cbor_event::Type::UnsignedInteger => match raw.unsigned_integer()? {
                    1 => {
                        if iss.is_some() {
                            return Err(DeserializeFailure::DuplicateKey(Key::Uint(1)).into());
                        }
                        read_len.read_elems(1)?;
                        iss = Some(raw.text()? as String);
                    }
                    2 => {
                        if sub.is_some() {
                            return Err(DeserializeFailure::DuplicateKey(Key::Uint(2)).into());
                        }
                        read_len.read_elems(1)?;
                        sub = Some(raw.text()? as String);
                    }
                    3 => {
                        if aud.is_some() {
                            return Err(DeserializeFailure::DuplicateKey(Key::Uint(3)).into());
                        }
                        aud = Some(raw.text()? as String);
                    }
                    4 => {
                        if key_4.is_some() {
                            return Err(DeserializeFailure::DuplicateKey(Key::Uint(4)).into());
                        }
                        read_len.read_elems(1)?;
                        key_4 = Some(Int::deserialize(raw)?);
                    }
                    5 => {
                        if key_5.is_some() {
                            return Err(DeserializeFailure::DuplicateKey(Key::Uint(5)).into());
                        }
                        read_len.read_elems(1)?;
                        key_5 = Some(Int::deserialize(raw)?);
                    }
                    6 => {
                        if key_6.is_some() {
                            return Err(DeserializeFailure::DuplicateKey(Key::Uint(6)).into());
                        }
                        key_6 = Some(Int::deserialize(raw)?);
                    }
                    8 => {
                        if cnf.is_some() {
                            return Err(DeserializeFailure::DuplicateKey(Key::Uint(8)).into());
                        }
                        read_len.read_elems(1)?;
                        let mut cnf_table = BTreeMap::new();
                        let cnf_len = raw.map()?;
                        while match cnf_len {
                            cbor_event::Len::Len(n) => (cnf_table.len() as u64) < n,
                            cbor_event::Len::Indefinite => true,
                        } {
                            if raw.cbor_type()? == cbor_event::Type::Special {
                                assert_eq!(raw.special()?, cbor_event::Special::Break);
                                break;
                            }
                            let cnf_key = Key::deserialize(raw)?;
                            let cnf_value = Anyy::deserialize(raw)?;
                            if cnf_table.insert(cnf_key.clone(), cnf_value).is_some() {
                                return Err(DeserializeFailure::DuplicateKey(Key::Str(
                                    String::from("some complicated/unsupported type"),
                                ))
                                .into());
                            }
                        }
                        cnf = Some(cnf_table);
                    }
                    39 => {
                        if cnonce.is_some() {
                            return Err(DeserializeFailure::DuplicateKey(Key::Uint(39)).into());
                        }
                        read_len.read_elems(1)?;
                        cnonce = Some(raw.bytes()? as Vec<u8>);
                    }
                    1113 => {
                        if sd_hash.is_some() {
                            return Err(DeserializeFailure::DuplicateKey(Key::Uint(1113)).into());
                        }
                        read_len.read_elems(1)?;
                        sd_hash = Some(raw.bytes()? as Vec<u8>);
                    }
                    1114 => {
                        if sd_alg.is_some() {
                            return Err(DeserializeFailure::DuplicateKey(Key::Uint(1114)).into());
                        }
                        read_len.read_elems(1)?;
                        sd_alg = Some(Int::deserialize(raw)?);
                    }
                    1115 => {
                        if redacted_keys.is_some() {
                            return Err(DeserializeFailure::DuplicateKey(Key::Uint(1115)).into());
                        }
                        read_len.read_elems(1)?;
                        let mut redacted_keys_arr = Vec::new();
                        let len = raw.array()?;
                        while match len {
                            cbor_event::Len::Len(n) => (redacted_keys_arr.len() as u64) < n,
                            cbor_event::Len::Indefinite => true,
                        } {
                            if raw.cbor_type()? == cbor_event::Type::Special {
                                assert_eq!(raw.special()?, cbor_event::Special::Break);
                                break;
                            }
                            redacted_keys_arr.push(raw.bytes()? as Vec<u8>);
                        }
                        redacted_keys = Some(redacted_keys_arr);
                    }
                    unknown_key => {
                        return Err(DeserializeFailure::UnknownKey(Key::Uint(unknown_key)).into())
                    }
                },
                cbor_event::Type::Text => match raw.text()?.as_str() {
                    "custom" => {
                        if custom.is_some() {
                            return Err(DeserializeFailure::DuplicateKey(Key::Str(
                                "custom".into(),
                            ))
                            .into());
                        }
                        let mut custom_table = BTreeMap::new();
                        let custom_len = raw.map()?;
                        while match custom_len {
                            cbor_event::Len::Len(n) => (custom_table.len() as u64) < n,
                            cbor_event::Len::Indefinite => true,
                        } {
                            if raw.cbor_type()? == cbor_event::Type::Special {
                                assert_eq!(raw.special()?, cbor_event::Special::Break);
                                break;
                            }
                            let custom_key = Key::deserialize(raw)?;
                            let custom_value = Anyy::deserialize(raw)?;
                            if custom_table
                                .insert(custom_key.clone(), custom_value)
                                .is_some()
                            {
                                return Err(DeserializeFailure::DuplicateKey(Key::Str(
                                    String::from("some complicated/unsupported type"),
                                ))
                                .into());
                            }
                        }
                        custom = Some(custom_table);
                    }
                    unknown_key => {
                        return Err(DeserializeFailure::UnknownKey(Key::Str(
                            unknown_key.to_owned(),
                        ))
                        .into())
                    }
                },
                cbor_event::Type::Special => match len {
                    cbor_event::Len::Len(_) => {
                        return Err(DeserializeFailure::BreakInDefiniteLen.into())
                    }
                    cbor_event::Len::Indefinite => match raw.special()? {
                        cbor_event::Special::Break => break,
                        _ => return Err(DeserializeFailure::EndingBreakMissing.into()),
                    },
                },
                other_type => return Err(DeserializeFailure::UnexpectedKeyType(other_type).into()),
            }
            read += 1;
        }
        let aud = match aud {
            Some(x) => x,
            None => return Err(DeserializeFailure::MandatoryFieldMissing(Key::Uint(3)).into()),
        };
        let key_6 = match key_6 {
            Some(x) => x,
            None => return Err(DeserializeFailure::MandatoryFieldMissing(Key::Uint(6)).into()),
        };
        let custom = match custom {
            Some(x) => x,
            None => {
                return Err(
                    DeserializeFailure::MandatoryFieldMissing(Key::Str(String::from("custom")))
                        .into(),
                )
            }
        };
        read_len.finish()?;
        Ok(Self {
            iss,
            sub,
            aud,
            key_4,
            key_5,
            key_6,
            cnonce,
            cnf,
            sd_hash,
            sd_alg,
            redacted_keys,
            custom,
        })
    }
}

impl cbor_event::se::Serialize for SdProtected {
    fn serialize<'se, W: Write>(
        &self,
        serializer: &'se mut Serializer<W>,
    ) -> cbor_event::Result<&'se mut Serializer<W>> {
        serializer.write_map(cbor_event::Len::Len(3))?;
        serializer.write_unsigned_integer(1u64)?;
        self.alg.serialize(serializer)?;
        serializer.write_unsigned_integer(16u64)?;
        serializer.write_text(&self.typ)?;
        serializer.write_text("custom")?;
        serializer.write_map(cbor_event::Len::Len(self.custom.len() as u64))?;
        for (key, value) in self.custom.iter() {
            key.serialize(serializer)?;
            value.serialize(serializer)?;
        }
        Ok(serializer)
    }
}

impl Deserialize for SdProtected {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        let len = raw.map()?;
        let mut read_len = CBORReadLen::new(len);
        read_len.read_elems(3)?;
        read_len.finish()?;
        let mut alg = None;
        let mut typ = None;
        let mut custom = None;
        let mut read = 0;
        while match len {
            cbor_event::Len::Len(n) => read < n,
            cbor_event::Len::Indefinite => true,
        } {
            match raw.cbor_type()? {
                cbor_event::Type::UnsignedInteger => match raw.unsigned_integer()? {
                    1 => {
                        if alg.is_some() {
                            return Err(DeserializeFailure::DuplicateKey(Key::Uint(1)).into());
                        }
                        alg = Some(Int::deserialize(raw)?);
                    }
                    16 => {
                        if typ.is_some() {
                            return Err(DeserializeFailure::DuplicateKey(Key::Uint(16)).into());
                        }
                        typ = Some(raw.text()? as String);
                    }
                    unknown_key => {
                        return Err(DeserializeFailure::UnknownKey(Key::Uint(unknown_key)).into())
                    }
                },
                cbor_event::Type::Text => match raw.text()?.as_str() {
                    "custom" => {
                        if custom.is_some() {
                            return Err(DeserializeFailure::DuplicateKey(Key::Str(
                                "custom".into(),
                            ))
                            .into());
                        }
                        let mut custom_table = BTreeMap::new();
                        let custom_len = raw.map()?;
                        while match custom_len {
                            cbor_event::Len::Len(n) => (custom_table.len() as u64) < n,
                            cbor_event::Len::Indefinite => true,
                        } {
                            if raw.cbor_type()? == cbor_event::Type::Special {
                                assert_eq!(raw.special()?, cbor_event::Special::Break);
                                break;
                            }
                            let custom_key = Key::deserialize(raw)?;
                            let custom_value = Anyy::deserialize(raw)?;
                            if custom_table
                                .insert(custom_key.clone(), custom_value)
                                .is_some()
                            {
                                return Err(DeserializeFailure::DuplicateKey(Key::Str(
                                    String::from("some complicated/unsupported type"),
                                ))
                                .into());
                            }
                        }
                        custom = Some(custom_table);
                    }
                    unknown_key => {
                        return Err(DeserializeFailure::UnknownKey(Key::Str(
                            unknown_key.to_owned(),
                        ))
                        .into())
                    }
                },
                cbor_event::Type::Special => match len {
                    cbor_event::Len::Len(_) => {
                        return Err(DeserializeFailure::BreakInDefiniteLen.into())
                    }
                    cbor_event::Len::Indefinite => match raw.special()? {
                        cbor_event::Special::Break => break,
                        _ => return Err(DeserializeFailure::EndingBreakMissing.into()),
                    },
                },
                other_type => return Err(DeserializeFailure::UnexpectedKeyType(other_type).into()),
            }
            read += 1;
        }
        let alg = match alg {
            Some(x) => x,
            None => return Err(DeserializeFailure::MandatoryFieldMissing(Key::Uint(1)).into()),
        };
        let typ = match typ {
            Some(x) => x,
            None => return Err(DeserializeFailure::MandatoryFieldMissing(Key::Uint(16)).into()),
        };
        let custom = match custom {
            Some(x) => x,
            None => {
                return Err(
                    DeserializeFailure::MandatoryFieldMissing(Key::Str(String::from("custom")))
                        .into(),
                )
            }
        };
        ();
        Ok(Self { alg, typ, custom })
    }
}

impl cbor_event::se::Serialize for Unprotected {
    fn serialize<'se, W: Write>(
        &self,
        serializer: &'se mut Serializer<W>,
    ) -> cbor_event::Result<&'se mut Serializer<W>> {
        serializer.write_map(cbor_event::Len::Len(
            1 + match &self.sd_claims {
                Some(_) => 1,
                None => 0,
            } + match &self.sd_kbt {
                Some(_) => 1,
                None => 0,
            },
        ))?;
        if let Some(field) = &self.sd_claims {
            serializer.write_unsigned_integer(1111u64)?;
            serializer.write_array(cbor_event::Len::Len(field.len() as u64))?;
            for element in field.iter() {
                element.serialize(serializer)?;
            }
        }
        if let Some(field) = &self.sd_kbt {
            serializer.write_unsigned_integer(1112u64)?;
            serializer.write_bytes(&field)?;
        }
        serializer.write_text("custom")?;
        serializer.write_map(cbor_event::Len::Len(self.custom.len() as u64))?;
        for (key, value) in self.custom.iter() {
            key.serialize(serializer)?;
            value.serialize(serializer)?;
        }
        Ok(serializer)
    }
}

impl Deserialize for Unprotected {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        let len = raw.map()?;
        let mut read_len = CBORReadLen::new(len);
        read_len.read_elems(1)?;
        let mut sd_claims = None;
        let mut sd_kbt = None;
        let mut custom = None;
        let mut read = 0;
        while match len {
            cbor_event::Len::Len(n) => read < n,
            cbor_event::Len::Indefinite => true,
        } {
            match raw.cbor_type()? {
                cbor_event::Type::UnsignedInteger => match raw.unsigned_integer()? {
                    1111 => {
                        if sd_claims.is_some() {
                            return Err(DeserializeFailure::DuplicateKey(Key::Uint(1111)).into());
                        }
                        read_len.read_elems(1)?;
                        let mut sd_claims_arr = Vec::new();
                        let len = raw.array()?;
                        while match len {
                            cbor_event::Len::Len(n) => (sd_claims_arr.len() as u64) < n,
                            cbor_event::Len::Indefinite => true,
                        } {
                            if raw.cbor_type()? == cbor_event::Type::Special {
                                assert_eq!(raw.special()?, cbor_event::Special::Break);
                                break;
                            }
                            sd_claims_arr.push(Salted::deserialize(raw)?);
                        }
                        sd_claims = Some(sd_claims_arr);
                    }
                    1112 => {
                        if sd_kbt.is_some() {
                            return Err(DeserializeFailure::DuplicateKey(Key::Uint(1112)).into());
                        }
                        read_len.read_elems(1)?;
                        sd_kbt = Some(raw.bytes()? as Vec<u8>);
                    }
                    unknown_key => {
                        return Err(DeserializeFailure::UnknownKey(Key::Uint(unknown_key)).into())
                    }
                },
                cbor_event::Type::Text => match raw.text()?.as_str() {
                    "custom" => {
                        if custom.is_some() {
                            return Err(DeserializeFailure::DuplicateKey(Key::Str(
                                "custom".into(),
                            ))
                            .into());
                        }
                        let mut custom_table = BTreeMap::new();
                        let custom_len = raw.map()?;
                        while match custom_len {
                            cbor_event::Len::Len(n) => (custom_table.len() as u64) < n,
                            cbor_event::Len::Indefinite => true,
                        } {
                            if raw.cbor_type()? == cbor_event::Type::Special {
                                assert_eq!(raw.special()?, cbor_event::Special::Break);
                                break;
                            }
                            let custom_key = Key::deserialize(raw)?;
                            let custom_value = Anyy::deserialize(raw)?;
                            if custom_table
                                .insert(custom_key.clone(), custom_value)
                                .is_some()
                            {
                                return Err(DeserializeFailure::DuplicateKey(Key::Str(
                                    String::from("some complicated/unsupported type"),
                                ))
                                .into());
                            }
                        }
                        custom = Some(custom_table);
                    }
                    unknown_key => {
                        return Err(DeserializeFailure::UnknownKey(Key::Str(
                            unknown_key.to_owned(),
                        ))
                        .into())
                    }
                },
                cbor_event::Type::Special => match len {
                    cbor_event::Len::Len(_) => {
                        return Err(DeserializeFailure::BreakInDefiniteLen.into())
                    }
                    cbor_event::Len::Indefinite => match raw.special()? {
                        cbor_event::Special::Break => break,
                        _ => return Err(DeserializeFailure::EndingBreakMissing.into()),
                    },
                },
                other_type => return Err(DeserializeFailure::UnexpectedKeyType(other_type).into()),
            }
            read += 1;
        }
        let custom = match custom {
            Some(x) => x,
            None => {
                return Err(
                    DeserializeFailure::MandatoryFieldMissing(Key::Str(String::from("custom")))
                        .into(),
                )
            }
        };
        read_len.finish()?;
        Ok(Self {
            sd_claims,
            sd_kbt,
            custom,
        })
    }
}
