// Maps the 120x120 logo SVG coordinate space to 3D units (centered, y-up).
export const UNIT = 1 / 40;
export const toXY = (x, y) => [(x - 60) * UNIT, (60 - y) * UNIT];
