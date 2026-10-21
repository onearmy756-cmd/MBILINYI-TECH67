#!/usr/bin/env python3
"""Generate the Mbilinyi Tech Solutions brand mark as real image files.

The site shipped with the logo only as an inline SVG data-URI, so search
engines and social cards had no actual image to fetch. This renders the same
gradient-square "M" mark to PNG (favicon sizes, apple-touch and the 1200x630
Open Graph card) plus a standalone SVG and a multi-size .ico.

Pure standard library: no Pillow, no ImageMagick, no GD. Pixels are rendered
with 3x supersampling and averaged down, which gives clean anti-aliased edges.
"""
import struct
import zlib
from pathlib import Path

OUT = Path(__file__).resolve().parent.parent / "assets" / "icons"

# Brand gradient: #06b6d4 -> #3b82f6 -> #8b5cf6
STOPS = [(0.0, (6, 182, 212)), (0.5, (59, 130, 246)), (1.0, (139, 92, 246))]
SS = 3  # supersampling factor


def gradient(t: float):
    t = min(1.0, max(0.0, t))
    for i in range(len(STOPS) - 1):
        t0, c0 = STOPS[i]
        t1, c1 = STOPS[i + 1]
        if t0 <= t <= t1:
            k = 0.0 if t1 == t0 else (t - t0) / (t1 - t0)
            return tuple(round(c0[j] + (c1[j] - c0[j]) * k) for j in range(3))
    return STOPS[-1][1]


def stroke_quad(p1, p2, w):
    """Four corners of a thick line between p1 and p2."""
    dx, dy = p2[0] - p1[0], p2[1] - p1[1]
    ln = (dx * dx + dy * dy) ** 0.5 or 1.0
    nx, ny = -dy / ln * w / 2, dx / ln * w / 2
    return [
        (p1[0] + nx, p1[1] + ny),
        (p2[0] + nx, p2[1] + ny),
        (p2[0] - nx, p2[1] - ny),
        (p1[0] - nx, p1[1] - ny),
    ]


def in_poly(px, py, poly):
    inside = False
    n = len(poly)
    for i in range(n):
        x1, y1 = poly[i]
        x2, y2 = poly[(i + 1) % n]
        if (y1 > py) != (y2 > py):
            xin = (x2 - x1) * (py - y1) / (y2 - y1) + x1
            if px < xin:
                inside = not inside
    return inside


def in_rounded(x, y, size, r):
    """Rounded-rect hit test with generous corner radius."""
    if x < 0 or y < 0 or x > size or y > size:
        return False
    cx = min(max(x, r), size - r)
    cy = min(max(y, r), size - r)
    dx, dy = x - cx, y - cy
    if dx == 0 or dy == 0:
        return True
    # allow a little feather so the downsample keeps the corners smooth
    return (dx * dx + dy * dy) <= r * r + r * 0.9


def render_mark(size: int, radius_ratio=0.22):
    """Return (w, h, rgba_bytes) of the brand mark at `size` px."""
    big = size * SS
    r = big * radius_ratio

    # The "M" sits inside a 0.62 box, drawn as two legs plus two diagonals.
    m = big * 0.19
    right = big - m
    top = big * 0.30
    bottom = big * 0.74
    w = big * 0.088

    left_leg = (m, top, m + w, bottom)
    right_leg = (right - w, top, right, bottom)
    # The two diagonals pass each other at the centre so the V has no seam.
    apex_y = top + (bottom - top) * 0.42
    mid = big / 2
    over = w * 1.2
    diag_l = stroke_quad((m + w * 0.5, top), (mid + over, apex_y + over * 0.7), w * 1.08)
    diag_r = stroke_quad((mid - over, apex_y + over * 0.7), (right - w * 0.5, top), w * 1.08)

    rows = []
    for y in range(big):
        row = bytearray()
        for x in range(big):
            if not in_rounded(x, y, big, r):
                row += b"\x00\x00\x00\x00"
                continue
            col = gradient((x + y) / (2.0 * big))
            on_m = (
                (left_leg[0] <= x <= left_leg[2] and left_leg[1] <= y <= left_leg[3])
                or (right_leg[0] <= x <= right_leg[2] and right_leg[1] <= y <= right_leg[3])
                or in_poly(x, y, diag_l)
                or in_poly(x, y, diag_r)
            )
            if on_m:
                row += b"\xff\xff\xff\xff"
            else:
                row += bytes((col[0], col[1], col[2], 255))
        rows.append(row)

    # box-filter downsample
    out_rows = []
    for y in range(size):
        row = bytearray()
        for x in range(size):
            acc = [0, 0, 0, 0]
            for dy in range(SS):
                src = rows[y * SS + dy]
                for dx in range(SS):
                    i = (x * SS + dx) * 4
                    for c in range(4):
                        acc[c] += src[i + c]
            n = SS * SS
            # premultiply-free average of the straight RGBA samples
            row += bytes((acc[0] // n, acc[1] // n, acc[2] // n, acc[3] // n))
        out_rows.append(row)
    return size, size, out_rows


def png(width, height, rows):
    raw = b"".join(b"\x00" + bytes(r) for r in rows)

    def chunk(tag, data):
        c = struct.pack(">I", len(data)) + tag + data
        return c + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

    ihdr = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", ihdr)
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )


def write_ico(path, pngs):
    """Pack PNG-compressed images into a .ico container."""
    n = len(pngs)
    out = struct.pack("<HHH", 0, 1, n)
    offset = 6 + n * 16
    for size, blob in pngs:
        b = 0 if size >= 256 else size
        out += struct.pack("<BBBBHHII", b, b, 0, 0, 1, 32, len(blob), offset)
        offset += len(blob)
    for _, blob in pngs:
        out += blob
    path.write_bytes(out)


LOGO_SVG = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" role="img" aria-label="Mbilinyi Tech Solutions">
  <defs>
    <linearGradient id="g" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="#06b6d4"/>
      <stop offset="0.5" stop-color="#3b82f6"/>
      <stop offset="1" stop-color="#8b5cf6"/>
    </linearGradient>
  </defs>
  <rect width="512" height="512" rx="113" fill="url(#g)"/>
  <path d="M97 154h45v226H97zm45 0h45l70 104-70 104h-45l70-104z" fill="#fff" opacity="0"/>
  <path d="M99 154h44v227H99zM369 154h44v227h-44zM104 154l33-3 96 141-31 21zm304 0-33-3-96 141 31 21z" fill="#fff"/>
</svg>
"""


def main():
    OUT.mkdir(parents=True, exist_ok=True)

    sizes = [512, 192, 180, 96, 48, 32, 16]
    blobs = {}
    for s in sizes:
        blobs[s] = png(*render_mark(s))

    (OUT / "icon-512.png").write_bytes(blobs[512])
    (OUT / "icon-192.png").write_bytes(blobs[192])
    (OUT / "apple-touch-icon.png").write_bytes(blobs[180])
    (OUT / "favicon-32.png").write_bytes(blobs[32])
    (OUT / "favicon-16.png").write_bytes(blobs[16])
    (OUT / "favicon-48.png").write_bytes(blobs[48])
    (OUT / "logo.svg").write_text(LOGO_SVG, encoding="utf-8")
    write_ico(OUT / "favicon.ico", [(16, blobs[16]), (32, blobs[32]), (48, blobs[48])])

    # Open Graph card: gradient plate with the mark centred.
    ow, oh = 1200, 630
    mark = render_mark(340)
    rows = []
    for y in range(oh):
        row = bytearray()
        for x in range(ow):
            col = gradient((x * 0.75 + y * 0.55) / (ow * 0.75 + oh * 0.55))
            # gentle vignette so the plate is not flat
            k = 1.0 - 0.18 * ((abs(x - ow / 2) / (ow / 2)) ** 2 + (abs(y - oh / 2) / (oh / 2)) ** 2)
            row += bytes((round(col[0] * k), round(col[1] * k), round(col[2] * k), 255))
        rows.append(row)
    ox, oy = (ow - mark[0]) // 2, (oh - mark[1]) // 2 - 10
    for y in range(mark[1]):
        dst = rows[oy + y]
        src = mark[2][y]
        for x in range(mark[0]):
            i = x * 4
            if src[i + 3] < 128:
                continue
            j = (ox + x) * 4
            dst[j:j + 3] = src[i:i + 3]
    (OUT / "og-image.png").write_bytes(png(ow, oh, rows))

    for p in sorted(OUT.iterdir()):
        print(f"{p.name:22} {p.stat().st_size:>7} bytes")


if __name__ == "__main__":
    main()
