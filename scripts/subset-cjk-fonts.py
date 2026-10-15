#!/usr/bin/env python3
import argparse
import re
import subprocess
import sys
from pathlib import Path

from fontTools.merge import Merger
from fontTools.pens.cu2quPen import Cu2QuPen
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.ttLib import TTCollection, TTFont, newTable
from fontTools.ttLib.scaleUpem import scale_upem

ROOT = Path(__file__).resolve().parents[1]
CJK_SOURCE = Path("/usr/share/fonts/noto-cjk/NotoSansCJK-Bold.ttc")
NOTO_DIR = Path("/usr/share/fonts/noto")
UI_SYMBOLS = {0x25C6, 0x25C7, 0x26A0, 0x2713}
SYMBOLS_SOURCE = "NotoSansSymbols2-Regular.ttf"
SUBSET_COMMAND = (sys.executable, "-m", "fontTools.subset")
UI_FONT = ROOT / "assets" / "RobotoCondensed-Bold.ttf"
LANGUAGE_LABELS = ROOT / "locales" / "en-US" / "settings" / "language.ftl"

CJK_SHARED_RANGES = [
    (0x3000, 0x303F),
    (0x3040, 0x309F),
    (0x30A0, 0x30FF),
    (0x31F0, 0x31FF),
    (0xFF01, 0xFF60),
    (0xFFE0, 0xFFE6),
]

CJK_FACES = [
    ("SC", 2, ["zh-CN"], ["settings-language-chinese"]),
    ("JP", 0, ["ja-JP"], ["settings-language-japanese"]),
]

SHAPED_FACES = [
    (
        "AR",
        "NotoSansArabicUI-Bold.ttf",
        ["ar-SA", "ur-PK", "fa-IR"],
        ["settings-language-arabic", "settings-language-urdu", "settings-language-persian"],
        [(0x0600, 0x06FF), (0x0750, 0x077F), (0x08A0, 0x08FF), (0x200C, 0x200F)],
    ),
    (
        "BN",
        "NotoSansBengaliUI-Bold.ttf",
        ["bn-BD"],
        ["settings-language-bengali"],
        [(0x0980, 0x09FF), (0x200C, 0x200D)],
    ),
    (
        "HI",
        "NotoSansDevanagariUI-Bold.ttf",
        ["hi-IN"],
        ["settings-language-hindi"],
        [(0x0900, 0x097F), (0x200C, 0x200D)],
    ),
]

MAXP_FIELDS = (
    "maxZones",
    "maxTwilightPoints",
    "maxStorage",
    "maxFunctionDefs",
    "maxInstructionDefs",
    "maxStackElements",
    "maxSizeOfInstructions",
    "maxComponentElements",
)


def catalog_codepoints(locales):
    found = set()
    for locale in locales:
        for path in sorted((ROOT / "locales" / locale).rglob("*.ftl")):
            for line in path.read_text(encoding="utf-8").splitlines():
                text = re.sub(r"^[A-Za-z0-9-]+\s*=", "", line)
                found |= {ord(ch) for ch in text if ord(ch) > 0x7F}
    return found


def baseline_codepoints(locale):
    path = ROOT / "data" / "fonts" / f"{locale}.codepoints"
    return {int(line, 16) for line in path.read_text().split()}


def label_codepoints(keys):
    labels = {}
    for line in LANGUAGE_LABELS.read_text(encoding="utf-8").splitlines():
        name, _, value = line.partition("=")
        labels[name.strip()] = value.strip()
    found = set()
    for key in keys:
        if key not in labels:
            sys.exit(f"missing {key} in {LANGUAGE_LABELS}")
        found |= {ord(ch) for ch in labels[key]}
    return found


def ranges(pairs):
    return {cp for start, end in pairs for cp in range(start, end + 1)}


def subset(full, codepoints, output, layout):
    subprocess.run(
        [
            *SUBSET_COMMAND,
            str(full),
            f"--unicodes={','.join(f'{cp:04X}' for cp in sorted(codepoints))}",
            f"--output-file={output}",
            f"--layout-features={layout}",
            "--no-hinting",
            "--desubroutinize",
        ],
        check=True,
    )
    return output


def quadratic(source, template, output):
    font = TTFont(source)
    order = font.getGlyphOrder()
    glyph_set = font.getGlyphSet()
    glyphs = {}
    for name in order:
        pen = TTGlyphPen(glyph_set)
        glyph_set[name].draw(Cu2QuPen(pen, 1.0, reverse_direction=True))
        glyphs[name] = pen.glyph()
    for tag in list(font.keys()):
        if tag not in template.keys():
            del font[tag]
    font["loca"] = newTable("loca")
    glyf = font["glyf"] = newTable("glyf")
    glyf.glyphOrder = order
    glyf.glyphs = glyphs
    glyf.compile(font)
    for name, glyph in glyphs.items():
        if hasattr(glyph, "xMin"):
            font["hmtx"][name] = (font["hmtx"][name][0], glyph.xMin)
    maxp = font["maxp"] = newTable("maxp")
    maxp.tableVersion = 0x00010000
    for field in MAXP_FIELDS:
        setattr(maxp, field, 1 if field == "maxZones" else 0)
    post = font["post"]
    post.formatType = 2.0
    post.extraNames = []
    post.mapping = {}
    post.glyphOrder = order
    font.sfntVersion = "\x00\x01\x00\x00"
    scale_upem(font, template["head"].unitsPerEm)
    font.save(output)
    return output


class CjkSource:
    def __init__(self, collection, index, work, template):
        self.full = work / f"cjk-{index}-full.otf"
        collection.fonts[index].save(self.full)
        self.work = work
        self.template = template

    def part(self, codepoints, name):
        cff = subset(self.full, codepoints, self.work / f"{name}.otf", "")
        return quadratic(cff, self.template, self.work / f"{name}.ttf")


class ShapedSource:
    def __init__(self, path, work, template):
        if not path.is_file():
            sys.exit(f"missing source font {path}; install noto-fonts")
        self.full = path
        self.work = work
        self.upem = template["head"].unitsPerEm
        self.tables = template.keys()

    def part(self, codepoints, name):
        output = subset(self.full, codepoints, self.work / f"{name}.ttf", "*")
        font = TTFont(output)
        for tag in list(font.keys()):
            if tag not in self.tables:
                del font[tag]
        scale_upem(font, self.upem)
        font.save(output)
        return output


def merge(parts, output):
    Merger().merge([str(UI_FONT), *map(str, parts)]).save(output)
    size = output.stat().st_size // 1024
    print(f"{output.name}: {size} KB, {len(TTFont(output).getGlyphOrder())} glyphs")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--cjk-source", type=Path, default=CJK_SOURCE)
    parser.add_argument("--noto-dir", type=Path, default=NOTO_DIR)
    args = parser.parse_args()
    if not args.cjk_source.is_file():
        sys.exit(f"missing source font {args.cjk_source}; install noto-cjk")

    template = TTFont(UI_FONT)
    base = set(template.getBestCmap())
    work = ROOT / "target" / "font-subset"
    work.mkdir(parents=True, exist_ok=True)
    collection = TTCollection(args.cjk_source)

    sources = {}
    wanted = {}
    labels = {}
    shared = ranges(CJK_SHARED_RANGES)
    for face, index, locales, label_keys in CJK_FACES:
        sources[face] = CjkSource(collection, index, work, template)
        wanted[face] = shared | baseline_codepoints(locales[0]) | catalog_codepoints(locales)
        labels[face] = label_codepoints(label_keys)
    for face, file, locales, label_keys, blocks in SHAPED_FACES:
        sources[face] = ShapedSource(args.noto_dir / file, work, template)
        wanted[face] = ranges(blocks) | catalog_codepoints(locales)
        labels[face] = label_codepoints(label_keys)

    symbols = ShapedSource(args.noto_dir / SYMBOLS_SOURCE, work, template)
    for face in ["UI", *sources]:
        covered = set(base)
        parts = []
        if face in sources:
            part = sources[face].part(wanted[face] - covered, f"{face}-script")
            parts.append(part)
            covered |= set(TTFont(part).getBestCmap())
        for script, codepoints in labels.items():
            missing = codepoints - covered
            if not missing:
                continue
            part = sources[script].part(missing, f"{face}-label-{script}")
            parts.append(part)
            covered |= set(TTFont(part).getBestCmap())
        missing = UI_SYMBOLS - covered
        if missing:
            parts.append(symbols.part(missing, f"{face}-symbols"))
        merge(parts, ROOT / "assets" / f"RobotoCondensed-Bold-{face}.ttf")


if __name__ == "__main__":
    main()
