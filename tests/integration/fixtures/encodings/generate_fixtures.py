#!/usr/bin/env python3
"""
Generate test fixture files in various Unicode encodings.
Each file contains malicious Unicode characters for testing detection.
"""

import os
from pathlib import Path

# Base directory for fixtures
BASE_DIR = Path(__file__).parent

# Test content with malicious Unicode characters
FIXTURES = {
    "zero_width_space.txt": "Hello\u200bWorld",  # U+200B Zero-width space
    "zero_width_joiner.txt": "Test\u200dFile",  # U+200D Zero-width joiner
    "bidi_override.txt": "Code\u202eAttack",  # U+202E Right-to-left override
    "homoglyph_cyrillic.txt": "let \u0430dmin = true",  # U+0430 Cyrillic 'a'
    "mixed_malicious.txt": "user\u200bname\u202e = \u0430dmin",  # Multiple issues
}


def create_utf16_le_fixtures():
    """Create UTF-16 LE encoded test files"""
    output_dir = BASE_DIR / "utf16le"
    output_dir.mkdir(exist_ok=True)

    for filename, content in FIXTURES.items():
        filepath = output_dir / filename
        with open(filepath, "w", encoding="utf-16-le") as f:
            f.write(content)
        print(f"Created UTF-16 LE: {filepath}")


def create_utf16_be_fixtures():
    """Create UTF-16 BE encoded test files"""
    output_dir = BASE_DIR / "utf16be"
    output_dir.mkdir(exist_ok=True)

    for filename, content in FIXTURES.items():
        filepath = output_dir / filename
        with open(filepath, "w", encoding="utf-16-be") as f:
            f.write(content)
        print(f"Created UTF-16 BE: {filepath}")


def create_utf32_le_fixtures():
    """Create UTF-32 LE encoded test files"""
    output_dir = BASE_DIR / "utf32le"
    output_dir.mkdir(exist_ok=True)

    for filename, content in FIXTURES.items():
        filepath = output_dir / filename
        with open(filepath, "w", encoding="utf-32-le") as f:
            f.write(content)
        print(f"Created UTF-32 LE: {filepath}")


def create_utf32_be_fixtures():
    """Create UTF-32 BE encoded test files"""
    output_dir = BASE_DIR / "utf32be"
    output_dir.mkdir(exist_ok=True)

    for filename, content in FIXTURES.items():
        filepath = output_dir / filename
        with open(filepath, "w", encoding="utf-32-be") as f:
            f.write(content)
        print(f"Created UTF-32 BE: {filepath}")


if __name__ == "__main__":
    print("Generating test fixtures in multiple encodings...")
    print()
    create_utf16_le_fixtures()
    print()
    create_utf16_be_fixtures()
    print()
    create_utf32_le_fixtures()
    print()
    create_utf32_be_fixtures()
    print()
    print("All test fixtures created successfully!")
    print()
    print("Fixture contents:")
    for filename, content in FIXTURES.items():
        print(f"  {filename}: {repr(content)}")
