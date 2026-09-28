#!/usr/bin/env python3
"""Convert a Sponge/WorldEdit .schem file to the project's text scene format."""

import argparse
import gzip
import hashlib
import struct
from collections import Counter
from pathlib import Path


TAG_END = 0
TAG_BYTE = 1
TAG_SHORT = 2
TAG_INT = 3
TAG_LONG = 4
TAG_FLOAT = 5
TAG_DOUBLE = 6
TAG_BYTE_ARRAY = 7
TAG_STRING = 8
TAG_LIST = 9
TAG_COMPOUND = 10
TAG_INT_ARRAY = 11
TAG_LONG_ARRAY = 12


class NbtReader:
    def __init__(self, data):
        self.data = data
        self.offset = 0

    def read(self, size):
        end = self.offset + size
        if end > len(self.data):
            raise ValueError("unexpected end of NBT data")
        value = self.data[self.offset:end]
        self.offset = end
        return value

    def unpack(self, pattern):
        return struct.unpack(">" + pattern, self.read(struct.calcsize(">" + pattern)))[0]

    def string(self):
        length = self.unpack("H")
        start = self.offset
        encoded = self.read(length)
        try:
            return encoded.decode("utf-8")
        except UnicodeDecodeError as error:
            raise ValueError(
                f"invalid NBT string at byte {start} with length {length}: "
                f"{encoded[:64].hex(' ')}"
            ) from error

    def payload(self, tag_type):
        if tag_type == TAG_BYTE:
            return self.unpack("b")
        if tag_type == TAG_SHORT:
            return self.unpack("h")
        if tag_type == TAG_INT:
            return self.unpack("i")
        if tag_type == TAG_LONG:
            return self.unpack("q")
        if tag_type == TAG_FLOAT:
            return self.unpack("f")
        if tag_type == TAG_DOUBLE:
            return self.unpack("d")
        if tag_type == TAG_BYTE_ARRAY:
            length = self.unpack("i")
            return self.read(length)
        if tag_type == TAG_STRING:
            return self.string()
        if tag_type == TAG_LIST:
            item_type = self.unpack("B")
            length = self.unpack("i")
            return [self.payload(item_type) for _ in range(length)]
        if tag_type == TAG_COMPOUND:
            result = {}
            while True:
                child_type = self.unpack("B")
                if child_type == TAG_END:
                    return result
                child_name = self.string()
                result[child_name] = self.payload(child_type)
        if tag_type in (TAG_INT_ARRAY, TAG_LONG_ARRAY):
            length = self.unpack("i")
            pattern = "i" if tag_type == TAG_INT_ARRAY else "q"
            return [self.unpack(pattern) for _ in range(length)]
        raise ValueError(f"unsupported NBT tag type: {tag_type}")

    def root(self):
        tag_type = self.unpack("B")
        if tag_type != TAG_COMPOUND:
            raise ValueError(f"expected compound root tag, found type {tag_type}")
        root_name = self.string()
        return root_name, self.payload(tag_type)


def decode_varints(data, expected_count):
    values = []
    value = 0
    shift = 0

    for byte in data:
        value |= (byte & 0x7F) << shift
        if byte & 0x80:
            shift += 7
            if shift >= 35:
                raise ValueError("BlockData contains an invalid VarInt")
        else:
            values.append(value)
            value = 0
            shift = 0

    if shift != 0:
        raise ValueError("BlockData ends in the middle of a VarInt")
    if len(values) != expected_count:
        raise ValueError(
            f"BlockData contains {len(values)} blocks, expected {expected_count}"
        )
    return values


def schematic_fields(root):
    schematic = root.get("Schematic", root)
    blocks = schematic.get("Blocks", schematic)
    palette = blocks.get("Palette", schematic.get("Palette"))
    block_data = blocks.get("Data", schematic.get("BlockData"))

    required = ("Version", "Width", "Height", "Length")
    missing = [name for name in required if name not in schematic]
    if missing or palette is None or block_data is None:
        names = ", ".join(missing + (["Palette"] if palette is None else []) + (["BlockData"] if block_data is None else []))
        raise ValueError(f"schematic is missing required fields: {names}")

    return schematic, palette, block_data


def base_block_name(block_state):
    return block_state.split("[", 1)[0]


def convert(source, destination):
    compressed = source.read_bytes()
    try:
        nbt_data = gzip.decompress(compressed)
    except gzip.BadGzipFile as error:
        raise ValueError("the schematic is not gzip-compressed NBT") from error

    root_name, root = NbtReader(nbt_data).root()
    schematic, palette, block_data = schematic_fields(root)
    width = schematic["Width"]
    height = schematic["Height"]
    length = schematic["Length"]
    total = width * height * length
    palette_by_id = {palette_id: state for state, palette_id in palette.items()}
    block_ids = decode_varints(block_data, total)

    unknown_ids = sorted(set(block_ids) - set(palette_by_id))
    if unknown_ids:
        raise ValueError(f"BlockData references missing palette IDs: {unknown_ids}")

    states = [palette_by_id[palette_id] for palette_id in block_ids]
    state_counts = Counter(states)
    block_counts = Counter(base_block_name(state) for state in states)
    source_hash = hashlib.sha256(compressed).hexdigest()
    offset_x = -(width - 1) / 2
    offset_y = -(height - 1) / 2
    offset_z = -(length - 1) / 2

    destination.parent.mkdir(parents=True, exist_ok=True)
    with destination.open("w", encoding="utf-8", newline="\n") as output:
        output.write("# black-clover-scene-v1\n")
        output.write(f"# source_sha256={source_hash}\n")
        output.write(f"# root_name={root_name}\n")
        output.write(f"# schematic_version={schematic['Version']}\n")
        output.write(f"# data_version={schematic.get('DataVersion', 'unknown')}\n")
        output.write(f"# dimensions={width},{height},{length}\n")
        output.write(f"# center_offset={offset_x:g},{offset_y:g},{offset_z:g}\n")
        output.write("# columns=x<TAB>y<TAB>z<TAB>block_state\n")

        for index, state in enumerate(states):
            if base_block_name(state) == "minecraft:air":
                continue
            x = index % width
            z = (index // width) % length
            y = index // (width * length)
            output.write(f"{x}\t{y}\t{z}\t{state}\n")

    print(f"Root name: {root_name!r}")
    print(f"Schematic version: {schematic['Version']}")
    print(f"Minecraft data version: {schematic.get('DataVersion', 'unknown')}")
    print(f"Dimensions: {width} x {height} x {length}")
    print(f"Total positions: {total}")
    print(f"Palette entries: {len(palette)}")
    print("Block types:")
    for block_name, count in sorted(block_counts.items()):
        print(f"  {block_name}: {count}")
    print("Block states:")
    for state, count in sorted(state_counts.items()):
        print(f"  {state}: {count}")
    print(f"Scene rows written: {total - block_counts['minecraft:air']}")
    print(f"Output: {destination}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path, help="input .schem file")
    parser.add_argument("destination", type=Path, help="output scene text file")
    args = parser.parse_args()

    try:
        convert(args.source, args.destination)
    except (OSError, ValueError) as error:
        parser.exit(1, f"conversion failed: {error}\n")


if __name__ == "__main__":
    main()
