//! Decode only documented x64 RelationProcessorCore records; no unaligned casts.
use crate::cpu_selection::Core;
use std::io;

pub fn decode(bytes: &[u8]) -> io::Result<Vec<Core>> {
    if bytes.is_empty() || bytes.len() > 1_048_576 {
        return Err(invalid());
    }
    let mut records = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        let header = bytes.get(offset..offset + 8).ok_or_else(invalid)?;
        let relationship = u32::from_le_bytes(header[..4].try_into().unwrap());
        let length = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
        if relationship != 0 || length < 48 || !length.is_multiple_of(8) {
            return Err(invalid());
        }
        let end = offset.checked_add(length).ok_or_else(invalid)?;
        let record = bytes.get(offset..end).ok_or_else(invalid)?;
        // PROCESSOR_RELATIONSHIP starts at 8; GroupCount is at 22, GroupMask at 24.
        if u16::from_le_bytes(record[30..32].try_into().unwrap()) != 1 {
            return Err(invalid());
        }
        let mask = u64::from_le_bytes(record[32..40].try_into().unwrap());
        let mask = usize::try_from(mask).map_err(|_| invalid())?;
        if mask == 0 {
            return Err(invalid());
        }
        records.push(Core {
            group: u16::from_le_bytes(record[40..42].try_into().unwrap()),
            mask,
            smt: record[8] & 1 != 0,
            efficiency_class: record[9],
        });
        offset = end;
    }
    Ok(records)
}
fn invalid() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "Incomplete processor-core topology record",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn core_record() -> Vec<u8> {
        let mut bytes = vec![0; 48];
        bytes[4..8].copy_from_slice(&48u32.to_le_bytes());
        bytes[8] = 1;
        bytes[30..32].copy_from_slice(&1u16.to_le_bytes());
        bytes[32..40].copy_from_slice(&3u64.to_le_bytes());
        bytes[40..42].copy_from_slice(&2u16.to_le_bytes());
        bytes
    }

    #[test]
    fn variable_records_require_complete_core_group_layout() {
        let mut bytes = core_record();
        assert_eq!(decode(&bytes).unwrap()[0].group, 2);
        for size in 0..48 {
            assert!(decode(&bytes[..size]).is_err());
        }
        let mut wrong = bytes.clone();
        wrong[30] = 2;
        assert!(decode(&wrong).is_err());
        wrong = bytes.clone();
        wrong[4..8].copy_from_slice(&0u32.to_le_bytes());
        assert!(decode(&wrong).is_err());
        bytes.extend_from_slice(&[0]);
        assert!(decode(&bytes).is_err());
    }

    #[test]
    fn record_size_controls_stride_without_guessing_a_fixed_array() {
        let mut first = core_record();
        first[4..8].copy_from_slice(&56u32.to_le_bytes());
        first.extend_from_slice(&[0; 8]);
        let mut second = core_record();
        second[32..40].copy_from_slice(&12u64.to_le_bytes());
        first.extend_from_slice(&second);
        let decoded = decode(&first).unwrap();
        assert_eq!(decoded.len(), 2);
        assert_eq!((decoded[0].mask, decoded[1].mask), (3, 12));
        assert!(decoded[0].smt);
        first[4..8].copy_from_slice(&49u32.to_le_bytes());
        assert!(decode(&first).is_err());
    }

    #[test]
    fn unknown_relationship_group_counts_and_empty_masks_are_rejected() {
        for relationship in [1u32, 2, 0xffff, u32::MAX] {
            let mut bytes = core_record();
            bytes[..4].copy_from_slice(&relationship.to_le_bytes());
            assert!(decode(&bytes).is_err());
        }
        for group_count in [0u16, 2, u16::MAX] {
            let mut bytes = core_record();
            bytes[30..32].copy_from_slice(&group_count.to_le_bytes());
            assert!(decode(&bytes).is_err());
        }
        let mut bytes = core_record();
        bytes[32..40].copy_from_slice(&0u64.to_le_bytes());
        assert!(decode(&bytes).is_err());
    }

    #[test]
    fn oversized_lengths_and_short_header_tails_cannot_escape_the_buffer() {
        for declared in [0u32, 8, 40, 56, 0xffff_fff8, u32::MAX] {
            let mut bytes = core_record();
            bytes[4..8].copy_from_slice(&declared.to_le_bytes());
            assert!(decode(&bytes).is_err());
        }
        for tail in 1..8 {
            let mut bytes = core_record();
            bytes.extend_from_slice(&vec![0; tail]);
            assert!(decode(&bytes).is_err());
        }
        let mut huge_followup = core_record();
        let mut next = core_record();
        next[4..8].copy_from_slice(&0xffff_fff8u32.to_le_bytes());
        huge_followup.extend_from_slice(&next);
        assert!(decode(&huge_followup).is_err());
        assert!(decode(&vec![0; 1_048_577]).is_err());
    }
}
