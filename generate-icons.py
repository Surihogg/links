#!/usr/bin/env python3
import os
import shutil
import subprocess
from PIL import Image

SRC = "app-icon.png"
OUT_DIR = "src-tauri/icons"
MACOS_ICONSET = "AppIcon.iconset"
CONTENT_RATIO = 0.80

MACOS_SIZES = [
    ("icon_16x16", 16),
    ("icon_16x16@2x", 32),
    ("icon_32x32", 32),
    ("icon_32x32@2x", 64),
    ("icon_128x128", 128),
    ("icon_128x128@2x", 256),
    ("icon_256x256", 256),
    ("icon_256x256@2x", 512),
    ("icon_512x512", 512),
    ("icon_512x512@2x", 1024),
]

OTHER_SIZES = {
    "32x32.png": 32,
    "64x64.png": 64,
    "128x128.png": 128,
    "128x128@2x.png": 256,
    "Square30x30Logo.png": 30,
    "Square44x44Logo.png": 44,
    "Square71x71Logo.png": 71,
    "Square89x89Logo.png": 89,
    "Square107x107Logo.png": 107,
    "Square142x142Logo.png": 142,
    "Square150x150Logo.png": 150,
    "Square284x284Logo.png": 284,
    "Square310x310Logo.png": 310,
    "StoreLogo.png": 50,
}


def process_image(src_path, size, has_margin=True):
    img = Image.open(src_path).convert("RGBA")

    if has_margin:
        canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
        content_size = int(size * CONTENT_RATIO)
        resized = img.resize((content_size, content_size), Image.LANCZOS)
        offset = (size - content_size) // 2
        canvas.paste(resized, (offset, offset), resized)
        return canvas
    else:
        return img.resize((size, size), Image.LANCZOS)


def generate_macos_icns():
    if os.path.exists(MACOS_ICONSET):
        shutil.rmtree(MACOS_ICONSET)
    os.makedirs(MACOS_ICONSET)

    print("Generating macOS iconset...")
    for name, size in MACOS_SIZES:
        img = process_image(SRC, size, has_margin=True)
        out_path = os.path.join(MACOS_ICONSET, f"{name}.png")
        img.save(out_path)
        print(f"  {out_path}")

    icns_path = os.path.join(OUT_DIR, "icon.icns")
    try:
        subprocess.run(["iconutil", "-c", "icns", MACOS_ICONSET, "-o", icns_path], check=True)
        print(f"Generated: {icns_path}")
    except FileNotFoundError:
        print("Warning: iconutil not found. Install Xcode command line tools.")
    except subprocess.CalledProcessError as e:
        print(f"Error generating icns: {e}")

    shutil.rmtree(MACOS_ICONSET)


def generate_other_icons():
    print("\nGenerating other platform icons...")
    for filename, size in OTHER_SIZES.items():
        img = process_image(SRC, size, has_margin=False)
        out_path = os.path.join(OUT_DIR, filename)
        img.save(out_path)
        print(f"  {out_path}")

    img = process_image(SRC, 1024, has_margin=True)
    out_path = os.path.join(OUT_DIR, "icon.png")
    img.save(out_path)
    print(f"  {out_path}")

    ico_sizes = [16, 32, 48, 64, 128, 256]
    ico_images = [process_image(SRC, s, has_margin=False) for s in ico_sizes]
    ico_path = os.path.join(OUT_DIR, "icon.ico")
    ico_images[0].save(ico_path, format="ICO", sizes=[(s, s) for s in ico_sizes])
    print(f"  {ico_path}")


def main():
    if not os.path.exists(SRC):
        print(f"Error: Source icon not found: {SRC}")
        return 1

    if not os.path.exists(OUT_DIR):
        os.makedirs(OUT_DIR)

    generate_macos_icns()
    generate_other_icons()

    print("\nDone!")
    return 0


if __name__ == "__main__":
    exit(main())
