//! Separable source-pixel box feather; fixed window, zero outside the source canvas.
use super::*;
pub(super) fn apply(
    input: &[u8],
    width: u32,
    height: u32,
    radius: u32,
    control: &dyn MutationControl,
) -> Result<Vec<u8>, ApiError> {
    let width = width as usize;
    let height = height as usize;
    let radius = radius as usize;
    let divisor = (radius * 2 + 1) as u64;
    let mut horizontal = vec![0_u8; input.len()];
    let mut prefix = vec![0_u64; width.max(height) + 1];
    for y in 0..height {
        control.checkpoint()?;
        prefix[0] = 0;
        for x in 0..width {
            prefix[x + 1] = prefix[x] + u64::from(input[y * width + x]);
        }
        for x in 0..width {
            let sum = prefix[(x + radius + 1).min(width)] - prefix[x.saturating_sub(radius)];
            horizontal[y * width + x] = ((sum + divisor / 2) / divisor) as u8;
        }
    }
    let mut output = vec![0_u8; input.len()];
    for x in 0..width {
        control.checkpoint()?;
        prefix[0] = 0;
        for y in 0..height {
            prefix[y + 1] = prefix[y] + u64::from(horizontal[y * width + x]);
        }
        for y in 0..height {
            let sum = prefix[(y + radius + 1).min(height)] - prefix[y.saturating_sub(radius)];
            output[y * width + x] = ((sum + divisor / 2) / divisor) as u8;
        }
    }
    Ok(output)
}
