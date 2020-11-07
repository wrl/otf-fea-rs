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


use crate::compile_model::{
    SourceMap,
    CompiledEntry
};

use crate::SourceSpan;

use crate::glyph_order::*;
pub use crate::compile_model::error::{
    EncodeError,
    EncodeResult
};

pub struct EncodeBuf<'a> {
    pub(crate) bytes: Vec<u8>,
    pub(crate) source_map: SourceMap,
    pub(crate) _glyph_order: &'a GlyphOrder,

    should_optimize_filesize: bool
}

impl<'a> EncodeBuf<'a> {
    pub fn new_with_glyph_order(glyph_order: &'a GlyphOrder) -> Self {
        Self {
            bytes: Vec::new(),
            source_map: SourceMap::new(),
            _glyph_order: glyph_order,

            should_optimize_filesize: false
        }
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &*self.bytes
    }

    #[inline]
    pub(crate) fn should_optimize_filesize(&self) -> bool {
        self.should_optimize_filesize
    }

    #[inline]
    pub(crate) fn reserve_bytes(&mut self, to_reserve: usize) {
        self.bytes.resize(self.bytes.len() + to_reserve, 0u8);
    }

    #[inline]
    pub(crate) fn add_source_map_entry(&mut self, span: &SourceSpan, entry: CompiledEntry) {
        if self.should_optimize_filesize() {
            return
        }

        self.source_map.entry(span.clone())
            .or_default()
            .insert(entry);
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

    fn encode_pool_internal<'b, Item, I, Record, RF, IWF, IWFR>
        (&mut self, table_start: usize, mut record_offset: usize, items: I, record_for_offset: RF, write_item: IWF)
            -> EncodeResult<()>

        where Item: 'b,
              I: Iterator<Item = Item> + ExactSizeIterator,
              Record: EncodeBE,
              RF: Fn(u16, &Item) -> Record,
              IWF: Fn(&mut EncodeBuf, &Item) -> EncodeResult<IWFR>
    {
        let mut dedup = HashMap::new();

        for item in items {
            let item_start = self.bytes.len();

            write_item(self, &item)?;

            let item_span = item_start..self.bytes.len();

            // okay, so we're hashing the encoded item. however, we can't just store the slice
            // itself in the dedup hash map, because there's a chance that the buf will need to be
            // reallocated as it grows, and the address may not be stable.
            //
            // so, we store the Range representing the item start/end indices, but we still have to
            // hash the actual bytes.
            let item_encoded_hash = {
                let mut hasher = dedup.hasher().build_hasher();
                hasher.write(&self.bytes[item_span.clone()]);
                hasher.finish()
            };

            // now we have the hash, but we have to check for slice equality. luckily, hashbrown
            // has this `raw_entry()` API which lets us drive the search process ourselves.
            let entry = dedup.raw_entry_mut()
                .from_hash(item_encoded_hash,
                    |span: &Range<usize>| {
                        self.bytes[span.clone()]
                            .eq(&self.bytes[item_span.clone()])
                    });

            let item_offset = match entry {
                RawEntryMut::Occupied(e) => {
                    // if we have a matching slice, we'll back up the encode buffer to "erase" the
                    // copy that we just encoded, then re-use the starting offset from the
                    // duplicate item.
                    self.bytes.truncate(item_start);
                    e.key().start
                }

                RawEntryMut::Vacant(e) => {
                    // if this is a unique item, we insert the Range into the dedup map, re-using
                    // the hash we calculated above. the key is the unit struct () since we don't
                    // need to actually store any associated data. we could have used a HashSet,
                    // but it doesn't have the raw_entry() API.
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

    pub(crate) fn encode_pool_with_header<'b, Header, HF, Item, I, Record, RF, IWF, IWFR>
        (&mut self, header_func: HF, items: I, record_for_offset: RF, write_item: IWF)
            -> EncodeResult<usize>

        where Item: 'b,
              I: Iterator<Item = Item> + ExactSizeIterator,
              Record: EncodeBE,
              RF: Fn(u16, &Item) -> Record,
              IWF: Fn(&mut EncodeBuf, &Item) -> EncodeResult<IWFR>,
              Header: EncodeBE,
              HF: FnOnce(&mut EncodeBuf) -> EncodeResult<Header>
    {
        let table_start = self.bytes.len();
        let record_offset = table_start + Header::PACKED_LEN;
        self.bytes.resize(record_offset + (items.len() * Record::PACKED_LEN), 0u8);

        let header = header_func(self)?;
        self.encode_at(&header, table_start)?;

        self.encode_pool_internal(table_start, record_offset, items, record_for_offset, write_item)?;

        Ok(table_start)
    }

    pub(crate) fn encode_pool<'b, Item, I, Record, RF, IWF, IWFR>(&mut self,
        table_start: usize, items: I, record_for_offset: RF, write_item: IWF) -> EncodeResult<()>

        where Item: 'b,
              I: Iterator<Item = Item> + ExactSizeIterator,
              Record: EncodeBE,
              RF: Fn(u16, &Item) -> Record,
              IWF: Fn(&mut EncodeBuf, &Item) -> EncodeResult<IWFR>
    {
        let record_offset = self.bytes.len();
        self.bytes.resize(record_offset + (items.len() * Record::PACKED_LEN), 0u8);

        self.encode_pool_internal(table_start, record_offset, items, record_for_offset, write_item)
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
