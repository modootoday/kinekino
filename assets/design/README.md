# Runtime SVG assets

Directly authored project SVGs (2026-10-18), not copied library artwork or generated raster assets. `icons/` uses 24×24 viewBox, white foreground for Rust tint, stroke 1.75 with round caps/joins. The icon canvas renders at 18 or 20 logical units inside independent hit targets. `window/` uses 16×16 viewBox and thinner 1-unit geometric strokes; it is not scaled from general control icons. Window caption targets are 42×32.

Rust semantic mapping: `src/presentation/icon_asset.rs`. State colors come from shared UI tokens. Native empty previews must not display reference-image example media as if it were a loaded source.
