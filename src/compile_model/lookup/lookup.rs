use bitflags::bitflags;
use endian_codec::{PackedSize, EncodeBE, DecodeBE};

use crate::util::*;

use crate::compile_model::util::decode::*;
use crate::compile_model::util::encode::*;


bitflags! {
    pub struct LookupFlags: u16 {
        const RIGHT_TO_LEFT = 0x0001;
        const IGNORE_BASE_GLYPHS = 0x0002;
        const IGNORE_LIGATURES = 0x0004;
        const IGNORE_MARKS = 0x0008;
        const USE_MARK_FILTERING_SET = 0x0010;

        const MARK_ATTACHMENT_TYPE = 0xFF00;
    }
}

#[derive(Debug)]
pub struct Lookup<T> {
    pub lookup_flags: LookupFlags,
    pub mark_filtering_set: Option<u16>,

    pub subtables: Vec<T>,
}

impl<T> Lookup<T> {
    pub fn new() -> Self {
        Self {
            lookup_flags: LookupFlags::empty(),
            mark_filtering_set: None,

            subtables: Vec::new()
        }
    }

    pub fn get_subtable_variant<V>(&mut self, skip: usize) -> &mut V
        where V: VariantExt<T> + Default + Into<T>
    {
        let idx = self.subtables.iter().enumerate()
            .filter_map(|(idx, subtable)| {
                V::get_variant(subtable)
                    .map(|_| idx)
            })
            .skip(skip)
            .next()

            .unwrap_or_else(|| {
                let idx = self.subtables.len();
                self.subtables.push(V::default().into());
                idx
            });

        V::get_variant_mut(&mut self.subtables[idx]).unwrap()
    }
}

#[derive(Debug, PackedSize, EncodeBE, DecodeBE)]
struct LookupTableHeader {
    pub lookup_type: u16,
    pub lookup_flags: u16,
    pub subtable_count: u16
}

impl<T: TTFDecode> TTFDecode for Lookup<T> {
    #[inline]
    fn ttf_decode(bytes: &[u8]) -> DecodeResult<Self> {
        let header = decode_from_slice::<LookupTableHeader>(bytes);

        let lookup_flags = LookupFlags::from_bits_truncate(header.lookup_flags);

        let subtables =
            decode_from_pool(header.subtable_count, &bytes[LookupTableHeader::PACKED_LEN..])
            .map(|offset: u16|
                T::ttf_decode(&bytes[offset as usize..]))
            .collect::<DecodeResult<_>>()?;

        let mark_filtering_set =
            if lookup_flags.contains(LookupFlags::USE_MARK_FILTERING_SET) {
                Some(decode_u16_be(bytes,
                    LookupTableHeader::PACKED_LEN
                    + (header.subtable_count as usize * 2)))
            } else {
                None
            };

        Ok(Lookup {
            lookup_flags,
            mark_filtering_set,

            subtables
        })
    }
}

impl<T: TTFEncode> Lookup<T> {
    pub fn ttf_encode_with_lookup_type(&self, buf: &mut EncodeBuf, lookup_type: u16) -> EncodeResult<usize> {
        let start = buf.bytes.len();
        let mut flags = self.lookup_flags;

        flags.set(LookupFlags::USE_MARK_FILTERING_SET,
            self.mark_filtering_set.is_some());

        let header = LookupTableHeader {
            lookup_type: lookup_type,
            lookup_flags: self.lookup_flags.bits(),
            subtable_count: self.subtables.len() as u16
        };

        buf.append(&header)?;

        let mut subtable_offset_start = buf.bytes.len();
        buf.bytes.resize(subtable_offset_start + (u16::PACKED_LEN * self.subtables.len()), 0u8);

        if let Some(mfs) = self.mark_filtering_set {
            buf.append(&mfs)?;
        }

        for subtable in &self.subtables {
            let offset = (buf.append(subtable)? - start) as u16;
            buf.encode_at(&offset, subtable_offset_start)?;

            subtable_offset_start += u16::PACKED_LEN;
        }

        Ok(start)
    }
}
