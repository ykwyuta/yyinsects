"""日本を中心にしたメルカトル図法の地図の素材を作る（assets/map/*.png）。

通常のビルドでは不要（作った PNG をリポジトリに入れてある）。作り直すときだけ使う。

    npm pack world-atlas@2.0.2 jpn-atlas@1.0.2   # 作業用のフォルダで
    tar xzf world-atlas-2.0.2.tgz && mv package world-atlas
    tar xzf jpn-atlas-1.0.2.tgz && mv package jpn-atlas
    unzip jpn-atlas/build/gm-jpn-all_u_2_2.zip
    pip install pyshp pillow
    python tools/make_map.py <作業用のフォルダ>

出典
- 世界の陸地：Natural Earth（パブリックドメイン）。world-atlas の land-10m.json を使う
- 日本の都道府県・湖沼・河川・市街地：国土地理院「地球地図日本」（第2.2版）。
  jpn-atlas に同梱の gm-jpn-all_u_2_2 を使う

出力（src/earth.rs の投影と同じ式で、地図の座標に合わせて置く）
- world.png（L）：0=海 255=陸。地図全体
- japan.png（L）：0=海 1〜47=都道府県 60=日本以外の陸 70=湖
- kanto.png（RGB）：R=環境（ZONES） G=都道府県 B=0。関東地方とそのまわり
"""

import json
import math
import os
import sys

import shapefile
from PIL import Image, ImageChops, ImageDraw, ImageFilter

# ---- src/earth.rs と同じ値
LON_LEFT = -40.0  # 地図の左端の経度（中心は東経140度）
DEG = 10.0  # 経度1度あたりの地図の単位
LAT_MAX = 80.0
R = DEG * 180.0 / math.pi


def merc(lat):
    lat = max(-LAT_MAX, min(LAT_MAX, lat))
    return math.log(math.tan(math.pi / 4 + math.radians(lat) / 2))


MAP_W = 360.0 * DEG
MAP_H = 2 * merc(LAT_MAX) * R


def map_xy(lon, lat):
    return (lon - LON_LEFT) * DEG, (merc(LAT_MAX) - merc(lat)) * R


# 地図の枠（経度・緯度）と、画像の幅
BOXES = {
    "world": (LON_LEFT, -LAT_MAX, LON_LEFT + 360.0, LAT_MAX, 2048),
    "japan": (122.0, 24.0, 150.0, 46.0, 2048),
    "kanto": (138.35, 34.85, 141.0, 37.2, 2048),
}

# 環境の番号（src/earth.rs の Habitat と同じ順）
SEA, FIELD, PADDY, SATOYAMA, MOUNTAIN, URBAN, LAKE, RIVER, SHORE = range(9)

KANTO_PREFS = range(8, 15)


class Canvas:
    def __init__(self, name):
        lon0, lat0, lon1, lat1, w = BOXES[name]
        self.x0, self.y1 = map_xy(lon0, lat0)
        self.x1, self.y0 = map_xy(lon1, lat1)
        self.w = w
        self.h = round(w * (self.y1 - self.y0) / (self.x1 - self.x0))
        self.lon0, self.lon1 = lon0, lon1

    def px(self, lon, lat):
        x, y = map_xy(lon, lat)
        return ((x - self.x0) / (self.x1 - self.x0) * self.w, (y - self.y0) / (self.y1 - self.y0) * self.h)

    def lonlat(self, i, j):
        x = self.x0 + (i + 0.5) / self.w * (self.x1 - self.x0)
        y = self.y0 + (j + 0.5) / self.h * (self.y1 - self.y0)
        lon = x / DEG + LON_LEFT
        lat = math.degrees(2 * math.atan(math.exp(merc(LAT_MAX) - y / R)) - math.pi / 2)
        return lon, lat

    def new(self, mode="L"):
        return Image.new(mode, (self.w, self.h), 0)


# ---------------------------------------------------------------- 読み込み

def topo_rings(path):
    """TopoJSON（world-atlas の land）を経緯度の輪の一覧にする"""
    t = json.load(open(path))
    sx, sy = t["transform"]["scale"]
    tx, ty = t["transform"]["translate"]
    arcs = []
    for arc in t["arcs"]:
        x = y = 0
        pts = []
        for dx, dy in arc:
            x += dx
            y += dy
            pts.append((x * sx + tx, y * sy + ty))
        arcs.append(pts)

    def ring(idx):
        out = []
        for a in idx:
            pts = arcs[a] if a >= 0 else arcs[~a][::-1]
            out.extend(pts if not out else pts[1:])
        return out

    polys = []
    for g in t["objects"]["land"]["geometries"]:
        if g["type"] == "Polygon":
            polys.append([ring(r) for r in g["arcs"]])
        elif g["type"] == "MultiPolygon":
            for p in g["arcs"]:
                polys.append([ring(r) for r in p])
    return polys


def shp_polys(path):
    """シェープファイルの多角形を（記録, 輪の一覧）にする"""
    r = shapefile.Reader(path, encoding="utf-8")
    out = []
    for sr in r.iterShapeRecords():
        s = sr.shape
        parts = list(s.parts) + [len(s.points)]
        rings = [s.points[parts[i]:parts[i + 1]] for i in range(len(parts) - 1)]
        out.append((sr.record, rings))
    return out


def shp_lines(path):
    r = shapefile.Reader(path, encoding="utf-8")
    out = []
    for sr in r.iterShapeRecords():
        s = sr.shape
        parts = list(s.parts) + [len(s.points)]
        for i in range(len(parts) - 1):
            out.append((sr.record, s.points[parts[i]:parts[i + 1]]))
    return out


# ---------------------------------------------------------------- 塗る

def unwrap(ring):
    """経度が ±180 をまたいでもつながるように直す"""
    out = []
    prev = None
    for lon, lat in ring:
        if prev is not None:
            while lon - prev > 180:
                lon -= 360
            while lon - prev < -180:
                lon += 360
        out.append((lon, lat))
        prev = lon
    return out


def fill_poly(cv, img, rings, value):
    """輪の偶奇で穴をあけて塗る（地図の左右の端をまたぐものは 360 度ずらして2回塗る）"""
    for shift in (-360.0, 0.0, 360.0):
        mask = Image.new("1", (cv.w, cv.h), 0)
        any_in = False
        for ring in rings:
            ring = unwrap(ring)
            pts = [cv.px(lon + shift, lat) for lon, lat in ring]
            xs = [p[0] for p in pts]
            ys = [p[1] for p in pts]
            if max(xs) < 0 or min(xs) > cv.w or max(ys) < 0 or min(ys) > cv.h or len(pts) < 3:
                continue
            any_in = True
            m = Image.new("1", (cv.w, cv.h), 0)
            ImageDraw.Draw(m).polygon(pts, fill=1)
            mask = ImageChops.logical_xor(mask, m)
        if any_in:
            img.paste(value, mask=mask)


def draw_lines(cv, img, lines, value, width):
    d = ImageDraw.Draw(img)
    for pts in lines:
        p = [cv.px(lon, lat) for lon, lat in pts]
        if len(p) >= 2:
            d.line(p, fill=value, width=width, joint="curve")


def ellipse(cv, img, lon, lat, rlon, rlat, value):
    a = cv.px(lon - rlon, lat + rlat)
    b = cv.px(lon + rlon, lat - rlat)
    ImageDraw.Draw(img).ellipse([a, b], fill=value)


def noise(cv, scale, seed):
    """なめらかなまだら模様（0〜255）"""
    import random

    rnd = random.Random(seed)
    small = Image.new("L", (max(2, cv.w // scale), max(2, cv.h // scale)))
    small.putdata([rnd.randrange(256) for _ in range(small.width * small.height)])
    return small.resize((cv.w, cv.h), Image.BICUBIC).filter(ImageFilter.GaussianBlur(scale / 4))


# ---------------------------------------------------------------- 関東の山・丘陵（ゲーム用のおおよその形）

# 関東山地・足尾山地・八溝山地などの山の縁（南から北へ）。この線より外（西・北）を山とする
MOUNTAIN_FRONT = [
    (139.12, 34.85), (139.10, 35.20), (139.16, 35.28), (139.18, 35.36), (139.24, 35.41), (139.30, 35.47),
    (139.30, 35.53), (139.26, 35.58), (139.22, 35.61), (139.25, 35.64), (139.27, 35.70), (139.24, 35.79),
    (139.28, 35.86), (139.25, 35.95), (139.20, 36.05), (139.12, 36.12), (139.04, 36.18), (138.94, 36.22),
    (138.86, 36.30), (138.88, 36.40), (138.97, 36.44), (139.05, 36.46), (139.15, 36.49), (139.30, 36.45),
    (139.42, 36.40), (139.55, 36.45), (139.65, 36.55), (139.72, 36.68), (139.85, 36.78), (139.95, 36.90),
    (140.05, 37.00), (140.20, 36.95), (140.28, 36.82), (140.35, 36.70), (140.48, 36.62), (140.62, 36.66),
    (140.72, 36.80), (141.0, 36.95), (141.0, 37.25), (138.30, 37.25), (138.30, 34.85),
]

# 丘陵・里山（雑木林）。(経度, 緯度, 経度の半径, 緯度の半径)
HILLS = [
    (139.45, 35.585, 0.13, 0.045),  # 多摩丘陵
    (139.43, 35.765, 0.06, 0.016),  # 狭山丘陵
    (139.35, 36.05, 0.08, 0.05),  # 比企丘陵
    (139.33, 35.72, 0.05, 0.025),  # 加住丘陵・草花丘陵
    (139.66, 35.25, 0.06, 0.07),  # 三浦半島
    (140.48, 35.80, 0.10, 0.05),  # 北総台地の森
    (140.35, 36.40, 0.06, 0.05),  # 笠間の山
]
MINI_MOUNTAINS = [
    (140.10, 36.225, 0.06, 0.045),  # 筑波山
]
# 地球地図の市街地に入っていない、まわりの大きな町（経度, 緯度, 経度の半径, 緯度の半径）
TOWNS = [
    (139.60, 35.47, 0.07, 0.07),  # 横浜
    (139.66, 35.55, 0.05, 0.03),  # 川崎
    (139.53, 35.68, 0.12, 0.035),  # 武蔵野・三鷹・府中
    (139.41, 35.70, 0.05, 0.025),  # 立川
    (139.33, 35.66, 0.04, 0.025),  # 八王子
    (139.42, 35.56, 0.06, 0.03),  # 町田・相模原
    (139.64, 35.87, 0.06, 0.05),  # さいたま
    (139.49, 35.92, 0.03, 0.025),  # 川越
    (139.47, 35.80, 0.03, 0.02),  # 所沢
    (139.80, 35.88, 0.04, 0.03),  # 越谷・草加
    (139.97, 35.86, 0.04, 0.03),  # 柏・松戸
    (139.99, 35.71, 0.06, 0.03),  # 市川・船橋
    (140.12, 35.61, 0.06, 0.04),  # 千葉
    (139.88, 36.56, 0.05, 0.04),  # 宇都宮
    (139.06, 36.39, 0.04, 0.03),  # 前橋
    (139.00, 36.32, 0.04, 0.03),  # 高崎
    (140.47, 36.37, 0.04, 0.03),  # 水戸
    (139.39, 36.15, 0.03, 0.02),  # 熊谷
    (139.35, 35.33, 0.04, 0.02),  # 平塚
    (139.67, 35.28, 0.03, 0.025),  # 横須賀
    (139.16, 35.26, 0.025, 0.02),  # 小田原
]
BOSO_HILLS = [
    (139.80, 35.33), (140.00, 35.36), (140.25, 35.35), (140.42, 35.25), (140.30, 35.05),
    (140.00, 34.95), (139.84, 34.98), (139.78, 35.15),
]


def main(work):
    out_dir = os.path.join(os.path.dirname(__file__), "..", "assets", "map")
    os.makedirs(out_dir, exist_ok=True)
    land = topo_rings(os.path.join(work, "world-atlas", "land-10m.json"))
    gm = os.path.join(work, "gm-jpn-all_u_2_2")
    prefs = shp_polys(os.path.join(gm, "polbnda_jpn"))
    lakes = shp_polys(os.path.join(gm, "inwatera_jpn"))
    builtup = shp_polys(os.path.join(gm, "builtupa_jpn"))
    rivers = shp_lines(os.path.join(gm, "riverl_jpn"))

    # ---- 世界
    cv = Canvas("world")
    img = cv.new()
    for rings in land:
        fill_poly(cv, img, rings, 255)
    img.save(os.path.join(out_dir, "world.png"), optimize=True)
    print("world", img.size)

    names = []
    for rec, _ in prefs:
        if rec["nam"] not in names:
            names.append(rec["nam"])
    assert len(names) == 47 and names[12] == "Tokyo To", names

    def pref_code(rec):
        # 都道府県の並び（北海道=1 … 沖縄=47）は、記録に出てくる順と同じ
        return names.index(rec["nam"]) + 1

    # ---- 日本
    cv = Canvas("japan")
    img = cv.new()
    for rings in land:
        fill_poly(cv, img, rings, 60)
    for rec, rings in prefs:
        fill_poly(cv, img, rings, pref_code(rec))
    for rec, rings in lakes:
        fill_poly(cv, img, rings, 70)
    img.save(os.path.join(out_dir, "japan.png"), optimize=True)
    print("japan", img.size)

    # ---- 関東
    cv = Canvas("kanto")
    pref = cv.new()
    for rings in land:
        fill_poly(cv, pref, rings, 60)
    for rec, rings in prefs:
        fill_poly(cv, pref, rings, pref_code(rec))
    landmask = pref.point(lambda v: 255 if v else 0)

    zone = Image.new("L", (cv.w, cv.h), FIELD)
    # 里山のまだら（平野の中の小さな雑木林）
    n1 = noise(cv, 22, 1)
    zone.paste(SATOYAMA, mask=n1.point(lambda v: 255 if v > 168 else 0))
    # 田んぼ：川と湖の近く
    near_water = Image.new("L", (cv.w, cv.h), 0)
    kanto_rivers = [pts for rec, pts in rivers if any(cv.lon0 - 0.1 < lon < cv.lon1 + 0.1 for lon, _ in pts)]
    draw_lines(cv, near_water, kanto_rivers, 255, 16)
    for rec, rings in lakes:
        fill_poly(cv, near_water, rings, 255)
    near_water = near_water.filter(ImageFilter.MaxFilter(9))
    n2 = noise(cv, 30, 2)
    paddy = ImageChops.multiply(near_water, n2.point(lambda v: 255 if v > 95 else 0))
    zone.paste(PADDY, mask=paddy)
    # 里山（山のふもと・丘陵）と山
    mountain = Image.new("L", (cv.w, cv.h), 0)
    fill_poly(cv, mountain, [MOUNTAIN_FRONT], 255)
    for lon, lat, rl, rt in MINI_MOUNTAINS:
        ellipse(cv, mountain, lon, lat, rl, rt, 255)
    foot = mountain.filter(ImageFilter.MaxFilter(45))
    zone.paste(SATOYAMA, mask=ImageChops.multiply(foot, n2.point(lambda v: 255 if v > 60 else 0)))
    hills = Image.new("L", (cv.w, cv.h), 0)
    for lon, lat, rl, rt in HILLS:
        ellipse(cv, hills, lon, lat, rl, rt, 255)
    fill_poly(cv, hills, [BOSO_HILLS], 255)
    zone.paste(SATOYAMA, mask=hills)
    zone.paste(MOUNTAIN, mask=mountain)
    # 町（地球地図の市街地）
    for rec, rings in builtup:
        fill_poly(cv, zone, rings, URBAN)
    for lon, lat, rl, rt in TOWNS:
        ellipse(cv, zone, lon, lat, rl, rt, URBAN)
    # 海岸（海から 4 画素＝約600m 以内の陸。町は町のまま）
    sea = landmask.point(lambda v: 0 if v else 255)
    shore = sea.filter(ImageFilter.MaxFilter(9))
    not_urban = zone.point(lambda v: 0 if v == URBAN else 255)
    zone.paste(SHORE, mask=ImageChops.multiply(shore, not_urban))
    # 川（河原）と湖
    river_img = Image.new("L", (cv.w, cv.h), 0)
    draw_lines(cv, river_img, kanto_rivers, 255, 3)
    zone.paste(RIVER, mask=river_img)
    for rec, rings in lakes:
        fill_poly(cv, zone, rings, LAKE)
    # 海
    zone.paste(SEA, mask=sea)
    pref = Image.composite(pref, Image.new("L", pref.size, 0), landmask)
    rgb = Image.merge("RGB", (zone, pref, Image.new("L", pref.size, 0)))
    rgb.save(os.path.join(out_dir, "kanto.png"), optimize=True)
    print("kanto", rgb.size)
    counts = {}
    zdata = zone.get_flattened_data()
    pdata = pref.get_flattened_data()
    for z, p in zip(zdata, pdata):
        if p in KANTO_PREFS:
            counts[z] = counts.get(z, 0) + 1
    print("kanto zones", sorted(counts.items()))


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else ".")
