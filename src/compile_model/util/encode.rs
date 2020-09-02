use std::any::type_name;
use std::hash::Hash;
use std::collections::HashMap;

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

    pub(crate) fn encode_pool_dedup<'a, Item, Record, RF, IWF>(&mut self,
        table_start: usize, mut record_offset: usize, items: impl Iterator<Item = &'a Item>,
        record_for_offset: RF, write_item: IWF) -> EncodeResult<()>

        where Item: 'a + Hash + Eq,
              Record: EncodeBE,
              RF: Fn(u16) -> Record,
              IWF: Fn(&mut EncodeBuf, &'a Item) -> EncodeResult<()>
    {
        let mut dedup = HashMap::new();

        for item in items {
            let item_offset =
                if let Some(item_offset) = dedup.get(item) {
                    *item_offset
                } else {
                    let item_offset = (self.bytes.len() - table_start) as u16;

                    dedup.insert(item, item_offset);
                    write_item(self, item)?;

                    item_offset
                };

            self.encode_at(&record_for_offset(item_offset), record_offset)?;
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
