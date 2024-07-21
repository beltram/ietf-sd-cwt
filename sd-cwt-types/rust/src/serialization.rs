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
    deser_len: cbor_event::LenSz,
    read: u64,
}

impl CBORReadLen {
    pub fn new(len: cbor_event::LenSz) -> Self {
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
            cbor_event::LenSz::Len(n, _) => {
                self.read += count as u64;
                if self.read > n {
                    Err(DeserializeFailure::DefiniteLenMismatch(n, None))
                } else {
                    Ok(())
                }
            }
            cbor_event::LenSz::Indefinite => Ok(()),
        }
    }

    pub fn finish(&self) -> Result<(), DeserializeFailure> {
        match self.deser_len {
            cbor_event::LenSz::Len(n, _) => {
                if self.read == n {
                    Ok(())
                } else {
                    Err(DeserializeFailure::DefiniteLenMismatch(n, Some(self.read)))
                }
            }
            cbor_event::LenSz::Indefinite => Ok(()),
        }
    }
}

pub trait DeserializeEmbeddedGroup {
    fn deserialize_as_embedded_group<R: BufRead + Seek>(
        raw: &mut Deserializer<R>,
        read_len: &mut CBORReadLen,
        len: cbor_event::LenSz,
    ) -> Result<Self, DeserializeError>
    where
        Self: Sized;
}

#[inline]
pub fn sz_max(sz: cbor_event::Sz) -> u64 {
    match sz {
        cbor_event::Sz::Inline => 23u64,
        cbor_event::Sz::One => u8::MAX as u64,
        cbor_event::Sz::Two => u16::MAX as u64,
        cbor_event::Sz::Four => u32::MAX as u64,
        cbor_event::Sz::Eight => u64::MAX,
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum LenEncoding {
    Canonical,
    Definite(cbor_event::Sz),
    Indefinite,
}

impl Default for LenEncoding {
    fn default() -> Self {
        Self::Canonical
    }
}

impl From<cbor_event::LenSz> for LenEncoding {
    fn from(len_sz: cbor_event::LenSz) -> Self {
        match len_sz {
            cbor_event::LenSz::Len(len, sz) => {
                if cbor_event::Sz::canonical(len) == sz {
                    Self::Canonical
                } else {
                    Self::Definite(sz)
                }
            }
            cbor_event::LenSz::Indefinite => Self::Indefinite,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StringEncoding {
    Canonical,
    Indefinite(Vec<(u64, cbor_event::Sz)>),
    Definite(cbor_event::Sz),
}

impl Default for StringEncoding {
    fn default() -> Self {
        Self::Canonical
    }
}

impl From<cbor_event::StringLenSz> for StringEncoding {
    fn from(len_sz: cbor_event::StringLenSz) -> Self {
        match len_sz {
            cbor_event::StringLenSz::Len(sz) => Self::Definite(sz),
            cbor_event::StringLenSz::Indefinite(lens) => Self::Indefinite(lens),
        }
    }
}
#[inline]
pub fn fit_sz(len: u64, sz: Option<cbor_event::Sz>) -> cbor_event::Sz {
    match sz {
        Some(sz) => {
            if len <= sz_max(sz) {
                sz
            } else {
                cbor_event::Sz::canonical(len)
            }
        }
        None => cbor_event::Sz::canonical(len),
    }
}

impl LenEncoding {
    pub fn to_len_sz(&self, len: u64) -> cbor_event::LenSz {
        match self {
            Self::Canonical => cbor_event::LenSz::Len(len, cbor_event::Sz::canonical(len)),
            Self::Definite(sz) => {
                if sz_max(*sz) >= len {
                    cbor_event::LenSz::Len(len, *sz)
                } else {
                    cbor_event::LenSz::Len(len, cbor_event::Sz::canonical(len))
                }
            }
            Self::Indefinite => cbor_event::LenSz::Indefinite,
        }
    }

    pub fn end<'a, W: Write + Sized>(
        &self,
        serializer: &'a mut Serializer<W>,
    ) -> cbor_event::Result<&'a mut Serializer<W>> {
        if *self == Self::Indefinite {
            serializer.write_special(cbor_event::Special::Break)?;
        }
        Ok(serializer)
    }
}

impl StringEncoding {
    pub fn to_str_len_sz(&self, len: u64) -> cbor_event::StringLenSz {
        match self {
            Self::Canonical => cbor_event::StringLenSz::Len(cbor_event::Sz::canonical(len)),
            Self::Definite(sz) => {
                if sz_max(*sz) >= len {
                    cbor_event::StringLenSz::Len(*sz)
                } else {
                    cbor_event::StringLenSz::Len(cbor_event::Sz::canonical(len))
                }
            }
            Self::Indefinite(lens) => cbor_event::StringLenSz::Indefinite(lens.clone()),
        }
    }
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

use super::cbor_encodings::*;
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
        serializer.write_array_sz(
            self.encodings
                .as_ref()
                .map(|encs| encs.len_encoding)
                .unwrap_or_default()
                .to_len_sz(1),
        )?;
        serializer.write_unsigned_integer_sz(
            0u64,
            fit_sz(
                0u64,
                self.encodings
                    .as_ref()
                    .map(|encs| encs.index_0_encoding)
                    .unwrap_or_default(),
            ),
        )?;
        self.encodings
            .as_ref()
            .map(|encs| encs.len_encoding)
            .unwrap_or_default()
            .end(serializer)
    }
}

impl Deserialize for Anyy {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        let len = raw.array_sz()?;
        let len_encoding: LenEncoding = len.into();
        let mut read_len = CBORReadLen::new(len);
        read_len.read_elems(1)?;
        read_len.finish()?;
        (|| -> Result<_, DeserializeError> {
            let index_0_encoding = (|| -> Result<_, DeserializeError> {
                let (index_0_value, index_0_encoding) = raw.unsigned_integer_sz()?;
                if index_0_value != 0 {
                    return Err(DeserializeFailure::FixedValueMismatch {
                        found: Key::Uint(index_0_value),
                        expected: Key::Uint(0),
                    }
                    .into());
                }
                Ok(Some(index_0_encoding))
            })()
            .map_err(|e| e.annotate("index_0"))?;
            match len {
                cbor_event::LenSz::Len(_, _) => (),
                cbor_event::LenSz::Indefinite => match raw.special()? {
                    cbor_event::Special::Break => (),
                    _ => return Err(DeserializeFailure::EndingBreakMissing.into()),
                },
            }
            Ok(Anyy {
                encodings: Some(AnyyEncoding {
                    len_encoding,
                    index_0_encoding,
                }),
            })
        })()
        .map_err(|e| e.annotate("Anyy"))
    }
}

impl cbor_event::se::Serialize for Int {
    fn serialize<'se, W: Write>(
        &self,
        serializer: &'se mut Serializer<W>,
    ) -> cbor_event::Result<&'se mut Serializer<W>> {
        match self {
            Self::Uint { value, encoding } => {
                serializer.write_unsigned_integer_sz(*value, fit_sz(*value, *encoding))
            }
            Self::Nint { value, encoding } => serializer
                .write_negative_integer_sz(-((*value as i128) + 1), fit_sz(*value, *encoding)),
        }
    }
}

impl Deserialize for Int {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        (|| -> Result<_, DeserializeError> {
            match raw.cbor_type()? {
                cbor_event::Type::UnsignedInteger => raw
                    .unsigned_integer_sz()
                    .map(|(x, enc)| Self::Uint {
                        value: x,
                        encoding: Some(enc),
                    })
                    .map_err(std::convert::Into::into),
                cbor_event::Type::NegativeInteger => raw
                    .negative_integer_sz()
                    .map(|(x, enc)| Self::Nint {
                        value: (-1 - x) as u64,
                        encoding: Some(enc),
                    })
                    .map_err(std::convert::Into::into),
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
            IntOrText::Text {
                text,
                text_encoding,
            } => serializer.write_text_sz(&text, text_encoding.to_str_len_sz(text.len() as u64)),
        }
    }
}

impl Deserialize for IntOrText {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        (|| -> Result<_, DeserializeError> {
            match raw.cbor_type()? {
                cbor_event::Type::Array | cbor_event::Type::Map => {
                    Ok(IntOrText::Int(Int::deserialize(raw)?))
                }
                cbor_event::Type::Text => {
                    let (text, text_encoding) = raw
                        .text_sz()
                        .map(|(s, enc)| (s, StringEncoding::from(enc)))?;
                    Ok(Self::Text {
                        text,
                        text_encoding,
                    })
                }
                _ => Err(DeserializeError::new(
                    "IntOrText",
                    DeserializeFailure::NoVariantMatched,
                )),
            }
        })()
        .map_err(|e| e.annotate("IntOrText"))
    }
}

impl cbor_event::se::Serialize for Keyy {
    fn serialize<'se, W: Write>(
        &self,
        serializer: &'se mut Serializer<W>,
    ) -> cbor_event::Result<&'se mut Serializer<W>> {
        match self {
            Keyy::Int(int) => int.serialize(serializer),
            Keyy::Text {
                text,
                text_encoding,
            } => serializer.write_text_sz(&text, text_encoding.to_str_len_sz(text.len() as u64)),
        }
    }
}

impl Deserialize for Keyy {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        (|| -> Result<_, DeserializeError> {
            match raw.cbor_type()? {
                cbor_event::Type::Array | cbor_event::Type::Map => {
                    Ok(Keyy::Int(Int::deserialize(raw)?))
                }
                cbor_event::Type::Text => {
                    let (text, text_encoding) = raw
                        .text_sz()
                        .map(|(s, enc)| (s, StringEncoding::from(enc)))?;
                    Ok(Self::Text {
                        text,
                        text_encoding,
                    })
                }
                _ => Err(DeserializeError::new(
                    "Keyy",
                    DeserializeFailure::NoVariantMatched,
                )),
            }
        })()
        .map_err(|e| e.annotate("Keyy"))
    }
}

impl cbor_event::se::Serialize for Salted {
    fn serialize<'se, W: Write>(
        &self,
        serializer: &'se mut Serializer<W>,
    ) -> cbor_event::Result<&'se mut Serializer<W>> {
        match self {
            Salted::SaltedClaim {
                salted_claim,
                salted_claim_bytes_encoding,
            } => {
                let mut salted_claim_inner_se = Serializer::new_vec();
                salted_claim.serialize(&mut salted_claim_inner_se)?;
                let salted_claim_bytes = salted_claim_inner_se.finalize();
                serializer.write_bytes_sz(
                    &salted_claim_bytes,
                    salted_claim_bytes_encoding.to_str_len_sz(salted_claim_bytes.len() as u64),
                )
            }
            Salted::SaltedElement {
                salted_element,
                salted_element_bytes_encoding,
            } => {
                let mut salted_element_inner_se = Serializer::new_vec();
                salted_element.serialize(&mut salted_element_inner_se)?;
                let salted_element_bytes = salted_element_inner_se.finalize();
                serializer.write_bytes_sz(
                    &salted_element_bytes,
                    salted_element_bytes_encoding.to_str_len_sz(salted_element_bytes.len() as u64),
                )
            }
        }
    }
}

impl Deserialize for Salted {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        (|| -> Result<_, DeserializeError> {
            let initial_position = raw.as_mut_ref().stream_position().unwrap();
            let mut errs = Vec::new();
            let deser_variant = (|raw: &mut Deserializer<_>| -> Result<_, DeserializeError> {
                let (salted_claim_bytes, salted_claim_bytes_encoding) = raw.bytes_sz()?;
                let inner_de = &mut Deserializer::from(std::io::Cursor::new(salted_claim_bytes));
                Ok((
                    SaltedClaimItem::deserialize(inner_de)?,
                    StringEncoding::from(salted_claim_bytes_encoding),
                ))
            })(raw);
            match deser_variant {
                Ok((salted_claim, salted_claim_bytes_encoding)) => {
                    return Ok(Self::SaltedClaim {
                        salted_claim,
                        salted_claim_bytes_encoding,
                    })
                }
                Err(e) => {
                    errs.push(e.annotate("SaltedClaim"));
                    raw.as_mut_ref()
                        .seek(SeekFrom::Start(initial_position))
                        .unwrap();
                }
            };
            let deser_variant = (|raw: &mut Deserializer<_>| -> Result<_, DeserializeError> {
                let (salted_element_bytes, salted_element_bytes_encoding) = raw.bytes_sz()?;
                let inner_de = &mut Deserializer::from(std::io::Cursor::new(salted_element_bytes));
                Ok((
                    SaltedElementItem::deserialize(inner_de)?,
                    StringEncoding::from(salted_element_bytes_encoding),
                ))
            })(raw);
            match deser_variant {
                Ok((salted_element, salted_element_bytes_encoding)) => {
                    return Ok(Self::SaltedElement {
                        salted_element,
                        salted_element_bytes_encoding,
                    })
                }
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
        })()
        .map_err(|e| e.annotate("Salted"))
    }
}

impl cbor_event::se::Serialize for SaltedClaimItem {
    fn serialize<'se, W: Write>(
        &self,
        serializer: &'se mut Serializer<W>,
    ) -> cbor_event::Result<&'se mut Serializer<W>> {
        serializer.write_array_sz(
            self.encodings
                .as_ref()
                .map(|encs| encs.len_encoding)
                .unwrap_or_default()
                .to_len_sz(3),
        )?;
        serializer.write_bytes_sz(
            &self.salt,
            self.encodings
                .as_ref()
                .map(|encs| encs.salt_encoding.clone())
                .unwrap_or_default()
                .to_str_len_sz(self.salt.len() as u64),
        )?;
        self.index_1.serialize(serializer)?;
        self.value.serialize(serializer)?;
        self.encodings
            .as_ref()
            .map(|encs| encs.len_encoding)
            .unwrap_or_default()
            .end(serializer)
    }
}

impl Deserialize for SaltedClaimItem {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        let len = raw.array_sz()?;
        let len_encoding: LenEncoding = len.into();
        let mut read_len = CBORReadLen::new(len);
        read_len.read_elems(3)?;
        read_len.finish()?;
        (|| -> Result<_, DeserializeError> {
            let (salt, salt_encoding) = raw
                .bytes_sz()
                .map_err(Into::<DeserializeError>::into)
                .map_err(Into::<DeserializeError>::into)
                .and_then(|(bytes, enc)| {
                    if bytes.len() < 16 || bytes.len() > 16 {
                        Err(DeserializeFailure::RangeCheck {
                            found: bytes.len() as isize,
                            min: Some(16),
                            max: Some(16),
                        }
                        .into())
                    } else {
                        Ok((bytes, StringEncoding::from(enc)))
                    }
                })
                .map_err(|e: DeserializeError| e.annotate("salt"))?;
            let index_1 =
                IntOrText::deserialize(raw).map_err(|e: DeserializeError| e.annotate("index_1"))?;
            let value =
                Anyy::deserialize(raw).map_err(|e: DeserializeError| e.annotate("value"))?;
            match len {
                cbor_event::LenSz::Len(_, _) => (),
                cbor_event::LenSz::Indefinite => match raw.special()? {
                    cbor_event::Special::Break => (),
                    _ => return Err(DeserializeFailure::EndingBreakMissing.into()),
                },
            }
            Ok(SaltedClaimItem {
                salt,
                index_1,
                value,
                encodings: Some(SaltedClaimItemEncoding {
                    len_encoding,
                    salt_encoding,
                }),
            })
        })()
        .map_err(|e| e.annotate("SaltedClaimItem"))
    }
}

impl cbor_event::se::Serialize for SaltedElementItem {
    fn serialize<'se, W: Write>(
        &self,
        serializer: &'se mut Serializer<W>,
    ) -> cbor_event::Result<&'se mut Serializer<W>> {
        serializer.write_array_sz(
            self.encodings
                .as_ref()
                .map(|encs| encs.len_encoding)
                .unwrap_or_default()
                .to_len_sz(2),
        )?;
        serializer.write_bytes_sz(
            &self.salt,
            self.encodings
                .as_ref()
                .map(|encs| encs.salt_encoding.clone())
                .unwrap_or_default()
                .to_str_len_sz(self.salt.len() as u64),
        )?;
        self.value.serialize(serializer)?;
        self.encodings
            .as_ref()
            .map(|encs| encs.len_encoding)
            .unwrap_or_default()
            .end(serializer)
    }
}

impl Deserialize for SaltedElementItem {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        let len = raw.array_sz()?;
        let len_encoding: LenEncoding = len.into();
        let mut read_len = CBORReadLen::new(len);
        read_len.read_elems(2)?;
        read_len.finish()?;
        (|| -> Result<_, DeserializeError> {
            let (salt, salt_encoding) = raw
                .bytes_sz()
                .map_err(Into::<DeserializeError>::into)
                .map_err(Into::<DeserializeError>::into)
                .and_then(|(bytes, enc)| {
                    if bytes.len() < 16 || bytes.len() > 16 {
                        Err(DeserializeFailure::RangeCheck {
                            found: bytes.len() as isize,
                            min: Some(16),
                            max: Some(16),
                        }
                        .into())
                    } else {
                        Ok((bytes, StringEncoding::from(enc)))
                    }
                })
                .map_err(|e: DeserializeError| e.annotate("salt"))?;
            let value =
                Anyy::deserialize(raw).map_err(|e: DeserializeError| e.annotate("value"))?;
            match len {
                cbor_event::LenSz::Len(_, _) => (),
                cbor_event::LenSz::Indefinite => match raw.special()? {
                    cbor_event::Special::Break => (),
                    _ => return Err(DeserializeFailure::EndingBreakMissing.into()),
                },
            }
            Ok(SaltedElementItem {
                salt,
                value,
                encodings: Some(SaltedElementItemEncoding {
                    len_encoding,
                    salt_encoding,
                }),
            })
        })()
        .map_err(|e| e.annotate("SaltedElementItem"))
    }
}

impl cbor_event::se::Serialize for SdCwt {
    fn serialize<'se, W: Write>(
        &self,
        serializer: &'se mut Serializer<W>,
    ) -> cbor_event::Result<&'se mut Serializer<W>> {
        serializer.write_tag_sz(
            18u64,
            fit_sz(
                18u64,
                self.encodings
                    .as_ref()
                    .map(|encs| encs.tag_encoding)
                    .unwrap_or_default(),
            ),
        )?;
        serializer.write_array_sz(
            self.encodings
                .as_ref()
                .map(|encs| encs.len_encoding)
                .unwrap_or_default()
                .to_len_sz(4),
        )?;
        let mut protected_inner_se = Serializer::new_vec();
        self.protected.serialize(&mut protected_inner_se)?;
        let protected_bytes = protected_inner_se.finalize();
        serializer.write_bytes_sz(
            &protected_bytes,
            self.encodings
                .as_ref()
                .map(|encs| encs.protected_bytes_encoding.clone())
                .unwrap_or_default()
                .to_str_len_sz(protected_bytes.len() as u64),
        )?;
        self.unprotected.serialize(serializer)?;
        let mut payload_inner_se = Serializer::new_vec();
        self.payload.serialize(&mut payload_inner_se)?;
        let payload_bytes = payload_inner_se.finalize();
        serializer.write_bytes_sz(
            &payload_bytes,
            self.encodings
                .as_ref()
                .map(|encs| encs.payload_bytes_encoding.clone())
                .unwrap_or_default()
                .to_str_len_sz(payload_bytes.len() as u64),
        )?;
        serializer.write_bytes_sz(
            &self.signature,
            self.encodings
                .as_ref()
                .map(|encs| encs.signature_encoding.clone())
                .unwrap_or_default()
                .to_str_len_sz(self.signature.len() as u64),
        )?;
        self.encodings
            .as_ref()
            .map(|encs| encs.len_encoding)
            .unwrap_or_default()
            .end(serializer)
    }
}

impl Deserialize for SdCwt {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        let (tag, tag_encoding) = raw.tag_sz()?;
        if tag != 18 {
            return Err(DeserializeError::new(
                "SdCwt",
                DeserializeFailure::TagMismatch {
                    found: tag,
                    expected: 18,
                },
            ));
        }
        let len = raw.array_sz()?;
        let len_encoding: LenEncoding = len.into();
        let mut read_len = CBORReadLen::new(len);
        read_len.read_elems(4)?;
        read_len.finish()?;
        (|| -> Result<_, DeserializeError> {
            let (protected, protected_bytes_encoding) = (|| -> Result<_, DeserializeError> {
                let (protected_bytes, protected_bytes_encoding) = raw.bytes_sz()?;
                let inner_de = &mut Deserializer::from(std::io::Cursor::new(protected_bytes));
                Ok((
                    SdProtected::deserialize(inner_de)?,
                    StringEncoding::from(protected_bytes_encoding),
                ))
            })()
            .map_err(|e| e.annotate("protected"))?;
            let unprotected = Unprotected::deserialize(raw)
                .map_err(|e: DeserializeError| e.annotate("unprotected"))?;
            let (payload, payload_bytes_encoding) = (|| -> Result<_, DeserializeError> {
                let (payload_bytes, payload_bytes_encoding) = raw.bytes_sz()?;
                let inner_de = &mut Deserializer::from(std::io::Cursor::new(payload_bytes));
                Ok((
                    SdPayload::deserialize(inner_de)?,
                    StringEncoding::from(payload_bytes_encoding),
                ))
            })()
            .map_err(|e| e.annotate("payload"))?;
            let (signature, signature_encoding) = raw
                .bytes_sz()
                .map_err(Into::<DeserializeError>::into)
                .map(|(bytes, enc)| (bytes, StringEncoding::from(enc)))
                .map_err(|e: DeserializeError| e.annotate("signature"))?;
            match len {
                cbor_event::LenSz::Len(_, _) => (),
                cbor_event::LenSz::Indefinite => match raw.special()? {
                    cbor_event::Special::Break => (),
                    _ => return Err(DeserializeFailure::EndingBreakMissing.into()),
                },
            }
            Ok(SdCwt {
                protected,
                unprotected,
                payload,
                signature,
                encodings: Some(SdCwtEncoding {
                    len_encoding,
                    tag_encoding: Some(tag_encoding),
                    protected_bytes_encoding,
                    payload_bytes_encoding,
                    signature_encoding,
                }),
            })
        })()
        .map_err(|e| e.annotate("SdCwt"))
    }
}

impl cbor_event::se::Serialize for SdPayload {
    fn serialize<'se, W: Write>(
        &self,
        serializer: &'se mut Serializer<W>,
    ) -> cbor_event::Result<&'se mut Serializer<W>> {
        serializer.write_map_sz(
            self.encodings
                .as_ref()
                .map(|encs| encs.len_encoding)
                .unwrap_or_default()
                .to_len_sz(
                    3 + match &self.iss {
                        Some(_) => 1,
                        None => 0,
                    } + match &self.sub {
                        Some(_) => 1,
                        None => 0,
                    } + match &self.exp {
                        Some(_) => 1,
                        None => 0,
                    } + match &self.nbf {
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
                ),
        )?;
        let deser_order = self
            .encodings
            .as_ref()
            .filter(|encs| {
                encs.orig_deser_order.len()
                    == 3 + match &self.iss {
                        Some(_) => 1,
                        None => 0,
                    } + match &self.sub {
                        Some(_) => 1,
                        None => 0,
                    } + match &self.exp {
                        Some(_) => 1,
                        None => 0,
                    } + match &self.nbf {
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
                    }
            })
            .map(|encs| encs.orig_deser_order.clone())
            .unwrap_or_else(|| (0..12).collect());
        for field_index in deser_order {
            match field_index {
                0 => {
                    if let Some(field) = &self.iss {
                        serializer.write_unsigned_integer_sz(
                            1u64,
                            fit_sz(
                                1u64,
                                self.encodings
                                    .as_ref()
                                    .map(|encs| encs.iss_key_encoding)
                                    .unwrap_or_default(),
                            ),
                        )?;
                        serializer.write_text_sz(
                            &field,
                            self.encodings
                                .as_ref()
                                .map(|encs| encs.iss_encoding.clone())
                                .unwrap_or_default()
                                .to_str_len_sz(field.len() as u64),
                        )?;
                    }
                }
                1 => {
                    if let Some(field) = &self.sub {
                        serializer.write_unsigned_integer_sz(
                            2u64,
                            fit_sz(
                                2u64,
                                self.encodings
                                    .as_ref()
                                    .map(|encs| encs.sub_key_encoding)
                                    .unwrap_or_default(),
                            ),
                        )?;
                        serializer.write_text_sz(
                            &field,
                            self.encodings
                                .as_ref()
                                .map(|encs| encs.sub_encoding.clone())
                                .unwrap_or_default()
                                .to_str_len_sz(field.len() as u64),
                        )?;
                    }
                }
                2 => {
                    serializer.write_unsigned_integer_sz(
                        3u64,
                        fit_sz(
                            3u64,
                            self.encodings
                                .as_ref()
                                .map(|encs| encs.aud_key_encoding)
                                .unwrap_or_default(),
                        ),
                    )?;
                    serializer.write_text_sz(
                        &self.aud,
                        self.encodings
                            .as_ref()
                            .map(|encs| encs.aud_encoding.clone())
                            .unwrap_or_default()
                            .to_str_len_sz(self.aud.len() as u64),
                    )?;
                }
                3 => {
                    if let Some(field) = &self.exp {
                        serializer.write_unsigned_integer_sz(
                            4u64,
                            fit_sz(
                                4u64,
                                self.encodings
                                    .as_ref()
                                    .map(|encs| encs.exp_key_encoding)
                                    .unwrap_or_default(),
                            ),
                        )?;
                        field.serialize(serializer)?;
                    }
                }
                4 => {
                    if let Some(field) = &self.nbf {
                        serializer.write_unsigned_integer_sz(
                            5u64,
                            fit_sz(
                                5u64,
                                self.encodings
                                    .as_ref()
                                    .map(|encs| encs.nbf_key_encoding)
                                    .unwrap_or_default(),
                            ),
                        )?;
                        field.serialize(serializer)?;
                    }
                }
                5 => {
                    serializer.write_unsigned_integer_sz(
                        6u64,
                        fit_sz(
                            6u64,
                            self.encodings
                                .as_ref()
                                .map(|encs| encs.iat_key_encoding)
                                .unwrap_or_default(),
                        ),
                    )?;
                    self.iat.serialize(serializer)?;
                }
                7 => {
                    if let Some(field) = &self.cnf {
                        serializer.write_unsigned_integer_sz(
                            8u64,
                            fit_sz(
                                8u64,
                                self.encodings
                                    .as_ref()
                                    .map(|encs| encs.cnf_key_encoding)
                                    .unwrap_or_default(),
                            ),
                        )?;
                        serializer.write_map_sz(
                            self.encodings
                                .as_ref()
                                .map(|encs| encs.cnf_encoding)
                                .unwrap_or_default()
                                .to_len_sz(field.len() as u64),
                        )?;
                        for (key, value) in field.iter() {
                            key.serialize(serializer)?;
                            value.serialize(serializer)?;
                        }
                        self.encodings
                            .as_ref()
                            .map(|encs| encs.cnf_encoding)
                            .unwrap_or_default()
                            .end(serializer)?;
                    }
                }
                6 => {
                    if let Some(field) = &self.cnonce {
                        serializer.write_unsigned_integer_sz(
                            39u64,
                            fit_sz(
                                39u64,
                                self.encodings
                                    .as_ref()
                                    .map(|encs| encs.cnonce_key_encoding)
                                    .unwrap_or_default(),
                            ),
                        )?;
                        serializer.write_bytes_sz(
                            &field,
                            self.encodings
                                .as_ref()
                                .map(|encs| encs.cnonce_encoding.clone())
                                .unwrap_or_default()
                                .to_str_len_sz(field.len() as u64),
                        )?;
                    }
                }
                8 => {
                    if let Some(field) = &self.sd_hash {
                        serializer.write_unsigned_integer_sz(
                            1113u64,
                            fit_sz(
                                1113u64,
                                self.encodings
                                    .as_ref()
                                    .map(|encs| encs.sd_hash_key_encoding)
                                    .unwrap_or_default(),
                            ),
                        )?;
                        serializer.write_bytes_sz(
                            &field,
                            self.encodings
                                .as_ref()
                                .map(|encs| encs.sd_hash_encoding.clone())
                                .unwrap_or_default()
                                .to_str_len_sz(field.len() as u64),
                        )?;
                    }
                }
                9 => {
                    if let Some(field) = &self.sd_alg {
                        serializer.write_unsigned_integer_sz(
                            1114u64,
                            fit_sz(
                                1114u64,
                                self.encodings
                                    .as_ref()
                                    .map(|encs| encs.sd_alg_key_encoding)
                                    .unwrap_or_default(),
                            ),
                        )?;
                        field.serialize(serializer)?;
                    }
                }
                10 => {
                    if let Some(field) = &self.redacted_keys {
                        serializer.write_unsigned_integer_sz(
                            1115u64,
                            fit_sz(
                                1115u64,
                                self.encodings
                                    .as_ref()
                                    .map(|encs| encs.redacted_keys_key_encoding)
                                    .unwrap_or_default(),
                            ),
                        )?;
                        serializer.write_array_sz(
                            self.encodings
                                .as_ref()
                                .map(|encs| encs.redacted_keys_encoding)
                                .unwrap_or_default()
                                .to_len_sz(field.len() as u64),
                        )?;
                        for (i, element) in field.iter().enumerate() {
                            let redacted_keys_elem_encoding = self
                                .encodings
                                .as_ref()
                                .and_then(|encs| encs.redacted_keys_elem_encodings.get(i))
                                .cloned()
                                .unwrap_or_default();
                            serializer.write_bytes_sz(
                                &element,
                                redacted_keys_elem_encoding.to_str_len_sz(element.len() as u64),
                            )?;
                        }
                        self.encodings
                            .as_ref()
                            .map(|encs| encs.redacted_keys_encoding)
                            .unwrap_or_default()
                            .end(serializer)?;
                    }
                }
                11 => {
                    serializer.write_text_sz(
                        "custom",
                        self.encodings
                            .as_ref()
                            .map(|encs| encs.custom_key_encoding.clone())
                            .unwrap_or_default()
                            .to_str_len_sz("custom".len() as u64),
                    )?;
                    serializer.write_map_sz(
                        self.encodings
                            .as_ref()
                            .map(|encs| encs.custom_encoding)
                            .unwrap_or_default()
                            .to_len_sz(self.custom.len() as u64),
                    )?;
                    for (key, value) in self.custom.iter() {
                        key.serialize(serializer)?;
                        value.serialize(serializer)?;
                    }
                    self.encodings
                        .as_ref()
                        .map(|encs| encs.custom_encoding)
                        .unwrap_or_default()
                        .end(serializer)?;
                }
                _ => unreachable!(),
            };
        }
        self.encodings
            .as_ref()
            .map(|encs| encs.len_encoding)
            .unwrap_or_default()
            .end(serializer)
    }
}

impl Deserialize for SdPayload {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        let len = raw.map_sz()?;
        let len_encoding: LenEncoding = len.into();
        let mut read_len = CBORReadLen::new(len);
        read_len.read_elems(3)?;
        (|| -> Result<_, DeserializeError> {
            let mut orig_deser_order = Vec::new();
            let mut iss_encoding = StringEncoding::default();
            let mut iss_key_encoding = None;
            let mut iss = None;
            let mut sub_encoding = StringEncoding::default();
            let mut sub_key_encoding = None;
            let mut sub = None;
            let mut aud_encoding = StringEncoding::default();
            let mut aud_key_encoding = None;
            let mut aud = None;
            let mut exp_key_encoding = None;
            let mut exp = None;
            let mut nbf_key_encoding = None;
            let mut nbf = None;
            let mut iat_key_encoding = None;
            let mut iat = None;
            let mut cnf_encoding = LenEncoding::default();
            let mut cnf_key_encoding = None;
            let mut cnf = None;
            let mut cnonce_encoding = StringEncoding::default();
            let mut cnonce_key_encoding = None;
            let mut cnonce = None;
            let mut sd_hash_encoding = StringEncoding::default();
            let mut sd_hash_key_encoding = None;
            let mut sd_hash = None;
            let mut sd_alg_key_encoding = None;
            let mut sd_alg = None;
            let mut redacted_keys_encoding = LenEncoding::default();
            let mut redacted_keys_elem_encodings = Vec::new();
            let mut redacted_keys_key_encoding = None;
            let mut redacted_keys = None;
            let mut custom_encoding = LenEncoding::default();
            let mut custom_key_encoding = StringEncoding::default();
            let mut custom = None;
            let mut read = 0;
            while match len {
                cbor_event::LenSz::Len(n, _) => read < n,
                cbor_event::LenSz::Indefinite => true,
            } {
                match raw.cbor_type()? {
                    cbor_event::Type::UnsignedInteger => match raw.unsigned_integer_sz()? {
                        (1, key_enc) => {
                            if iss.is_some() {
                                return Err(DeserializeFailure::DuplicateKey(Key::Uint(1)).into());
                            }
                            let (tmp_iss, tmp_iss_encoding) =
                                (|| -> Result<_, DeserializeError> {
                                    read_len.read_elems(1)?;
                                    raw.text_sz()
                                        .map_err(Into::<DeserializeError>::into)
                                        .map(|(s, enc)| (s, StringEncoding::from(enc)))
                                })()
                                .map_err(|e| e.annotate("iss"))?;
                            iss = Some(tmp_iss);
                            iss_encoding = tmp_iss_encoding;
                            iss_key_encoding = Some(key_enc);
                            orig_deser_order.push(0);
                        }
                        (2, key_enc) => {
                            if sub.is_some() {
                                return Err(DeserializeFailure::DuplicateKey(Key::Uint(2)).into());
                            }
                            let (tmp_sub, tmp_sub_encoding) =
                                (|| -> Result<_, DeserializeError> {
                                    read_len.read_elems(1)?;
                                    raw.text_sz()
                                        .map_err(Into::<DeserializeError>::into)
                                        .map(|(s, enc)| (s, StringEncoding::from(enc)))
                                })()
                                .map_err(|e| e.annotate("sub"))?;
                            sub = Some(tmp_sub);
                            sub_encoding = tmp_sub_encoding;
                            sub_key_encoding = Some(key_enc);
                            orig_deser_order.push(1);
                        }
                        (3, key_enc) => {
                            if aud.is_some() {
                                return Err(DeserializeFailure::DuplicateKey(Key::Uint(3)).into());
                            }
                            let (tmp_aud, tmp_aud_encoding) = raw
                                .text_sz()
                                .map_err(Into::<DeserializeError>::into)
                                .map(|(s, enc)| (s, StringEncoding::from(enc)))
                                .map_err(|e: DeserializeError| e.annotate("aud"))?;
                            aud = Some(tmp_aud);
                            aud_encoding = tmp_aud_encoding;
                            aud_key_encoding = Some(key_enc);
                            orig_deser_order.push(2);
                        }
                        (4, key_enc) => {
                            if exp.is_some() {
                                return Err(DeserializeFailure::DuplicateKey(Key::Uint(4)).into());
                            }
                            let tmp_exp = (|| -> Result<_, DeserializeError> {
                                read_len.read_elems(1)?;
                                Int::deserialize(raw)
                            })()
                            .map_err(|e| e.annotate("exp"))?;
                            exp = Some(tmp_exp);
                            exp_key_encoding = Some(key_enc);
                            orig_deser_order.push(3);
                        }
                        (5, key_enc) => {
                            if nbf.is_some() {
                                return Err(DeserializeFailure::DuplicateKey(Key::Uint(5)).into());
                            }
                            let tmp_nbf = (|| -> Result<_, DeserializeError> {
                                read_len.read_elems(1)?;
                                Int::deserialize(raw)
                            })()
                            .map_err(|e| e.annotate("nbf"))?;
                            nbf = Some(tmp_nbf);
                            nbf_key_encoding = Some(key_enc);
                            orig_deser_order.push(4);
                        }
                        (6, key_enc) => {
                            if iat.is_some() {
                                return Err(DeserializeFailure::DuplicateKey(Key::Uint(6)).into());
                            }
                            let tmp_iat = Int::deserialize(raw)
                                .map_err(|e: DeserializeError| e.annotate("iat"))?;
                            iat = Some(tmp_iat);
                            iat_key_encoding = Some(key_enc);
                            orig_deser_order.push(5);
                        }
                        (8, key_enc) => {
                            if cnf.is_some() {
                                return Err(DeserializeFailure::DuplicateKey(Key::Uint(8)).into());
                            }
                            let (tmp_cnf, tmp_cnf_encoding) =
                                (|| -> Result<_, DeserializeError> {
                                    read_len.read_elems(1)?;
                                    let mut cnf_table = OrderedHashMap::new();
                                    let cnf_len = raw.map_sz()?;
                                    let cnf_encoding = cnf_len.into();
                                    while match cnf_len {
                                        cbor_event::LenSz::Len(n, _) => {
                                            (cnf_table.len() as u64) < n
                                        }
                                        cbor_event::LenSz::Indefinite => true,
                                    } {
                                        if raw.cbor_type()? == cbor_event::Type::Special {
                                            assert_eq!(raw.special()?, cbor_event::Special::Break);
                                            break;
                                        }
                                        let cnf_key = Keyy::deserialize(raw)?;
                                        let cnf_value = Anyy::deserialize(raw)?;
                                        if cnf_table.insert(cnf_key.clone(), cnf_value).is_some() {
                                            return Err(DeserializeFailure::DuplicateKey(
                                                Key::Str(String::from(
                                                    "some complicated/unsupported type",
                                                )),
                                            )
                                            .into());
                                        }
                                    }
                                    Ok((cnf_table, cnf_encoding))
                                })()
                                .map_err(|e| e.annotate("cnf"))?;
                            cnf = Some(tmp_cnf);
                            cnf_encoding = tmp_cnf_encoding;
                            cnf_key_encoding = Some(key_enc);
                            orig_deser_order.push(7);
                        }
                        (39, key_enc) => {
                            if cnonce.is_some() {
                                return Err(DeserializeFailure::DuplicateKey(Key::Uint(39)).into());
                            }
                            let (tmp_cnonce, tmp_cnonce_encoding) =
                                (|| -> Result<_, DeserializeError> {
                                    read_len.read_elems(1)?;
                                    raw.bytes_sz()
                                        .map_err(Into::<DeserializeError>::into)
                                        .map(|(bytes, enc)| (bytes, StringEncoding::from(enc)))
                                })()
                                .map_err(|e| e.annotate("cnonce"))?;
                            cnonce = Some(tmp_cnonce);
                            cnonce_encoding = tmp_cnonce_encoding;
                            cnonce_key_encoding = Some(key_enc);
                            orig_deser_order.push(6);
                        }
                        (1113, key_enc) => {
                            if sd_hash.is_some() {
                                return Err(
                                    DeserializeFailure::DuplicateKey(Key::Uint(1113)).into()
                                );
                            }
                            let (tmp_sd_hash, tmp_sd_hash_encoding) =
                                (|| -> Result<_, DeserializeError> {
                                    read_len.read_elems(1)?;
                                    raw.bytes_sz()
                                        .map_err(Into::<DeserializeError>::into)
                                        .map(|(bytes, enc)| (bytes, StringEncoding::from(enc)))
                                })()
                                .map_err(|e| e.annotate("sd_hash"))?;
                            sd_hash = Some(tmp_sd_hash);
                            sd_hash_encoding = tmp_sd_hash_encoding;
                            sd_hash_key_encoding = Some(key_enc);
                            orig_deser_order.push(8);
                        }
                        (1114, key_enc) => {
                            if sd_alg.is_some() {
                                return Err(
                                    DeserializeFailure::DuplicateKey(Key::Uint(1114)).into()
                                );
                            }
                            let tmp_sd_alg = (|| -> Result<_, DeserializeError> {
                                read_len.read_elems(1)?;
                                Int::deserialize(raw)
                            })()
                            .map_err(|e| e.annotate("sd_alg"))?;
                            sd_alg = Some(tmp_sd_alg);
                            sd_alg_key_encoding = Some(key_enc);
                            orig_deser_order.push(9);
                        }
                        (1115, key_enc) => {
                            if redacted_keys.is_some() {
                                return Err(
                                    DeserializeFailure::DuplicateKey(Key::Uint(1115)).into()
                                );
                            }
                            let (
                                tmp_redacted_keys,
                                tmp_redacted_keys_encoding,
                                tmp_redacted_keys_elem_encodings,
                            ) = (|| -> Result<_, DeserializeError> {
                                read_len.read_elems(1)?;
                                let mut redacted_keys_arr = Vec::new();
                                let len = raw.array_sz()?;
                                let redacted_keys_encoding = len.into();
                                let mut redacted_keys_elem_encodings = Vec::new();
                                while match len {
                                    cbor_event::LenSz::Len(n, _) => {
                                        (redacted_keys_arr.len() as u64) < n
                                    }
                                    cbor_event::LenSz::Indefinite => true,
                                } {
                                    if raw.cbor_type()? == cbor_event::Type::Special {
                                        assert_eq!(raw.special()?, cbor_event::Special::Break);
                                        break;
                                    }
                                    let (redacted_keys_elem, redacted_keys_elem_encoding) = raw
                                        .bytes_sz()
                                        .map(|(bytes, enc)| (bytes, StringEncoding::from(enc)))?;
                                    redacted_keys_arr.push(redacted_keys_elem);
                                    redacted_keys_elem_encodings.push(redacted_keys_elem_encoding);
                                }
                                Ok((
                                    redacted_keys_arr,
                                    redacted_keys_encoding,
                                    redacted_keys_elem_encodings,
                                ))
                            })()
                            .map_err(|e| e.annotate("redacted_keys"))?;
                            redacted_keys = Some(tmp_redacted_keys);
                            redacted_keys_encoding = tmp_redacted_keys_encoding;
                            redacted_keys_elem_encodings = tmp_redacted_keys_elem_encodings;
                            redacted_keys_key_encoding = Some(key_enc);
                            orig_deser_order.push(10);
                        }
                        (unknown_key, _enc) => {
                            return Err(
                                DeserializeFailure::UnknownKey(Key::Uint(unknown_key)).into()
                            )
                        }
                    },
                    cbor_event::Type::Text => {
                        let (text_key, key_enc) = raw.text_sz()?;
                        match text_key.as_str() {
                            "custom" => {
                                if custom.is_some() {
                                    return Err(DeserializeFailure::DuplicateKey(Key::Str(
                                        "custom".into(),
                                    ))
                                    .into());
                                }
                                let (tmp_custom, tmp_custom_encoding) =
                                    (|| -> Result<_, DeserializeError> {
                                        let mut custom_table = OrderedHashMap::new();
                                        let custom_len = raw.map_sz()?;
                                        let custom_encoding = custom_len.into();
                                        while match custom_len {
                                            cbor_event::LenSz::Len(n, _) => {
                                                (custom_table.len() as u64) < n
                                            }
                                            cbor_event::LenSz::Indefinite => true,
                                        } {
                                            if raw.cbor_type()? == cbor_event::Type::Special {
                                                assert_eq!(
                                                    raw.special()?,
                                                    cbor_event::Special::Break
                                                );
                                                break;
                                            }
                                            let custom_key = Keyy::deserialize(raw)?;
                                            let custom_value = Anyy::deserialize(raw)?;
                                            if custom_table
                                                .insert(custom_key.clone(), custom_value)
                                                .is_some()
                                            {
                                                return Err(DeserializeFailure::DuplicateKey(
                                                    Key::Str(String::from(
                                                        "some complicated/unsupported type",
                                                    )),
                                                )
                                                .into());
                                            }
                                        }
                                        Ok((custom_table, custom_encoding))
                                    })()
                                    .map_err(|e| e.annotate("custom"))?;
                                custom = Some(tmp_custom);
                                custom_encoding = tmp_custom_encoding;
                                custom_key_encoding = StringEncoding::from(key_enc);
                                orig_deser_order.push(11);
                            }
                            unknown_key => {
                                return Err(DeserializeFailure::UnknownKey(Key::Str(
                                    unknown_key.to_owned(),
                                ))
                                .into())
                            }
                        }
                    }
                    cbor_event::Type::Special => match len {
                        cbor_event::LenSz::Len(_, _) => {
                            return Err(DeserializeFailure::BreakInDefiniteLen.into())
                        }
                        cbor_event::LenSz::Indefinite => match raw.special()? {
                            cbor_event::Special::Break => break,
                            _ => return Err(DeserializeFailure::EndingBreakMissing.into()),
                        },
                    },
                    other_type => {
                        return Err(DeserializeFailure::UnexpectedKeyType(other_type).into())
                    }
                }
                read += 1;
            }
            let aud = match aud {
                Some(x) => x,
                None => return Err(DeserializeFailure::MandatoryFieldMissing(Key::Uint(3)).into()),
            };
            let iat = match iat {
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
                exp,
                nbf,
                iat,
                cnonce,
                cnf,
                sd_hash,
                sd_alg,
                redacted_keys,
                custom,
                encodings: Some(SdPayloadEncoding {
                    len_encoding,
                    orig_deser_order,
                    iss_key_encoding,
                    iss_encoding,
                    sub_key_encoding,
                    sub_encoding,
                    aud_key_encoding,
                    aud_encoding,
                    exp_key_encoding,
                    nbf_key_encoding,
                    iat_key_encoding,
                    cnonce_key_encoding,
                    cnonce_encoding,
                    cnf_key_encoding,
                    cnf_encoding,
                    sd_hash_key_encoding,
                    sd_hash_encoding,
                    sd_alg_key_encoding,
                    redacted_keys_key_encoding,
                    redacted_keys_encoding,
                    redacted_keys_elem_encodings,
                    custom_key_encoding,
                    custom_encoding,
                }),
            })
        })()
        .map_err(|e| e.annotate("SdPayload"))
    }
}

impl cbor_event::se::Serialize for SdProtected {
    fn serialize<'se, W: Write>(
        &self,
        serializer: &'se mut Serializer<W>,
    ) -> cbor_event::Result<&'se mut Serializer<W>> {
        serializer.write_map_sz(
            self.encodings
                .as_ref()
                .map(|encs| encs.len_encoding)
                .unwrap_or_default()
                .to_len_sz(3),
        )?;
        let deser_order = self
            .encodings
            .as_ref()
            .filter(|encs| encs.orig_deser_order.len() == 3)
            .map(|encs| encs.orig_deser_order.clone())
            .unwrap_or_else(|| (0..3).collect());
        for field_index in deser_order {
            match field_index {
                0 => {
                    serializer.write_unsigned_integer_sz(
                        1u64,
                        fit_sz(
                            1u64,
                            self.encodings
                                .as_ref()
                                .map(|encs| encs.alg_key_encoding)
                                .unwrap_or_default(),
                        ),
                    )?;
                    self.alg.serialize(serializer)?;
                }
                1 => {
                    serializer.write_unsigned_integer_sz(
                        16u64,
                        fit_sz(
                            16u64,
                            self.encodings
                                .as_ref()
                                .map(|encs| encs.typ_key_encoding)
                                .unwrap_or_default(),
                        ),
                    )?;
                    serializer.write_text_sz(
                        &self.typ,
                        self.encodings
                            .as_ref()
                            .map(|encs| encs.typ_encoding.clone())
                            .unwrap_or_default()
                            .to_str_len_sz(self.typ.len() as u64),
                    )?;
                }
                2 => {
                    serializer.write_text_sz(
                        "custom",
                        self.encodings
                            .as_ref()
                            .map(|encs| encs.custom_key_encoding.clone())
                            .unwrap_or_default()
                            .to_str_len_sz("custom".len() as u64),
                    )?;
                    serializer.write_map_sz(
                        self.encodings
                            .as_ref()
                            .map(|encs| encs.custom_encoding)
                            .unwrap_or_default()
                            .to_len_sz(self.custom.len() as u64),
                    )?;
                    for (key, value) in self.custom.iter() {
                        key.serialize(serializer)?;
                        value.serialize(serializer)?;
                    }
                    self.encodings
                        .as_ref()
                        .map(|encs| encs.custom_encoding)
                        .unwrap_or_default()
                        .end(serializer)?;
                }
                _ => unreachable!(),
            };
        }
        self.encodings
            .as_ref()
            .map(|encs| encs.len_encoding)
            .unwrap_or_default()
            .end(serializer)
    }
}

impl Deserialize for SdProtected {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        let len = raw.map_sz()?;
        let len_encoding: LenEncoding = len.into();
        let mut read_len = CBORReadLen::new(len);
        read_len.read_elems(3)?;
        read_len.finish()?;
        (|| -> Result<_, DeserializeError> {
            let mut orig_deser_order = Vec::new();
            let mut alg_key_encoding = None;
            let mut alg = None;
            let mut typ_encoding = StringEncoding::default();
            let mut typ_key_encoding = None;
            let mut typ = None;
            let mut custom_encoding = LenEncoding::default();
            let mut custom_key_encoding = StringEncoding::default();
            let mut custom = None;
            let mut read = 0;
            while match len {
                cbor_event::LenSz::Len(n, _) => read < n,
                cbor_event::LenSz::Indefinite => true,
            } {
                match raw.cbor_type()? {
                    cbor_event::Type::UnsignedInteger => match raw.unsigned_integer_sz()? {
                        (1, key_enc) => {
                            if alg.is_some() {
                                return Err(DeserializeFailure::DuplicateKey(Key::Uint(1)).into());
                            }
                            let tmp_alg = Int::deserialize(raw)
                                .map_err(|e: DeserializeError| e.annotate("alg"))?;
                            alg = Some(tmp_alg);
                            alg_key_encoding = Some(key_enc);
                            orig_deser_order.push(0);
                        }
                        (16, key_enc) => {
                            if typ.is_some() {
                                return Err(DeserializeFailure::DuplicateKey(Key::Uint(16)).into());
                            }
                            let (tmp_typ, tmp_typ_encoding) = raw
                                .text_sz()
                                .map_err(Into::<DeserializeError>::into)
                                .map(|(s, enc)| (s, StringEncoding::from(enc)))
                                .map_err(|e: DeserializeError| e.annotate("typ"))?;
                            typ = Some(tmp_typ);
                            typ_encoding = tmp_typ_encoding;
                            typ_key_encoding = Some(key_enc);
                            orig_deser_order.push(1);
                        }
                        (unknown_key, _enc) => {
                            return Err(
                                DeserializeFailure::UnknownKey(Key::Uint(unknown_key)).into()
                            )
                        }
                    },
                    cbor_event::Type::Text => {
                        let (text_key, key_enc) = raw.text_sz()?;
                        match text_key.as_str() {
                            "custom" => {
                                if custom.is_some() {
                                    return Err(DeserializeFailure::DuplicateKey(Key::Str(
                                        "custom".into(),
                                    ))
                                    .into());
                                }
                                let (tmp_custom, tmp_custom_encoding) =
                                    (|| -> Result<_, DeserializeError> {
                                        let mut custom_table = OrderedHashMap::new();
                                        let custom_len = raw.map_sz()?;
                                        let custom_encoding = custom_len.into();
                                        while match custom_len {
                                            cbor_event::LenSz::Len(n, _) => {
                                                (custom_table.len() as u64) < n
                                            }
                                            cbor_event::LenSz::Indefinite => true,
                                        } {
                                            if raw.cbor_type()? == cbor_event::Type::Special {
                                                assert_eq!(
                                                    raw.special()?,
                                                    cbor_event::Special::Break
                                                );
                                                break;
                                            }
                                            let custom_key = Keyy::deserialize(raw)?;
                                            let custom_value = Anyy::deserialize(raw)?;
                                            if custom_table
                                                .insert(custom_key.clone(), custom_value)
                                                .is_some()
                                            {
                                                return Err(DeserializeFailure::DuplicateKey(
                                                    Key::Str(String::from(
                                                        "some complicated/unsupported type",
                                                    )),
                                                )
                                                .into());
                                            }
                                        }
                                        Ok((custom_table, custom_encoding))
                                    })()
                                    .map_err(|e| e.annotate("custom"))?;
                                custom = Some(tmp_custom);
                                custom_encoding = tmp_custom_encoding;
                                custom_key_encoding = StringEncoding::from(key_enc);
                                orig_deser_order.push(2);
                            }
                            unknown_key => {
                                return Err(DeserializeFailure::UnknownKey(Key::Str(
                                    unknown_key.to_owned(),
                                ))
                                .into())
                            }
                        }
                    }
                    cbor_event::Type::Special => match len {
                        cbor_event::LenSz::Len(_, _) => {
                            return Err(DeserializeFailure::BreakInDefiniteLen.into())
                        }
                        cbor_event::LenSz::Indefinite => match raw.special()? {
                            cbor_event::Special::Break => break,
                            _ => return Err(DeserializeFailure::EndingBreakMissing.into()),
                        },
                    },
                    other_type => {
                        return Err(DeserializeFailure::UnexpectedKeyType(other_type).into())
                    }
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
            Ok(Self {
                alg,
                typ,
                custom,
                encodings: Some(SdProtectedEncoding {
                    len_encoding,
                    orig_deser_order,
                    alg_key_encoding,
                    typ_key_encoding,
                    typ_encoding,
                    custom_key_encoding,
                    custom_encoding,
                }),
            })
        })()
        .map_err(|e| e.annotate("SdProtected"))
    }
}

impl cbor_event::se::Serialize for Unprotected {
    fn serialize<'se, W: Write>(
        &self,
        serializer: &'se mut Serializer<W>,
    ) -> cbor_event::Result<&'se mut Serializer<W>> {
        serializer.write_map_sz(
            self.encodings
                .as_ref()
                .map(|encs| encs.len_encoding)
                .unwrap_or_default()
                .to_len_sz(
                    1 + match &self.sd_claims {
                        Some(_) => 1,
                        None => 0,
                    } + match &self.sd_kbt {
                        Some(_) => 1,
                        None => 0,
                    },
                ),
        )?;
        let deser_order = self
            .encodings
            .as_ref()
            .filter(|encs| {
                encs.orig_deser_order.len()
                    == 1 + match &self.sd_claims {
                        Some(_) => 1,
                        None => 0,
                    } + match &self.sd_kbt {
                        Some(_) => 1,
                        None => 0,
                    }
            })
            .map(|encs| encs.orig_deser_order.clone())
            .unwrap_or_else(|| (0..3).collect());
        for field_index in deser_order {
            match field_index {
                0 => {
                    if let Some(field) = &self.sd_claims {
                        serializer.write_unsigned_integer_sz(
                            1111u64,
                            fit_sz(
                                1111u64,
                                self.encodings
                                    .as_ref()
                                    .map(|encs| encs.sd_claims_key_encoding)
                                    .unwrap_or_default(),
                            ),
                        )?;
                        serializer.write_array_sz(
                            self.encodings
                                .as_ref()
                                .map(|encs| encs.sd_claims_encoding)
                                .unwrap_or_default()
                                .to_len_sz(field.len() as u64),
                        )?;
                        for element in field.iter() {
                            element.serialize(serializer)?;
                        }
                        self.encodings
                            .as_ref()
                            .map(|encs| encs.sd_claims_encoding)
                            .unwrap_or_default()
                            .end(serializer)?;
                    }
                }
                1 => {
                    if let Some(field) = &self.sd_kbt {
                        serializer.write_unsigned_integer_sz(
                            1112u64,
                            fit_sz(
                                1112u64,
                                self.encodings
                                    .as_ref()
                                    .map(|encs| encs.sd_kbt_key_encoding)
                                    .unwrap_or_default(),
                            ),
                        )?;
                        serializer.write_bytes_sz(
                            &field,
                            self.encodings
                                .as_ref()
                                .map(|encs| encs.sd_kbt_encoding.clone())
                                .unwrap_or_default()
                                .to_str_len_sz(field.len() as u64),
                        )?;
                    }
                }
                2 => {
                    serializer.write_text_sz(
                        "custom",
                        self.encodings
                            .as_ref()
                            .map(|encs| encs.custom_key_encoding.clone())
                            .unwrap_or_default()
                            .to_str_len_sz("custom".len() as u64),
                    )?;
                    serializer.write_map_sz(
                        self.encodings
                            .as_ref()
                            .map(|encs| encs.custom_encoding)
                            .unwrap_or_default()
                            .to_len_sz(self.custom.len() as u64),
                    )?;
                    for (key, value) in self.custom.iter() {
                        key.serialize(serializer)?;
                        value.serialize(serializer)?;
                    }
                    self.encodings
                        .as_ref()
                        .map(|encs| encs.custom_encoding)
                        .unwrap_or_default()
                        .end(serializer)?;
                }
                _ => unreachable!(),
            };
        }
        self.encodings
            .as_ref()
            .map(|encs| encs.len_encoding)
            .unwrap_or_default()
            .end(serializer)
    }
}

impl Deserialize for Unprotected {
    fn deserialize<R: BufRead + Seek>(raw: &mut Deserializer<R>) -> Result<Self, DeserializeError> {
        let len = raw.map_sz()?;
        let len_encoding: LenEncoding = len.into();
        let mut read_len = CBORReadLen::new(len);
        read_len.read_elems(1)?;
        (|| -> Result<_, DeserializeError> {
            let mut orig_deser_order = Vec::new();
            let mut sd_claims_encoding = LenEncoding::default();
            let mut sd_claims_key_encoding = None;
            let mut sd_claims = None;
            let mut sd_kbt_encoding = StringEncoding::default();
            let mut sd_kbt_key_encoding = None;
            let mut sd_kbt = None;
            let mut custom_encoding = LenEncoding::default();
            let mut custom_key_encoding = StringEncoding::default();
            let mut custom = None;
            let mut read = 0;
            while match len {
                cbor_event::LenSz::Len(n, _) => read < n,
                cbor_event::LenSz::Indefinite => true,
            } {
                match raw.cbor_type()? {
                    cbor_event::Type::UnsignedInteger => match raw.unsigned_integer_sz()? {
                        (1111, key_enc) => {
                            if sd_claims.is_some() {
                                return Err(
                                    DeserializeFailure::DuplicateKey(Key::Uint(1111)).into()
                                );
                            }
                            let (tmp_sd_claims, tmp_sd_claims_encoding) =
                                (|| -> Result<_, DeserializeError> {
                                    read_len.read_elems(1)?;
                                    let mut sd_claims_arr = Vec::new();
                                    let len = raw.array_sz()?;
                                    let sd_claims_encoding = len.into();
                                    while match len {
                                        cbor_event::LenSz::Len(n, _) => {
                                            (sd_claims_arr.len() as u64) < n
                                        }
                                        cbor_event::LenSz::Indefinite => true,
                                    } {
                                        if raw.cbor_type()? == cbor_event::Type::Special {
                                            assert_eq!(raw.special()?, cbor_event::Special::Break);
                                            break;
                                        }
                                        sd_claims_arr.push(Salted::deserialize(raw)?);
                                    }
                                    Ok((sd_claims_arr, sd_claims_encoding))
                                })()
                                .map_err(|e| e.annotate("sd_claims"))?;
                            sd_claims = Some(tmp_sd_claims);
                            sd_claims_encoding = tmp_sd_claims_encoding;
                            sd_claims_key_encoding = Some(key_enc);
                            orig_deser_order.push(0);
                        }
                        (1112, key_enc) => {
                            if sd_kbt.is_some() {
                                return Err(
                                    DeserializeFailure::DuplicateKey(Key::Uint(1112)).into()
                                );
                            }
                            let (tmp_sd_kbt, tmp_sd_kbt_encoding) =
                                (|| -> Result<_, DeserializeError> {
                                    read_len.read_elems(1)?;
                                    raw.bytes_sz()
                                        .map_err(Into::<DeserializeError>::into)
                                        .map(|(bytes, enc)| (bytes, StringEncoding::from(enc)))
                                })()
                                .map_err(|e| e.annotate("sd_kbt"))?;
                            sd_kbt = Some(tmp_sd_kbt);
                            sd_kbt_encoding = tmp_sd_kbt_encoding;
                            sd_kbt_key_encoding = Some(key_enc);
                            orig_deser_order.push(1);
                        }
                        (unknown_key, _enc) => {
                            return Err(
                                DeserializeFailure::UnknownKey(Key::Uint(unknown_key)).into()
                            )
                        }
                    },
                    cbor_event::Type::Text => {
                        let (text_key, key_enc) = raw.text_sz()?;
                        match text_key.as_str() {
                            "custom" => {
                                if custom.is_some() {
                                    return Err(DeserializeFailure::DuplicateKey(Key::Str(
                                        "custom".into(),
                                    ))
                                    .into());
                                }
                                let (tmp_custom, tmp_custom_encoding) =
                                    (|| -> Result<_, DeserializeError> {
                                        let mut custom_table = OrderedHashMap::new();
                                        let custom_len = raw.map_sz()?;
                                        let custom_encoding = custom_len.into();
                                        while match custom_len {
                                            cbor_event::LenSz::Len(n, _) => {
                                                (custom_table.len() as u64) < n
                                            }
                                            cbor_event::LenSz::Indefinite => true,
                                        } {
                                            if raw.cbor_type()? == cbor_event::Type::Special {
                                                assert_eq!(
                                                    raw.special()?,
                                                    cbor_event::Special::Break
                                                );
                                                break;
                                            }
                                            let custom_key = Keyy::deserialize(raw)?;
                                            let custom_value = Anyy::deserialize(raw)?;
                                            if custom_table
                                                .insert(custom_key.clone(), custom_value)
                                                .is_some()
                                            {
                                                return Err(DeserializeFailure::DuplicateKey(
                                                    Key::Str(String::from(
                                                        "some complicated/unsupported type",
                                                    )),
                                                )
                                                .into());
                                            }
                                        }
                                        Ok((custom_table, custom_encoding))
                                    })()
                                    .map_err(|e| e.annotate("custom"))?;
                                custom = Some(tmp_custom);
                                custom_encoding = tmp_custom_encoding;
                                custom_key_encoding = StringEncoding::from(key_enc);
                                orig_deser_order.push(2);
                            }
                            unknown_key => {
                                return Err(DeserializeFailure::UnknownKey(Key::Str(
                                    unknown_key.to_owned(),
                                ))
                                .into())
                            }
                        }
                    }
                    cbor_event::Type::Special => match len {
                        cbor_event::LenSz::Len(_, _) => {
                            return Err(DeserializeFailure::BreakInDefiniteLen.into())
                        }
                        cbor_event::LenSz::Indefinite => match raw.special()? {
                            cbor_event::Special::Break => break,
                            _ => return Err(DeserializeFailure::EndingBreakMissing.into()),
                        },
                    },
                    other_type => {
                        return Err(DeserializeFailure::UnexpectedKeyType(other_type).into())
                    }
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
                encodings: Some(UnprotectedEncoding {
                    len_encoding,
                    orig_deser_order,
                    sd_claims_key_encoding,
                    sd_claims_encoding,
                    sd_kbt_key_encoding,
                    sd_kbt_encoding,
                    custom_key_encoding,
                    custom_encoding,
                }),
            })
        })()
        .map_err(|e| e.annotate("Unprotected"))
    }
}
