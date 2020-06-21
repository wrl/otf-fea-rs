pub fn checksum_head(table: &[u8]) -> u32 {
    return table.chunks(4)
        .enumerate()
        .fold(0u32, |acc, (i, bytes)| {
            let raw = match (i, bytes) {
                // for the `head` table, we have to treat `checksum_adjustment` as 0 while calculating
                // the checksum for the header.
                (2, _) => [0, 0, 0, 0],

                (_, &[a]) => [a, 0, 0, 0],
                (_, &[a, b]) => [a, b, 0, 0],
                (_, &[a, b, c]) => [a, b, c, 0],
                (_, &[a, b, c, d]) => [a, b, c, d],
                _ => unreachable!()
            };

            return acc.overflowing_add(u32::from_be_bytes(raw)).0;
        });
}

pub fn checksum(table: &[u8]) -> u32 {
    return table.chunks(4)
        .fold(0u32, |acc, bytes| {
            let raw = match bytes {
                &[a] => [a, 0, 0, 0],
                &[a, b] => [a, b, 0, 0],
                &[a, b, c] => [a, b, c, 0],
                &[a, b, c, d] => [a, b, c, d],
                _ => unreachable!()
            };

            return acc.overflowing_add(u32::from_be_bytes(raw)).0;
        });
}
