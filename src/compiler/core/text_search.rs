// Shared pure Unicode substring-position handling for runtime and optimizer.
// Match indices are UTF-8 byte offsets; convert them to Unicode scalar offsets
// without rescanning the whole prefix for every occurrence.
const MAX_TEXT_POSITION_MATCHES: usize = 10_000;

fn text_positions(text: &str, needle: &str) -> Option<Vec<i64>> {
    if needle.is_empty() {
        return None;
    }
    let needle_scalars = needle.chars().count();
    let (mut previous_end, mut scalar_offset) = (0, 0usize);
    let mut positions = Vec::new();
    for (byte_offset, _) in text.match_indices(needle) {
        if positions.len() >= MAX_TEXT_POSITION_MATCHES {
            return None;
        }
        scalar_offset += text[previous_end..byte_offset].chars().count();
        positions.push(i64::try_from(scalar_offset).ok()?);
        scalar_offset += needle_scalars;
        previous_end = byte_offset + needle.len();
    }
    Some(positions)
}

fn text_last_position(text: &str, needle: &str) -> Option<i64> {
    // An empty search term matches the end of text, consistent with rfind.
    match text.rfind(needle) {
        Some(byte_offset) => i64::try_from(text[..byte_offset].chars().count()).ok(),
        None => Some(-1),
    }
}
