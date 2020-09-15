use std::any::type_name;
use std::ops::Range;
use std::hash::{
    Hasher,
    BuildHasher
};

use hashbrown::{
    HashMap,
    hash_map::RawEntryMut
};

use endian_codec::EncodeBE;

pub use crate::compile_model::error::{
    EncodeError,
    EncodeResult
};


pub struct EncodeBuf {
    pub(crate) bytes: Vec<u8>
}

impl EncodeBuf {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new()
        }
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &*self.bytes
    }

    #[inline]
    pub(crate) fn append<T: TTFEncode>(&mut self, val: &T) -> EncodeResult<usize> {
        val.ttf_encode(self)
    }

    #[inline]
    pub(crate) fn encode_at<T: EncodeBE>(&mut self, val: &T, start: usize)
            -> EncodeResult<usize> {
        let end = start + T::PACKED_LEN;

        if end > self.bytes.len() {
            return Err(EncodeError::BufferTooSmallForType(type_name::<T>()));
        }

        val.encode_as_be_bytes(&mut self.bytes[start..end]);
        Ok(start)
    }

    pub(crate) fn defer_header_encode<Header, HF, PF, PR>(&mut self, header_func: HF, encode_pool: PF) -> EncodeResult<usize>
        where Header: EncodeBE,
              HF: FnOnce(&mut EncodeBuf) -> EncodeResult<Header>,
              PF: FnOnce(&mut EncodeBuf) -> EncodeResult<PR>
    {
        let start = self.bytes.len();
        self.bytes.resize(start + Header::PACKED_LEN, 0u8);

        encode_pool(self)?;

        let header = header_func(self)?;
        self.encode_at(&header, start)
    }

    pub(crate) fn encode_pool<'a, Item, I, Record, RF, IWF, IWFR>(&mut self,
        table_start: usize, items: I, record_for_offset: RF, write_item: IWF) -> EncodeResult<()>

        where Item: 'a,
              I: Iterator<Item = Item> + ExactSizeIterator,
              Record: EncodeBE,
              RF: Fn(u16, &Item) -> Record,
              IWF: Fn(&mut EncodeBuf, &Item) -> EncodeResult<IWFR>
    {
        let mut record_offset = self.bytes.len();
        self.bytes.resize(record_offset + (items.len() * Record::PACKED_LEN), 0u8);

        let mut dedup = HashMap::new();

        for item in items {
            let item_start = self.bytes.len();

            write_item(self, &item)?;

            let item_span = item_start..self.bytes.len();

            let item_encoded_hash = {
                let mut hasher = dedup.hasher().build_hasher();
                hasher.write(&self.bytes[item_span.clone()]);
                hasher.finish()
            };

            let entry = dedup.raw_entry_mut()
                .from_hash(item_encoded_hash,
                    |span: &Range<usize>| {
                        self.bytes[span.clone()]
                            .eq(&self.bytes[item_span.clone()])
                    });

            let item_offset = match entry {
                RawEntryMut::Occupied(e) => {
                    self.bytes.truncate(item_start);
                    e.key().start
                }

                RawEntryMut::Vacant(e) => {
                    e.insert_hashed_nocheck(item_encoded_hash, item_span.clone(), ());
                    item_start
                }
            };

            let item_offset = (item_offset - table_start) as u16;

            self.encode_at(
                &record_for_offset(item_offset, &item),
                record_offset)?;

            record_offset += Record::PACKED_LEN;
        }

        Ok(())
    }
}


pub trait TTFEncode {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize>;
}

impl<T: EncodeBE> TTFEncode for T {
    fn ttf_encode(&self, buf: &mut EncodeBuf) -> EncodeResult<usize> {
        let start = buf.bytes.len();
        let end = start + T::PACKED_LEN;

        buf.bytes.resize(end, 0u8);
        self.encode_as_be_bytes(&mut buf.bytes[start..end]);

        Ok(start)
    }
}
