"""Generate the pixel art sheets in assets/sprites/.

Run: python3 tools/sprites.py. Needs Pillow. The PNGs are committed, so the
game build does not need Python. Characters are ASCII maps with palette keys,
round shapes (trees, bushes) are drawn from circles and outlined automatically.
"""

from pathlib import Path

from PIL import Image

OUT = Path(__file__).resolve().parent.parent / "assets" / "sprites"


def hexc(s, a=255):
    s = s.lstrip("#")
    return (int(s[0:2], 16), int(s[2:4], 16), int(s[4:6], 16), a)


BASE = {
    "o": "#2a1e2c",
    "s": "#f7c69a",
    "S": "#d98e62",
    "w": "#fbf3e4",
    "m": "#b0484a",
    "n": "#3a4a8a",
    "N": "#28336a",
    "k": "#4a3022",
    "f": "#f0cf7a",
    "F": "#c9a050",
}

PLAYER = {"h": "#4a2c1c", "H": "#6e4428", "y": "#e8b030", "Y": "#b8821c"}
VILLAGERS = [
    {"h": "#1e1a24", "H": "#3a3448", "y": "#d8604c", "Y": "#a8443a", "n": "#5a4a3a", "N": "#3e3228"},
    {"h": "#c8c0b8", "H": "#ece6de", "y": "#6aa86a", "Y": "#4a804e"},
    {"h": "#a85a2c", "H": "#cc7a44", "y": "#8a6ac8", "Y": "#644a9a", "n": "#4a4a4a", "N": "#333333"},
]
CLERKS = [
    ({"h": "#2a2228", "H": "#463a44", "y": "#c8b47a", "Y": "#9e8a56"}, False),
    ({"h": "#6a6460", "H": "#8e8882", "y": "#f4f0e6", "Y": "#c8c2b4"}, True),
    ({"h": "#5a301c", "H": "#7e4a2c", "y": "#4a9a9a", "Y": "#357272"}, False),
    ({"h": "#1a1a20", "H": "#34343e", "y": "#9a3a48", "Y": "#722a36"}, True),
]


def grid(rows, width):
    for r in rows:
        assert len(r) == width, (len(r), r)
    return [list(r) for r in rows]


def paint(img, rows, pal, ox=0, oy=0, flip=False):
    w = len(rows[0])
    for y, row in enumerate(rows):
        for x, ch in enumerate(row):
            if ch == ".":
                continue
            px = ox + (w - 1 - x if flip else x)
            img.putpixel((px, oy + y), hexc(pal[ch]))


# 16x24 character frames. Head rows 0-11, body 12-19, legs 20-23.
HEAD_DOWN = [
    ".....oooooo.....",
    "...oohhhhhhoo...",
    "..ohhhhhhhhhho..",
    ".ohhhHhhhhHhhho.",
    ".ohhhhhhhhhhhho.",
    ".ohhsssssssshho.",
    ".ohssssssssssho.",
    ".osssossssossso.",
    ".osssossssossso.",
    ".oSssssssssssSo.",
    "..oSsssmmsssSo..",
    "...ooSSSSSSoo...",
]
HEAD_UP = HEAD_DOWN[:5] + [
    ".ohhhhhhhhhhhho.",
    ".ohhhhhhhhhhhho.",
    ".ohhhhhhhhhhhho.",
    ".ohhhHhhhhHhhho.",
    ".oShhhhhhhhhhSo.",
    "..oShhhhhhhhSo..",
    "...ooSSSSSSoo...",
]
HEAD_SIDE = [
    "......ooooo.....",
    "....oohhhhhoo...",
    "...ohhhhhhhhho..",
    "..ohhhhhhHhhhho.",
    "..ohhhhhhhhhhho.",
    "..ohhhhhhssssso.",
    "..ohhhhhSssssso.",
    "..ohhhhhSssosso.",
    "..ohhhhhsssossso",
    "...ohhhssssssso.",
    "...ohhSsssssmo..",
    "....ooSSSSSoo...",
]
BODY_DOWN = [
    "....oyyyyyyo....",
    "...oyyywwyyyo...",
    "..oyyyyyyyyyyo..",
    "..oyYyyyyyyYyo..",
    "..osYyyyyyyYso..",
    "..oSoyyyyyyoSo..",
    "...onnnnnnnno...",
    "...onnnnonnnno..",
]
BODY_UP = [BODY_DOWN[0], "...oyyyyyyyyo...", *BODY_DOWN[2:]]
BODY_SIDE = [
    ".....oyyyyyo....",
    "....oyyyyyyyo...",
    "....oyyyyyyyo...",
    "....oyYYyyfffo..",
    "....oyYsyyFFFo..",
    "....oyyyyyoooo..",
    ".....onnnnno....",
    ".....onnnnno....",
]
LEGS_STAND = ["...onnno.onnno..", "...onnno.onnno..", "...okkko.okkko..", "....ooo...ooo..."]
LEGS_A = ["...onnno.onnno..", "...onnno.okkko..", "...okkko..ooo...", "....ooo........."]
LEGS_B = ["...onnno.onnno..", "...okkko.onnno..", "....ooo..okkko..", "..........ooo..."]
SIDE_STAND = [".....onnnno.....", ".....onnnno.....", ".....okkkkko....", ".....oooooo....."]
SIDE_STRIDE = ["....onno.onno...", "...onno...onno..", "..okkko...okkko.", "..oooo....oooo.."]

# Rows: down, up, right. Columns: stand, step A, step B.
FRAMES = [
    [HEAD_DOWN + BODY_DOWN + l for l in (LEGS_STAND, LEGS_A, LEGS_B)],
    [HEAD_UP + BODY_UP + l for l in (LEGS_STAND, LEGS_B, LEGS_A)],
    [HEAD_SIDE + BODY_SIDE + l for l in (SIDE_STAND, SIDE_STRIDE, SIDE_STAND)],
]


def character_sheet(colors):
    pal = {**BASE, **colors}
    img = Image.new("RGBA", (48, 72))
    for r, row in enumerate(FRAMES):
        for c, rows in enumerate(row):
            paint(img, grid(rows, 16), pal, c * 16, r * 24)
    return img


def clerk_rows(glasses, blink):
    rows = grid(HEAD_DOWN + BODY_DOWN[:6], 16)
    if glasses:
        for x in range(3, 13):
            rows[7][x] = "o"
        for x in (4, 11):
            rows[8][x] = "w"
    if blink:
        for x in (5, 10):
            rows[7][x] = "o" if glasses else "s"
    return rows


def clerk_sheet():
    img = Image.new("RGBA", (32, 18 * len(CLERKS)))
    for r, (colors, glasses) in enumerate(CLERKS):
        pal = {**BASE, **colors}
        for c, blink in enumerate((False, True)):
            paint(img, clerk_rows(glasses, blink), pal, c * 16, r * 18)
    return img


def hash2(x, y, seed=0):
    h = (x * 374761393 + y * 668265263 + seed * 2246822519) & 0xFFFFFFFF
    h = ((h ^ (h >> 13)) * 1274126177) & 0xFFFFFFFF
    return h ^ (h >> 16)


def outline(img, color="#2a1e2c"):
    w, h = img.size
    src = img.copy()
    for y in range(h):
        for x in range(w):
            if src.getpixel((x, y))[3]:
                continue
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                nx, ny = x + dx, y + dy
                if 0 <= nx < w and 0 <= ny < h and src.getpixel((nx, ny))[3]:
                    img.putpixel((x, y), hexc(color))
                    break


def blob(size, circles, light, mid, dark, speck):
    """Shaded round foliage: lit from the top left, outlined."""
    w, h = size
    img = Image.new("RGBA", size)
    for y in range(1, h - 1):
        for x in range(1, w - 1):
            best = None
            for cx, cy, r in circles:
                d = ((x - cx) ** 2 + (y - cy) ** 2) ** 0.5 / r
                if d <= 1 and (best is None or d < best[0]):
                    best = (d, (x - cx) / r, (y - cy) / r)
            if best is None:
                continue
            _, dx, dy = best
            lit = -dx * 0.6 - dy * 0.8
            c = light if lit > 0.45 else dark if lit < -0.3 else mid
            if c == mid and hash2(x, y, 7) % 9 == 0:
                c = speck
            img.putpixel((x, y), hexc(c))
    return img


def tree():
    img = Image.new("RGBA", (32, 40))
    for y in range(26, 38):
        for x in range(13, 19):
            img.putpixel((x, y), hexc("#5e3a20" if x >= 17 else "#8a5a32"))
    for x in (12, 19):
        img.putpixel((x, 37), hexc("#8a5a32"))
    crown = blob((32, 32), [(16, 12, 11), (9, 18, 8), (23, 18, 8), (16, 21, 9)],
                 "#7cc85a", "#4c9a40", "#2f6e34", "#63b04a")
    img.alpha_composite(crown)
    outline(img)
    return img


def bush():
    img = blob((16, 14), [(5, 8, 5), (10, 7, 5), (8, 5, 4)], "#7cc85a", "#4c9a40", "#2f6e34", "#63b04a")
    for x, y, c in ((5, 6, "#f4f0e6"), (11, 8, "#f08aa0"), (8, 4, "#f4f0e6")):
        img.putpixel((x, y), hexc(c))
    outline(img)
    return img


def tile(size, base, specks):
    """A flat tile with deterministic specks: specks is [(color, one_in_n, seed)]."""
    w, h = size
    img = Image.new("RGBA", size, hexc(base))
    for color, n, seed in specks:
        for y in range(h):
            for x in range(w):
                if hash2(x, y, seed) % n == 0:
                    img.putpixel((x, y), hexc(color))
    return img


def grass():
    img = Image.new("RGBA", (48, 16))
    plain = tile((16, 16), "#74bf4f", [("#68b246", 9, 1)])
    for i in range(3):
        img.paste(plain, (i * 16, 0))
    for bx, by in ((19, 4), (27, 10), (22, 12), (36, 3)):
        for dx, dy in ((0, 0), (2, 0), (1, 1)):
            img.putpixel((bx + dx, by + dy - 1), hexc("#4f9a38"))
        img.putpixel((bx + 1, by - 1), hexc("#94d86a"))
    for fx, fy, c in ((36, 4, "#fbf3e4"), (42, 10, "#f6d84a"), (38, 12, "#f08aa0")):
        for dx, dy in ((0, -1), (-1, 0), (1, 0), (0, 1)):
            img.putpixel((fx + dx, fy + dy), hexc(c))
        img.putpixel((fx, fy), hexc("#e8a030"))
    return img


def path():
    img = tile((16, 16), "#dcb67c", [("#c9a066", 6, 2)])
    for x, y in ((4, 5), (11, 11), (13, 3)):
        img.putpixel((x, y), hexc("#a8804c"))
        img.putpixel((x, y - 1), hexc("#f0d29a"))
    return img


ROOFS = [
    ("#e06a50", "#c04a3a", "#8a3028"),
    ("#5a84d4", "#3e62aa", "#2a4478"),
    ("#5eac66", "#43884e", "#2c6038"),
    ("#c08048", "#966030", "#683e20"),
]


def roofs():
    """8x8 shingles, one per colour. The game offsets every other row by 4."""
    img = Image.new("RGBA", (8 * len(ROOFS), 8))
    for i, (light, mid, dark) in enumerate(ROOFS):
        for y in range(8):
            for x in range(8):
                c = light if y == 0 else dark if y >= 6 or x == 7 else mid
                img.putpixel((i * 8 + x, y), hexc(c))
    return img


def wall():
    return tile((8, 8), "#f2e6c4", [("#e2d2a8", 7, 3)])


def door():
    rows = grid([
        "oooooooooooooooo",
        "oFFFFFFFFFFFFFFo",
        "oFkkkkkkkkkkkkFo",
        "oFkFFFFkkFFFFkFo",
        "oFkFffFkkFffFkFo",
        "oFkFffFkkFffFkFo",
        "oFkFffFkkFffFkFo",
        "oFkFFFFkkFFFFkFo",
        "oFkkkkkkkkkkkkFo",
        "oFkFFFFkkFFFFkFo",
        "oFkFffFkkFffFkFo",
        "oFkFffFkkFffFkFo",
        "oFkFffFkkFffFwFo",
        "oFkFffFkkFffFkFo",
        "oFkFffFkkFffFkFo",
        "oFkFffFkkFffFkFo",
        "oFkFFFFkkFFFFkFo",
        "oFkkkkkkkkkkkkFo",
        "oFkFFFFkkFFFFkFo",
        "oFkFffFkkFffFkFo",
        "oFkFffFkkFffFkFo",
        "oFkFFFFkkFFFFkFo",
        "oFkkkkkkkkkkkkFo",
        "oooooooooooooooo",
    ], 16)
    img = Image.new("RGBA", (16, 24))
    paint(img, rows, {**BASE, "f": "#b07440", "F": "#8a5530", "k": "#6a3e22"})
    return img


def window():
    rows = grid([
        "oooooooooooooooo",
        "owwwwwwwwwwwwwwo",
        "owggggggwggwgggo",
        "owgwgggwwgwgggwo",
        "owggggwwwggggwwo",
        "owgggwwgwgggwwgo",
        "owwwwwwwwwwwwwwo",
        "owgggggwwgggggwo",
        "owggggwwwggggwwo",
        "owgggwwgwgggwwgo",
        "oFFFFFFFFFFFFFFo",
        "oooooooooooooooo",
    ], 16)
    img = Image.new("RGBA", (16, 12))
    paint(img, rows, {**BASE, "g": "#8ccfee", "w": "#fbf3e4", "F": "#b07440"})
    return img


def shadow():
    img = Image.new("RGBA", (12, 4))
    for y in range(4):
        for x in range(12):
            if ((x - 5.5) / 6) ** 2 + ((y - 1.5) / 2) ** 2 <= 1:
                img.putpixel((x, y), (20, 30, 20, 70))
    return img


# Office interior, drawn at 3x.
def wallpaper():
    img = tile((16, 16), "#eadfbe", [])
    for y in range(16):
        for x in (0, 1, 8, 9):
            img.putpixel((x, y), hexc("#dccda4"))
    for y in (4, 12):
        img.putpixel((4, y), hexc("#c8b68a"))
        img.putpixel((12, (y + 8) % 16), hexc("#c8b68a"))
    return img


def floor():
    img = Image.new("RGBA", (16, 16))
    for y in range(16):
        for x in range(16):
            dark = (x // 8 + y // 8) % 2
            c = "#b8a074" if dark else "#d2bc8e"
            if x % 8 == 7 or y % 8 == 7:
                c = "#9e8660"
            img.putpixel((x, y), hexc(c))
    return img


def clock():
    img = Image.new("RGBA", (16, 16))
    for y in range(16):
        for x in range(16):
            d = ((x - 7.5) ** 2 + (y - 7.5) ** 2) ** 0.5
            if d <= 7.5:
                img.putpixel((x, y), hexc("#2a1e2c" if d > 6.3 else "#fbf3e4"))
    for x, y in ((7, 3), (7, 4), (7, 5), (7, 6), (7, 7), (8, 7), (9, 7), (10, 7)):
        img.putpixel((x, y), hexc("#2a1e2c"))
    for x, y in ((7, 1), (13, 7), (7, 13), (1, 7)):
        img.putpixel((x, y), hexc("#c04a3a"))
    return img


def board():
    img = Image.new("RGBA", (32, 24), hexc("#8a5530"))
    for y in range(2, 22):
        for x in range(2, 30):
            img.putpixel((x, y), hexc("#c9955a" if hash2(x, y, 5) % 5 else "#b07c48"))
    for px, py, w, h, c in ((4, 4, 8, 10, "#fbf3e4"), (14, 3, 7, 7, "#f6e08a"), (22, 5, 6, 9, "#fbf3e4"), (13, 12, 9, 8, "#cfe6f4")):
        for y in range(py, py + h):
            for x in range(px, px + w):
                line = (y - py) % 2 == 1 and px + 1 <= x < px + w - 1 and y < py + h - 1
                img.putpixel((x, y), hexc("#9a9088" if line else c))
        img.putpixel((px + w // 2, py), hexc("#c04a3a"))
    outline(img)
    return img


def plant():
    img = Image.new("RGBA", (16, 24))
    leaves = blob((16, 16), [(8, 8, 6), (4, 10, 4), (12, 10, 4), (8, 4, 4)], "#7cc85a", "#4c9a40", "#2f6e34", "#63b04a")
    img.alpha_composite(leaves, (0, 0))
    for y in range(15, 23):
        inset = (y - 15) // 4
        for x in range(4 + inset, 12 - inset):
            img.putpixel((x, y), hexc("#a85a3a" if y > 16 else "#c87050"))
    outline(img)
    return img


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    sheets = {
        "player": character_sheet(PLAYER),
        "clerks": clerk_sheet(),
        "tree": tree(),
        "bush": bush(),
        "grass": grass(),
        "path": path(),
        "roofs": roofs(),
        "wall": wall(),
        "door": door(),
        "window": window(),
        "shadow": shadow(),
        "wallpaper": wallpaper(),
        "floor": floor(),
        "clock": clock(),
        "board": board(),
        "plant": plant(),
    }
    villagers = Image.new("RGBA", (48, 72 * len(VILLAGERS)))
    for i, colors in enumerate(VILLAGERS):
        villagers.paste(character_sheet(colors), (0, i * 72))
    sheets["villagers"] = villagers
    for name, img in sheets.items():
        img.save(OUT / f"{name}.png")
    print(f"wrote {len(sheets)} sheets to {OUT}")


if __name__ == "__main__":
    main()
