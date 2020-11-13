use thiserror::Error;

use endian_codec::{PackedSize, DecodeBE};

use crate::*;
use crate::compile_model::util::decode::*;
use crate::compile_model::util::checksum;
use crate::compile_model::*;


#[derive(Debug, Error)]
pub enum TTFLoadError {
    #[error("bad whole-file checksum")]
    BadWholeFileChecksum
}

pub type TTFLoadResult<T> = Result<T, TTFLoadError>;


fn verify_whole_file_checksum(whole_file: &[u8], offset_table: &TTFOffsetTable,
    combined_records_checksum: u32, head_adjustment: u32) -> TTFLoadResult<()>
{
    let directory_end =
        TTFOffsetTable::PACKED_LEN
        + ((offset_table.num_tables as usize) * TTFTableRecord::PACKED_LEN);

    let adjustment =
        0xB1B0AFBAu32.overflowing_sub(
            combined_records_checksum.overflowing_add(
                checksum(&whole_file[..directory_end])).0).0;

    if adjustment == head_adjustment {
        Ok(())
    } else {
        Err(TTFLoadError::BadWholeFileChecksum)
    }
}

impl<'a> EncodedTables<'a> {
    fn load_ttf_data(&mut self, buf: &'a [u8]) -> TTFLoadResult<()> {
        let (offset_buf, rest) = buf.split_at(TTFOffsetTable::PACKED_LEN);
        let offset_table = TTFOffsetTable::decode_from_be_bytes(offset_buf);

        let mut head_record = None;
        let mut running_checksum = 0u32;

        for record in decode_from_pool::<TTFTableRecord>(offset_table.num_tables, rest) {
            running_checksum = running_checksum.overflowing_add(record.checksum).0;

            self.add_borrowed_table(record.tag, record.table_data(buf));

            if record.tag == tag!(h,e,a,d) {
                head_record = Some(record);
            }
        }

        if let Some(record) = head_record {
            let head = tables::Head::decode_from_be_bytes(record.table_data(buf));
            verify_whole_file_checksum(buf, &offset_table, running_checksum, head.checksum_adjustment)?;
            self.head = Some(head);
        }

        Ok(())
    }

    pub fn from_ttf_file(buf: &'a [u8]) -> TTFLoadResult<Self> {
        let mut et = Self::new(None);

        et.load_ttf_data(buf)?;
        Ok(et)
    }
}
