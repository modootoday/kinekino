//! The window's own icon, taken from the mascot-derived app icon set
//! (`assets/design/app-icon/`, ADR 20261031111316). Nothing here is drawn or derived at run
//! time: the PNGs are embedded as they were derived and only repacked for the window system.

/// Derived app icon sizes handed to the window system, smallest first. The window manager
/// picks the size it shows (title bar, task switcher, taskbar).
const SIZES: [(u32, &[u8]); 5] = [
    (
        16,
        include_bytes!("../../../../assets/design/app-icon/kinekino-16.png"),
    ),
    (
        24,
        include_bytes!("../../../../assets/design/app-icon/kinekino-24.png"),
    ),
    (
        32,
        include_bytes!("../../../../assets/design/app-icon/kinekino-32.png"),
    ),
    (
        48,
        include_bytes!("../../../../assets/design/app-icon/kinekino-48.png"),
    ),
    (
        64,
        include_bytes!("../../../../assets/design/app-icon/kinekino-64.png"),
    ),
];

/// One decoded icon image: straight (non-premultiplied) RGBA rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Decodes every embedded size. An asset that fails to decode, is not square at its
/// declared size, or is not 8-bit RGBA is an error, never a silently missing icon.
pub fn app_icons() -> Result<Vec<IconImage>, String> {
    SIZES
        .iter()
        .map(|&(size, bytes)| decode(size, bytes))
        .collect()
}

fn decode(size: u32, bytes: &[u8]) -> Result<IconImage, String> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder
        .read_info()
        .map_err(|e| format!("icon {size}: {e}"))?;
    let mut buf = vec![0; reader.output_buffer_size().ok_or("icon: size overflow")?];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|e| format!("icon {size}: {e}"))?;
    if (info.width, info.height) != (size, size) {
        return Err(format!(
            "icon {size}: asset is {}x{}",
            info.width, info.height
        ));
    }
    let rgba = match info.color_type {
        png::ColorType::Rgba => buf[..info.buffer_size()].to_vec(),
        png::ColorType::Rgb => buf[..info.buffer_size()]
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        other => return Err(format!("icon {size}: unsupported colour type {other:?}")),
    };
    Ok(IconImage {
        width: size,
        height: size,
        rgba,
    })
}

/// Hands the app icon to the player's X window, once per window. A failure is reported
/// once, never fatal.
pub fn send(x11: &super::actions::x11::X11, window: u32) {
    if !x11.first_icon_request(window) {
        return;
    }
    if let Err(e) = app_icons().and_then(|i| x11.set_icon(window, &net_wm_icon(&i))) {
        eprintln!("{{\"warning\":\"window icon not set\",\"detail\":{e:?}}}");
    }
}

/// The EWMH `_NET_WM_ICON` value: for each image its width, its height, then one
/// `0xAARRGGBB` cardinal per pixel, row by row.
pub fn net_wm_icon(images: &[IconImage]) -> Vec<u32> {
    let mut out = Vec::new();
    for image in images {
        out.push(image.width);
        out.push(image.height);
        out.extend(image.rgba.as_chunks::<4>().0.iter().map(|p| {
            u32::from(p[3]) << 24 | u32::from(p[0]) << 16 | u32::from(p[1]) << 8 | u32::from(p[2])
        }));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_derived_size_decodes_square_and_complete() {
        let icons = app_icons().expect("embedded icons decode");
        let sizes: Vec<u32> = icons.iter().map(|i| i.width).collect();
        assert_eq!(sizes, [16, 24, 32, 48, 64]);
        for icon in &icons {
            assert_eq!(icon.height, icon.width);
            assert_eq!(icon.rgba.len(), (icon.width * icon.height * 4) as usize);
        }
    }

    #[test]
    fn icons_are_not_blank() {
        // A derived mascot icon has both transparent corners and opaque artwork.
        for icon in app_icons().unwrap() {
            let alphas: Vec<u8> = icon.rgba.as_chunks::<4>().0.iter().map(|p| p[3]).collect();
            assert!(alphas.contains(&255), "{} has no opaque pixel", icon.width);
            assert!(
                alphas.iter().any(|&a| a < 255),
                "{} has no transparency",
                icon.width
            );
        }
    }

    #[test]
    fn net_wm_icon_packs_size_then_argb_pixels() {
        let image = IconImage {
            width: 2,
            height: 1,
            rgba: vec![0x11, 0x22, 0x33, 0x44, 0xAA, 0xBB, 0xCC, 0xFF],
        };
        assert_eq!(
            net_wm_icon(&[image.clone(), image]),
            [
                2,
                1,
                0x4411_2233,
                0xFFAA_BBCC,
                2,
                1,
                0x4411_2233,
                0xFFAA_BBCC
            ]
        );
    }

    #[test]
    fn net_wm_icon_of_the_app_set_has_header_and_pixels_for_each_size() {
        let words = net_wm_icon(&app_icons().unwrap());
        let expected: usize = [16usize, 24, 32, 48, 64].iter().map(|s| 2 + s * s).sum();
        assert_eq!(words.len(), expected);
        assert_eq!(&words[..2], &[16, 16]);
    }
}
