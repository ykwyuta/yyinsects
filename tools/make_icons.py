"""アプリのアイコンを作る：葉っぱの上のテントウムシ（PIL が必要）
python3 tools/make_icons.py  → web/icon-192.png, web/icon-512.png, web/apple-touch-icon.png, assets/app_icon.ico
"""
import math
from PIL import Image, ImageDraw


def icon(n):
    S = 4 * n  # 大きく描いて縮める
    im = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    # 角の丸い背景（空と草の色）
    d.rounded_rectangle([0, 0, S - 1, S - 1], radius=S * 0.22, fill=(120, 190, 230, 255))
    d.rounded_rectangle([0, S * 0.62, S - 1, S - 1], radius=S * 0.22, fill=(80, 150, 60, 255))
    d.rectangle([0, S * 0.62, S - 1, S * 0.8], fill=(80, 150, 60, 255))
    # 葉っぱ
    leaf = [(S * 0.12, S * 0.78), (S * 0.40, S * 0.52), (S * 0.78, S * 0.46), (S * 0.92, S * 0.40), (S * 0.80, S * 0.62), (S * 0.46, S * 0.80)]
    d.polygon(leaf, fill=(60, 140, 50, 255))
    d.line([(S * 0.16, S * 0.76), (S * 0.88, S * 0.43)], fill=(150, 210, 110, 255), width=int(S * 0.012))
    # テントウムシ（上から見た形）
    cx, cy, r = S * 0.52, S * 0.50, S * 0.24
    for k in range(6):
        a = math.pi * (0.15 + k * 0.14) if k < 3 else math.pi * (0.71 + (k - 3) * 0.14)
        x0, y0 = cx + math.cos(a) * r * 0.6, cy + math.sin(a) * r * 0.6
        x1, y1 = cx + math.cos(a) * r * 1.25, cy + math.sin(a) * r * 1.25
        d.line([(x0, y0), (x1, y1)], fill=(20, 20, 20, 255), width=int(S * 0.018))
    d.ellipse([cx - r * 0.42, cy - r * 1.28, cx + r * 0.42, cy - r * 0.62], fill=(20, 20, 20, 255))
    d.ellipse([cx - r * 0.24, cy - r * 1.18, cx - r * 0.08, cy - r * 1.02], fill=(250, 250, 250, 255))
    d.ellipse([cx + r * 0.08, cy - r * 1.18, cx + r * 0.24, cy - r * 1.02], fill=(250, 250, 250, 255))
    d.ellipse([cx - r, cy - r * 0.95, cx + r, cy + r * 1.0], fill=(220, 40, 30, 255))
    d.line([(cx, cy - r * 0.95), (cx, cy + r * 1.0)], fill=(20, 20, 20, 255), width=int(S * 0.012))
    for (u, v, q) in [(0, -0.55, 0.22), (-0.5, -0.15, 0.2), (0.5, -0.15, 0.2), (-0.42, 0.45, 0.18), (0.42, 0.45, 0.18), (-0.15, 0.1, 0.0), (0.15, 0.1, 0.0)]:
        if q:
            d.ellipse([cx + u * r - q * r, cy + v * r - q * r, cx + u * r + q * r, cy + v * r + q * r], fill=(20, 20, 20, 255))
    d.ellipse([cx - r * 0.55, cy - r * 0.75, cx - r * 0.25, cy - r * 0.5], fill=(255, 140, 130, 200))
    return im.resize((n, n), Image.LANCZOS)


if __name__ == "__main__":
    icon(512).save("web/icon-512.png")
    icon(192).save("web/icon-192.png")
    icon(180).save("web/apple-touch-icon.png")
    big = icon(256)
    big.save("assets/app_icon.ico", sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
