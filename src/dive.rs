//! 観察の場面：小さくなったカプセルで、林・草はら・公園・水辺に降りる。横から見た断面に虫が暮らす。
//!
//! 座標は横 x、下向き y。陸では地面の奥の縁がおよそ y=-0.4、水のある場面では水面が y=0。
//! 地面は奥行きのある帯（`BAND`）として描き、地面を歩く虫は帯の中の奥行き z（0=奥、1=手前）に置く。
//! 1 単位はおよそ 20cm（カプセルは虫と同じくらいの大きさに縮んでいる）。

use crate::art;
use crate::earth::*;
use crate::gfx::{Gfx, Texture};
use crate::life::*;
use crate::rng::{fbm, value_noise, Rng};
use crate::units::{self, Pose, Units};

/// 地面の帯の奥行き（画面の縦）
pub const BAND: f32 = 0.7;
const FSTEP: f32 = 0.25;
pub const CAP_R: f32 = 0.2;
/// 1 単位の長さ（cm）
pub const CM_PER_UNIT: f32 = 20.0;
const GRAVITY: f32 = 7.0;

pub struct Creature {
    pub sp: usize,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32,
    pub facing: f32,
    pub pitch: f32,
    pub phase: f32,
    pub seed: u32,
    pub tx: f32,
    pub ty: f32,
    pub wait: f32,
    pub flee: f32,
    pub scale: f32,
    /// 木の幹に縦にとまっている
    pub vertical: bool,
    /// とまる場所（とまる虫）
    pub anchor: (f32, f32, bool),
    /// 跳んでいる・飛んでいる途中（地面の虫・とまる虫）
    pub airborne: bool,
    /// 活動しない時間帯で、休んでいる（Some なら休む場所）
    pub rest: Option<Rest>,
    /// 成虫の季節でなく、冬越しの姿で越冬場所にいる
    pub winter: Option<(Stage, Site)>,
}

/// 巣（休んでいるハチ・アリのいるところ）
pub struct Nest {
    pub x: f32,
    pub y: f32,
    /// 0=地面の穴 1=木のうろ・切り株 2=巣箱 3=つり下がった巣
    pub kind: u8,
}

impl Creature {
    /// 動いている度合い（0〜1）。はばたき・脚の動き
    pub fn activity(&self) -> f32 {
        if self.rest.is_some() || self.winter.is_some() {
            return 0.0;
        }
        let beh = SPECIES[self.sp].beh;
        if matches!(beh, Beh::Flutter | Beh::Fly) || self.airborne {
            return 1.0;
        }
        (self.vx.abs() / SPECIES[self.sp].speed.max(0.01)).clamp(0.0, 1.0)
    }

    /// 絵の中心
    pub fn center(&self) -> (f32, f32) {
        let sp = &SPECIES[self.sp];
        if self.rest.is_some() || self.winter.is_some() {
            return (self.x, self.y);
        }
        if sp.beh.grounded() && !self.airborne {
            (self.x, self.y - art::base_lift(self.sp) * sp.disp_len() * self.scale)
        } else {
            (self.x, self.y)
        }
    }

    /// 絵の大きさ（体長）
    pub fn len(&self) -> f32 {
        SPECIES[self.sp].disp_len() * self.scale * self.winter.map_or(1.0, |(st, _)| st.scale())
    }

    pub fn bounds(&self) -> [f32; 4] {
        let (cx, cy) = self.center();
        if let Some((stage, _)) = self.winter.filter(|w| w.0 != Stage::Adult) {
            let (w, h) = art::stage_extent(self.sp, stage);
            let l = self.len();
            let (w, h) = if self.vertical { (h * l, w * l) } else { (w * l, h * l) };
            return [cx - w * 0.5, cy - h * 0.5, cx + w * 0.5, cy + h * 0.5];
        }
        units::bounds(self.sp, cx, cy, self.len(), self.vertical)
    }
}

pub struct Rock {
    pub x: f32,
    pub z: f32,
    pub r: f32,
    pub tone: f32,
}

pub struct Mote {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub s: f32,
}

/// 木（幹の位置と太さ）
pub struct Tree {
    pub x: f32,
    pub w: f32,
    pub tone: f32,
}

/// 草・花・ヨシ・イネ
pub struct Plant {
    pub x: f32,
    pub z: f32,
    pub h: f32,
    pub kind: u8,
    pub col: [f32; 3],
}

pub struct Capsule {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub facing: f32,
    pub tilt: f32,
}

pub struct Scene {
    pub hab: Habitat,
    pub spot: usize,
    /// 月（1〜12）
    pub month: u8,
    /// 時間帯 0=早朝 1=昼 2=夕方 3=夜
    pub time: u8,
    pub w: f32,
    /// 地面（水のある場面は水底。帯の奥の縁）の y
    floor: Vec<f32>,
    pub creatures: Vec<Creature>,
    pub rocks: Vec<Rock>,
    pub motes: Vec<Mote>,
    pub trees: Vec<Tree>,
    pub plants: Vec<Plant>,
    pub nests: Vec<Nest>,
    /// 朽ち木（地面に横たわる木。x と長さの半分）
    pub logs: Vec<(f32, f32)>,
    pub cap: Capsule,
    pub t: f32,
    pub seed: u32,
    rng: Rng,
    /// その場にいる虫（種類）
    pub present: Vec<usize>,
}

#[derive(Clone, Copy)]
pub struct View {
    pub cx: f32,
    pub cy: f32,
    /// 画面の 1 単位あたりの画素（UI 座標）
    pub ppm: f32,
    pub w: f32,
    pub h: f32,
}

impl View {
    pub fn x0(&self) -> f32 {
        self.cx - self.w * 0.5 / self.ppm
    }
    pub fn x1(&self) -> f32 {
        self.cx + self.w * 0.5 / self.ppm
    }
    pub fn y0(&self) -> f32 {
        self.cy - self.h * 0.5 / self.ppm
    }
    pub fn y1(&self) -> f32 {
        self.cy + self.h * 0.5 / self.ppm
    }
    pub fn to_world(&self, sx: f32, sy: f32) -> (f32, f32) {
        ((sx - self.w * 0.5) / self.ppm + self.cx, (sy - self.h * 0.5) / self.ppm + self.cy)
    }
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

fn c4(c: [f32; 3], a: f32) -> [f32; 4] {
    [c[0], c[1], c[2], a]
}

/// 写真に写った虫（種類、点数）
pub struct ShotResult {
    pub subjects: Vec<(usize, f32)>,
    /// 写った姿（冬越しの姿なら Some。subjects と同じ順）
    pub stages: Vec<Option<Stage>>,
}

/// 図鑑に収録できる写りなら 1、そうでなければ 0
pub fn stars_of(score: f32) -> u8 {
    if score >= 30.0 {
        1
    } else {
        0
    }
}

impl Scene {
    pub fn new(spot: usize, hab: Habitat, month: u8, time: u8, seed: u32) -> Scene {
        let mut rng = Rng::new(seed as u64 ^ 0x9e37_79b9);
        let w = 30.0;
        let n = (w / FSTEP) as usize + 2;
        let (base, rough) = match hab {
            Habitat::Pond => (1.6, 0.35),
            Habitat::River => (1.1, 0.3),
            Habitat::Paddy => (0.45, 0.05),
            Habitat::Mountain => (-0.4, 0.9),
            Habitat::Shore => (-0.4, 0.25),
            Habitat::Park => (-0.4, 0.15),
            _ => (-0.4, 0.4),
        };
        let mut floor = Vec::with_capacity(n);
        for i in 0..n {
            let x = i as f32 * FSTEP;
            let y = base + (fbm(x * 0.09, 3.7, seed, 4) - 0.5) * rough * 2.0 + (value_noise(x * 1.2, 1.3, seed + 7) - 0.5) * 0.04;
            floor.push(y);
        }
        let water = hab.is_water();
        let mut s = Scene {
            hab,
            spot,
            month,
            time,
            w,
            floor,
            creatures: Vec::new(),
            rocks: Vec::new(),
            motes: Vec::new(),
            trees: Vec::new(),
            plants: Vec::new(),
            nests: Vec::new(),
            logs: Vec::new(),
            cap: Capsule { x: w * 0.5, y: if water { -0.9 } else { -1.3 }, vx: 0.0, vy: 0.0, facing: 1.0, tilt: 0.0 },
            t: 0.0,
            seed,
            rng: Rng::new(seed as u64),
            present: Vec::new(),
        };
        // 木
        if hab.has_trees() {
            let mut x = rng.range(0.5, 2.5);
            let gap = if hab == Habitat::Park { 5.5 } else { 3.2 };
            while x < w - 0.5 {
                s.trees.push(Tree { x, w: rng.range(0.35, 0.75) * if hab == Habitat::Park { 1.2 } else { 1.0 }, tone: rng.f32() });
                x += rng.range(gap * 0.7, gap * 1.3);
            }
        }
        // 草・花など（kind 0=草 1=花 2=ヨシ 3=イネ 4=ハス・スイレンの葉）
        let flower_cols = [[0.98, 0.95, 0.92], [0.98, 0.85, 0.20], [0.85, 0.40, 0.75], [0.95, 0.40, 0.35], [0.55, 0.60, 0.95]];
        let np = match hab {
            Habitat::Meadow => 140,
            Habitat::Park | Habitat::Shore => 70,
            Habitat::Paddy => 120,
            Habitat::Pond | Habitat::River => 40,
            _ => 60,
        };
        for _ in 0..np {
            let x = rng.range(0.0, w);
            let kind = match hab {
                Habitat::Paddy => 3,
                Habitat::Pond => {
                    if rng.chance(0.5) {
                        4
                    } else {
                        2
                    }
                }
                Habitat::River => 2,
                Habitat::Meadow | Habitat::Park => {
                    if rng.chance(0.3) {
                        1
                    } else {
                        0
                    }
                }
                Habitat::Shore => 0,
                _ => {
                    if rng.chance(0.12) {
                        1
                    } else {
                        0
                    }
                }
            };
            let h = match kind {
                2 => rng.range(1.2, 2.6),
                3 => rng.range(0.6, 1.1),
                4 => 0.0,
                _ => rng.range(0.25, if hab == Habitat::Meadow { 1.3 } else { 0.7 }),
            };
            let season = season_of(month);
            // 冬は花がなく、秋は花が少ない
            let kind = if kind == 1 && (season == 3 || season == 2 && rng.chance(0.5)) { 0 } else { kind };
            let grass = match season {
                0 => mix([0.42, 0.66, 0.26], [0.58, 0.76, 0.32], rng.f32()),
                2 => mix([0.56, 0.58, 0.26], [0.70, 0.62, 0.30], rng.f32()),
                3 => mix([0.62, 0.54, 0.36], [0.72, 0.64, 0.44], rng.f32()),
                _ => mix([0.30, 0.55, 0.22], [0.50, 0.66, 0.28], rng.f32()),
            };
            let h = if season == 3 && kind == 0 { h * 0.5 } else { h };
            let col = if kind == 1 { flower_cols[rng.below(flower_cols.len() as u32) as usize] } else { grass };
            s.plants.push(Plant { x, z: rng.range(0.0, 1.0), h, kind, col });
        }
        s.plants.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap());
        // 石
        let nr = match hab {
            Habitat::River | Habitat::Mountain => 18,
            Habitat::Shore => 10,
            Habitat::Paddy => 0,
            _ => 6,
        };
        for _ in 0..nr {
            s.rocks.push(Rock { x: rng.range(0.0, w), z: rng.range(0.0, 1.0), r: rng.range(0.05, 0.3), tone: rng.f32() });
        }
        s.rocks.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap());
        for _ in 0..90 {
            s.motes.push(Mote { x: rng.range(0.0, 1.0), y: rng.range(0.0, 1.0), z: rng.range(0.3, 1.0), s: rng.range(0.6, 1.6) });
        }
        // 虫
        for (i, sp) in SPECIES.iter().enumerate() {
            if !lives_here(sp, spot, hab, month) {
                continue;
            }
            s.present.push(i);
            if !sp.in_season(month) {
                // 成虫の季節でなければ、冬越しの姿で越冬場所に
                let (stage, site) = sp.winter().unwrap();
                let count = ((sp.rarity * 0.5).round() as usize).clamp(1, 3);
                s.spawn_wintering(i, stage, site, count);
            } else if sp.active_at(time) {
                let count = ((sp.rarity * 1.4 * rng.range(0.7, 1.3)).round() as usize).clamp(1, 10);
                for _ in 0..count {
                    s.spawn(i);
                }
            } else {
                // 活動しない時間帯は、休む場所に少しだけ（見つけにくい）
                let count = ((sp.rarity * 0.6).round() as usize).clamp(1, 4);
                s.spawn_resting(i, count);
            }
        }
        s
    }

    /// 空を飛ぶ範囲（上端, 下端）
    fn air(&self, x: f32) -> (f32, f32) {
        let top = self.top();
        let low = if self.has_water() { -0.15 } else { self.floor_at(x) - 0.25 };
        (top + 0.3, low.max(top + 0.6))
    }

    /// とまる場所を選ぶ（幹にとまる虫は木の幹、ほかは草の先）
    fn pick_anchor(&mut self, sp: usize, near: f32) -> (f32, f32, bool) {
        let art = SPECIES[sp].art;
        if art.trunk() && !self.trees.is_empty() {
            let k = self.rng.below(self.trees.len() as u32) as usize;
            let k = if self.rng.chance(0.5) { self.trees.iter().enumerate().min_by(|a, b| (a.1.x - near).abs().partial_cmp(&(b.1.x - near).abs()).unwrap()).map_or(k, |t| t.0) } else { k };
            let (tx, tw) = (self.trees[k].x, self.trees[k].w);
            let side = if self.rng.chance(0.5) { 1.0 } else { -1.0 };
            let f = self.floor_at(tx);
            let y = f - self.rng.range(0.5, 3.2);
            return (tx + side * tw * 0.5, y, true);
        }
        let tall: Vec<usize> = (0..self.plants.len()).filter(|&i| self.plants[i].h > 0.25 && self.plants[i].kind != 4).collect();
        if tall.is_empty() {
            let x = (near + self.rng.range(-4.0, 4.0)).clamp(1.0, self.w - 1.0);
            return (x, self.floor_at(x) - 0.4, false);
        }
        let k = tall[self.rng.below(tall.len() as u32) as usize];
        let p = &self.plants[k];
        let base = if self.has_water() { 0.0 } else { self.floor_at(p.x) + p.z * BAND };
        (p.x, base - p.h * self.rng.range(0.6, 1.0), false)
    }

    fn spawn(&mut self, sp: usize) {
        let spd = &SPECIES[sp];
        let x = self.rng.range(2.0, self.w - 2.0);
        let z = self.rng.range(0.05, 0.95);
        let f = self.floor_at(x);
        let mut anchor = (x, 0.0, false);
        let y = match spd.beh {
            Beh::Flutter | Beh::Fly => {
                let (a, b) = self.air(x);
                self.rng.range(a, b)
            }
            Beh::Swim => self.rng.range(0.2, (f - 0.1).max(0.25)),
            Beh::Skate => 0.0,
            Beh::Perch => {
                anchor = self.pick_anchor(sp, x);
                anchor.1
            }
            Beh::Crawl | Beh::Hop => {
                if self.has_water() {
                    // 水辺では、浮いた葉や岸の上（水面）を歩く
                    0.0
                } else {
                    f + z * BAND
                }
            }
        };
        let x = if spd.beh == Beh::Perch { anchor.0 } else { x };
        let seed = self.rng.next_u32();
        let facing = if spd.beh == Beh::Perch && anchor.2 { if x > self.nearest_tree(x) { 1.0 } else { -1.0 } } else if self.rng.chance(0.5) { 1.0 } else { -1.0 };
        self.creatures.push(Creature {
            sp,
            x,
            y,
            z,
            vx: 0.0,
            vy: 0.0,
            facing,
            pitch: 0.0,
            phase: self.rng.range(0.0, 100.0),
            seed,
            tx: x,
            ty: y,
            wait: self.rng.range(0.0, 3.0),
            flee: 0.0,
            scale: self.rng.range(0.9, 1.1),
            vertical: spd.beh == Beh::Perch && anchor.2,
            anchor,
            airborne: false,
            rest: None,
            winter: None,
        });
    }

    /// 冬越しの姿の虫を置く
    fn spawn_wintering(&mut self, sp: usize, stage: Stage, site: Site, count: usize) {
        if site == Site::Nest {
            let start = self.creatures.len();
            self.spawn_resting(sp, count);
            for c in &mut self.creatures[start..] {
                c.rest = None;
                c.winter = Some((stage, site));
            }
            return;
        }
        let w = self.w;
        let len = SPECIES[sp].disp_len() * stage.scale();
        if site == Site::DeadWood && self.logs.len() < 2 {
            let x = self.rng.range(3.0, w - 3.0);
            let half = self.rng.range(0.9, 1.4);
            self.logs.push((x, half));
        }
        for _ in 0..count {
            let x0 = self.rng.range(2.0, w - 2.0);
            let (x, y, vertical, z) = match site {
                Site::Soil => {
                    // 土の断面の中（セミの幼虫は木の根の近く）
                    let x = if stage == Stage::CicadaNymph && !self.trees.is_empty() { self.nearest_tree(x0) + self.rng.range(-0.8, 0.8) } else { x0 };
                    let x = x.clamp(1.0, w - 1.0);
                    (x, self.floor_at(x) + BAND + self.rng.range(0.3, 0.9), false, 1.0)
                }
                Site::Litter | Site::UnderStone => {
                    let x = if site == Site::UnderStone && !self.rocks.is_empty() {
                        let k = self.rng.below(self.rocks.len() as u32) as usize;
                        self.rocks[k].x + self.rng.range(-0.3, 0.3)
                    } else if !self.trees.is_empty() && self.rng.chance(0.6) {
                        self.nearest_tree(x0) + self.rng.range(-0.7, 0.7)
                    } else {
                        x0
                    };
                    let x = x.clamp(1.0, w - 1.0);
                    let z = self.rng.range(0.3, 0.9);
                    (x, self.floor_at(x) + z * BAND - len * 0.1, false, z)
                }
                Site::DeadWood => {
                    let k = self.rng.below(self.logs.len() as u32) as usize;
                    let (lx, half) = self.logs[k];
                    let x = lx + self.rng.range(-half * 0.6, half * 0.6);
                    (x, self.floor_at(lx) + BAND * 0.5 - 0.3, false, 0.5)
                }
                Site::Stem | Site::Bark | Site::InTrunk | Site::LeafUnder => {
                    if matches!(site, Site::Bark | Site::InTrunk) && !self.trees.is_empty() {
                        let k = self.rng.below(self.trees.len() as u32) as usize;
                        let (tx, tw) = (self.trees[k].x, self.trees[k].w);
                        let side = if self.rng.chance(0.5) { 1.0 } else { -1.0 };
                        (tx + side * tw * 0.5, self.floor_at(tx) - self.rng.range(0.4, 2.2), true, 0.5)
                    } else {
                        let a = self.pick_anchor(sp, x0);
                        let base = if self.has_water() { 0.0 } else { self.floor_at(a.0) + BAND * 0.5 };
                        (a.0, (a.1 + 0.15).min(base - len * 0.3), false, 0.5)
                    }
                }
                Site::Bank => {
                    // 岸の草の根元（水のある場面では水面の草のかげ）
                    let a = self.pick_anchor(sp, x0);
                    let y = if self.has_water() { -0.04 } else { self.floor_at(a.0) + BAND * 0.5 - len * 0.1 };
                    (a.0, y, false, 0.5)
                }
                Site::WaterBottom => {
                    let z = self.rng.range(0.2, 0.9);
                    (x0, self.floor_at(x0) + z * BAND * 0.4 - len * 0.12, false, z)
                }
                Site::Nest => unreachable!(),
            };
            let seed = self.rng.next_u32();
            let facing = if vertical { if x > self.nearest_tree(x) { 1.0 } else { -1.0 } } else if self.rng.chance(0.5) { 1.0 } else { -1.0 };
            self.creatures.push(Creature {
                sp,
                x,
                y,
                z,
                vx: 0.0,
                vy: 0.0,
                facing,
                pitch: if vertical { std::f32::consts::FRAC_PI_2 } else { 0.0 },
                phase: self.rng.range(0.0, 100.0),
                seed,
                tx: x,
                ty: y,
                wait: 0.0,
                flee: 0.0,
                scale: self.rng.range(0.9, 1.1),
                vertical,
                anchor: (x, y, vertical),
                airborne: false,
                rest: None,
                winter: Some((stage, site)),
            });
        }
    }

    /// 休んでいる虫を置く
    fn spawn_resting(&mut self, sp: usize, count: usize) {
        let rest = SPECIES[sp].rest();
        let w = self.w;
        // 巣の虫は巣のまわりに集める
        let nest = if rest == Rest::Nest {
            let slug = SPECIES[sp].slug;
            let kind = match SPECIES[sp].art {
                Art::Ant => 0,
                Art::Bee(1) if slug == "asian-giant-hornet" => 0,
                Art::Bee(1) => 3,
                Art::Bee(0) if slug == "western-honey-bee" => 2,
                _ => 1,
            };
            let on_tree = matches!(kind, 1 | 3) && !self.trees.is_empty() && !self.has_water();
            let (x, y) = if on_tree {
                let k = self.rng.below(self.trees.len() as u32) as usize;
                let t = &self.trees[k];
                let side = if self.rng.chance(0.5) { 1.0 } else { -1.0 };
                let f = self.floor_at(t.x);
                if kind == 3 { (t.x + side * (t.w * 0.5 + 0.35), f - self.rng.range(2.0, 2.8)) } else { (t.x + side * t.w * 0.2, f - self.rng.range(1.0, 1.8)) }
            } else {
                let x = self.rng.range(2.0, w - 2.0);
                let ground = if self.has_water() { -0.05 } else { self.floor_at(x) + BAND * 0.6 };
                (x, if kind == 0 { ground } else { ground - 0.25 })
            };
            self.nests.push(Nest { x, y, kind: if on_tree || kind == 0 || kind == 2 { kind } else { 1 } });
            Some((x, y))
        } else {
            None
        };
        for n in 0..count {
            let x0 = self.rng.range(2.0, w - 2.0);
            let len = SPECIES[sp].disp_len();
            let (x, y, vertical, z) = match rest {
                Rest::Nest => {
                    let (nx, ny) = nest.unwrap();
                    (nx + (n as f32 - 0.5) * len * 0.8, ny - len * 0.15, false, 0.6)
                }
                Rest::Trunk | Rest::Roost | Rest::UnderLeaf => {
                    let a = self.pick_anchor(sp, x0);
                    if rest == Rest::UnderLeaf {
                        // 葉の裏：とまり場所の少し下にぶら下がる
                        (a.0, a.1 + len * 0.3, false, 0.5)
                    } else {
                        (a.0, a.1, a.2 && rest == Rest::Trunk, 0.5)
                    }
                }
                Rest::Litter | Rest::Hideout => {
                    // 木の根元・石のわき
                    let x = if rest == Rest::Litter && !self.trees.is_empty() {
                        let t = self.nearest_tree(x0);
                        t + self.rng.range(-0.6, 0.6)
                    } else if !self.rocks.is_empty() {
                        let k = self.rng.below(self.rocks.len() as u32) as usize;
                        self.rocks[k].x + self.rng.range(-0.3, 0.3)
                    } else {
                        x0
                    };
                    let z = self.rng.range(0.3, 0.9);
                    let ground = if self.has_water() { self.floor_at(x) + z * BAND } else { self.floor_at(x) + z * BAND };
                    (x.clamp(1.0, w - 1.0), ground - len * 0.15, false, z)
                }
                Rest::WaterEdge => {
                    let a = self.pick_anchor(sp, x0);
                    (a.0, -0.02, false, 0.5)
                }
            };
            let seed = self.rng.next_u32();
            let facing = if vertical { if x > self.nearest_tree(x) { 1.0 } else { -1.0 } } else if self.rng.chance(0.5) { 1.0 } else { -1.0 };
            self.creatures.push(Creature {
                sp,
                x,
                y,
                z,
                vx: 0.0,
                vy: 0.0,
                facing,
                pitch: if vertical { std::f32::consts::FRAC_PI_2 } else { 0.0 },
                phase: self.rng.range(0.0, 100.0),
                seed,
                tx: x,
                ty: y,
                wait: 0.0,
                flee: 0.0,
                scale: self.rng.range(0.9, 1.1),
                vertical,
                anchor: (x, y, vertical),
                airborne: false,
                rest: Some(rest),
                winter: None,
            });
        }
    }

    fn nearest_tree(&self, x: f32) -> f32 {
        self.trees.iter().map(|t| t.x).min_by(|a, b| (a - x).abs().partial_cmp(&(b - x).abs()).unwrap()).unwrap_or(x)
    }

    /// 地面（水底）の奥の縁の y
    pub fn floor_at(&self, x: f32) -> f32 {
        let fx = (x / FSTEP).clamp(0.0, (self.floor.len() - 2) as f32);
        let i = fx as usize;
        let t = fx - i as f32;
        self.floor[i] * (1.0 - t) + self.floor[i + 1] * t
    }

    /// カプセルが上がれるいちばん上
    pub fn top(&self) -> f32 {
        -4.8
    }

    pub fn has_water(&self) -> bool {
        self.hab.is_water()
    }

    /// 明るさ（0〜1）。森の中は木かげで暗く、水の中は少し暗い
    pub fn ambient_at(&self, y: f32) -> f32 {
        let winter = season_of(self.month) == 3;
        let base = match self.hab {
            Habitat::Forest if !winter => 0.5,
            Habitat::Mountain if !winter => 0.6,
            Habitat::Forest | Habitat::Mountain => 0.85,
            _ => 1.0,
        };
        // 早朝・夕方はうす暗く、夜は暗い（ライトが要る）
        let base = base * [0.75, 1.0, 0.62, 0.2][self.time as usize % 4];
        if self.has_water() && y > 0.0 {
            base * (0.85 - y * 0.15).max(0.45)
        } else {
            base
        }
    }

    pub fn cap_speed(&self) -> f32 {
        (self.cap.vx * self.cap.vx + self.cap.vy * self.cap.vy).sqrt()
    }

    /// カプセルを動かす（ax, ay：-1〜1 の入力。target：指・マウスで示した場所）
    pub fn update(&mut self, dt: f32, ax: f32, ay: f32, target: Option<(f32, f32)>, boost: bool) {
        self.t += dt;
        let top = self.top();
        let c = &mut self.cap;
        let (mut ix, mut iy) = (ax, ay);
        if let Some((tx, ty)) = target {
            let (dx, dy) = (tx - c.x, ty - c.y);
            let d = (dx * dx + dy * dy).sqrt();
            if d > 0.15 {
                let k = (d / 1.2).min(1.0);
                ix += dx / d * k;
                iy += dy / d * k;
            }
        }
        let l = (ix * ix + iy * iy).sqrt();
        if l > 1.0 {
            ix /= l;
            iy /= l;
        }
        let vmax = if boost { 3.4 } else { 1.2 };
        let acc = 4.0;
        c.vx += (ix * vmax - c.vx) * (acc / vmax.max(1.0) * dt).min(1.0) * 1.6;
        c.vy += (iy * vmax - c.vy) * (acc / vmax.max(1.0) * dt).min(1.0) * 1.6;
        if ix.abs() < 0.05 && iy.abs() < 0.05 {
            let k = (1.0 - dt * 2.5).max(0.0);
            c.vx *= k;
            c.vy *= k;
        }
        c.x += c.vx * dt;
        c.y += c.vy * dt;
        if c.vx > 0.2 {
            c.facing = 1.0;
        } else if c.vx < -0.2 {
            c.facing = -1.0;
        }
        c.tilt += ((c.vx * 0.05).clamp(-0.25, 0.25) - c.tilt) * (dt * 4.0).min(1.0);
        let w = self.w;
        c.x = c.x.clamp(1.0, w - 1.0);
        let cx = c.x;
        if c.y < top {
            c.y = top;
            c.vy = c.vy.max(0.0);
        }
        let fl = self.floor_at(cx) + BAND * 0.45 - CAP_R;
        let fl = fl.min(self.floor_at(cx - 0.4) + BAND * 0.45 - CAP_R).min(self.floor_at(cx + 0.4) + BAND * 0.45 - CAP_R);
        // 陸では、土の断面の中までもぐって、冬越しの虫を見られる
        let fl = if self.has_water() { fl } else { self.floor_at(cx) + BAND + 1.2 };
        let c = &mut self.cap;
        if c.y > fl {
            c.y = fl;
            c.vy = c.vy.min(0.0);
        }
        self.update_life(dt);
        for m in &mut self.motes {
            m.y += dt * 0.004 * m.z;
            m.x += dt * 0.003 * (m.y * 40.0).sin();
            if m.y > 1.0 {
                m.y -= 1.0;
            }
        }
    }

    fn update_life(&mut self, dt: f32) {
        let (cx, cy) = (self.cap.x, self.cap.y);
        let cs = self.cap_speed();
        let w = self.w;
        let water = self.has_water();
        for i in 0..self.creatures.len() {
            if self.creatures[i].rest.is_some() || self.creatures[i].winter.is_some() {
                // 休んでいる虫は、その場でじっとしている
                self.creatures[i].phase += dt * 0.2;
                continue;
            }
            let sp = self.creatures[i].sp;
            let spd = &SPECIES[sp];
            let x = self.creatures[i].x;
            let fl = self.floor_at(x);
            let (air0, air1) = self.air(x);
            let (r1, r2, r3) = (self.rng.f32(), self.rng.f32(), self.rng.f32());
            // とまる虫が逃げるときは、新しいとまり場所を選んでおく
            let new_anchor = if spd.beh == Beh::Perch {
                let cr = &self.creatures[i];
                let (dx, dy) = (cr.x - cx, cr.y - cy);
                let dist = (dx * dx + dy * dy).sqrt();
                let scared = dist < spd.shy * 0.5 && (cs > 0.45 || dist < spd.shy * 0.2) && cr.flee <= 0.0;
                let restless = cr.wait <= 0.0 && r1 < 0.002;
                if scared || restless { Some(self.pick_anchor(sp, x + (dx.signum() * 5.0))) } else { None }
            } else {
                None
            };
            let cr = &mut self.creatures[i];
            let (dx, dy) = (cr.x - cx, cr.y - cy);
            let dist = (dx * dx + dy * dy).sqrt();
            cr.phase += dt;
            let shy = spd.shy * 0.5;
            let scared = shy > 0.0 && dist < shy && (cs > 0.45 || dist < shy * 0.4);
            match spd.beh {
                Beh::Flutter | Beh::Fly | Beh::Swim => {
                    let swim = spd.beh == Beh::Swim;
                    let (lo, hi) = if swim { (0.15, (fl - 0.08).max(0.2)) } else { (air0, air1) };
                    let mut speed = spd.speed;
                    if scared && cr.flee <= 0.0 {
                        cr.flee = 2.0;
                        let ang = dy.atan2(dx) + (r1 - 0.5) * 0.8;
                        cr.tx = (cr.x + ang.cos() * 4.0).clamp(1.0, w - 1.0);
                        cr.ty = (cr.y + ang.sin() * 1.5).clamp(lo, hi);
                        cr.wait = 0.0;
                    }
                    if cr.flee > 0.0 {
                        cr.flee -= dt;
                        speed *= 2.8;
                    }
                    let (ddx, ddy) = (cr.tx - cr.x, cr.ty - cr.y);
                    let d = (ddx * ddx + ddy * ddy).sqrt();
                    if cr.wait > 0.0 {
                        // トンボ・ハチは空中で止まる。チョウは止まらずにただよう
                        cr.wait -= dt;
                        let k = (1.0 - dt * 2.0).max(0.0);
                        cr.vx *= k;
                        cr.vy *= k;
                    } else if d < 0.25 {
                        cr.wait = match spd.beh {
                            Beh::Fly => r2 * 2.5 + 0.3,
                            Beh::Swim => r2 * 3.0 + 0.5,
                            _ => 0.0,
                        };
                        let span = if spd.beh == Beh::Flutter { 3.0 } else { 5.0 };
                        cr.tx = (cr.x + (r1 - 0.5) * span * 2.0).clamp(1.0, w - 1.0);
                        cr.ty = lo + (hi - lo) * r3;
                    } else {
                        let k = (speed / d.max(0.01)).min(4.0);
                        cr.vx += (ddx * k - cr.vx) * (dt * 2.5).min(1.0);
                        cr.vy += (ddy * k - cr.vy) * (dt * 2.5).min(1.0);
                    }
                    if spd.beh == Beh::Flutter {
                        cr.vy += (cr.phase * 5.0).sin() * 1.2 * dt;
                    }
                    cr.x = (cr.x + cr.vx * dt).clamp(1.0, w - 1.0);
                    cr.y = (cr.y + cr.vy * dt).clamp(lo, hi);
                    if cr.vx.abs() > 0.05 {
                        cr.facing = cr.vx.signum();
                    }
                    let target_pitch = if swim && cr.vx.abs() + cr.vy.abs() > 0.05 { (-cr.vy / (cr.vx.abs() + 0.2)).clamp(-0.5, 0.5) } else { 0.0 };
                    cr.pitch += (target_pitch - cr.pitch) * (dt * 3.0).min(1.0);
                }
                Beh::Perch => {
                    if let Some(a) = new_anchor {
                        cr.anchor = a;
                        cr.flee = 2.5;
                        cr.airborne = true;
                        cr.vertical = false;
                        cr.wait = 3.0 + r2 * 6.0;
                    }
                    if cr.airborne {
                        // 飛んで新しいとまり場所へ
                        let (ddx, ddy) = (cr.anchor.0 - cr.x, cr.anchor.1 - cr.y);
                        let d = (ddx * ddx + ddy * ddy).sqrt();
                        let speed = 2.4;
                        if d < 0.08 {
                            cr.airborne = false;
                            cr.x = cr.anchor.0;
                            cr.y = cr.anchor.1;
                            cr.vertical = cr.anchor.2;
                            cr.vx = 0.0;
                            cr.vy = 0.0;
                        } else {
                            // 少し上へふくらむ道すじ
                            let lift = if d > 1.0 { -0.8 } else { 0.0 };
                            cr.vx = ddx / d * speed;
                            cr.vy = ddy / d * speed + lift;
                            cr.x += cr.vx * dt;
                            cr.y = (cr.y + cr.vy * dt).max(air0);
                            cr.facing = if ddx >= 0.0 { 1.0 } else { -1.0 };
                        }
                        cr.pitch = 0.0;
                    } else {
                        cr.flee = (cr.flee - dt).max(0.0);
                        cr.wait -= dt;
                        // 幹にとまる虫は頭を上に、幹の方を向く
                        if cr.vertical {
                            cr.pitch = std::f32::consts::FRAC_PI_2;
                            // ゆっくり幹をのぼりおりする
                            cr.y = cr.anchor.1 + (cr.phase * 0.3).sin() * 0.05;
                        } else {
                            cr.pitch = 0.0;
                        }
                    }
                }
                Beh::Crawl | Beh::Hop => {
                    let ground = if water { 0.0 } else { fl + cr.z * BAND };
                    if cr.airborne {
                        cr.vy += GRAVITY * dt;
                        cr.x = (cr.x + cr.vx * dt).clamp(0.8, w - 0.8);
                        cr.y += cr.vy * dt;
                        let ground = if water { 0.0 } else { self_floor(&self.floor, cr.x) + cr.z * BAND };
                        if cr.y >= ground && cr.vy > 0.0 {
                            cr.y = ground;
                            cr.airborne = false;
                            cr.vx = 0.0;
                            cr.vy = 0.0;
                            cr.wait = r2 * 2.0 + 0.5;
                        }
                    } else {
                        let jump = spd.beh == Beh::Hop && ((scared && cr.flee <= 0.0) || (cr.wait <= 0.0 && r3 < 0.004));
                        if jump {
                            let dir = if scared { dx.signum() } else if r1 < 0.5 { -1.0 } else { 1.0 };
                            cr.vx = dir * spd.speed * 5.0;
                            cr.vy = -(1.6 + r2 * 1.4);
                            cr.facing = dir;
                            cr.airborne = true;
                            cr.flee = 1.5;
                        } else if scared && cr.flee <= 0.0 {
                            // 歩く虫は急いで逃げる
                            cr.flee = 2.0;
                            cr.tx = (cr.x + dx.signum() * 2.0).clamp(0.8, w - 0.8);
                            cr.wait = 0.0;
                        } else if cr.wait > 0.0 {
                            cr.wait -= dt;
                            cr.vx = 0.0;
                        } else {
                            let ddx = cr.tx - cr.x;
                            if ddx.abs() < 0.05 {
                                cr.wait = r2 * 4.0 + 1.0;
                                cr.tx = (cr.x + (r1 - 0.5) * 2.0).clamp(0.8, w - 0.8);
                                cr.z = (cr.z + (r3 - 0.5) * 0.3).clamp(0.05, 0.95);
                            } else {
                                let fast = if cr.flee > 0.0 { 3.0 } else { 1.0 };
                                cr.vx = ddx.signum() * spd.speed * 0.6 * fast;
                                cr.facing = ddx.signum();
                                cr.x += cr.vx * dt;
                            }
                        }
                        cr.flee = (cr.flee - dt).max(0.0);
                        cr.y = ground;
                    }
                }
                Beh::Skate => {
                    // 水面をすいっとすべっては止まる
                    if scared && cr.flee <= 0.0 {
                        cr.flee = 1.5;
                        cr.vx = dx.signum() * spd.speed * 3.0;
                    } else if cr.wait > 0.0 {
                        cr.wait -= dt;
                    } else if cr.vx.abs() < 0.02 {
                        cr.vx = (if r1 < 0.5 { -1.0 } else { 1.0 }) * spd.speed * (1.0 + r2);
                        cr.wait = 0.4 + r3 * 1.5;
                    }
                    cr.flee = (cr.flee - dt).max(0.0);
                    cr.x = (cr.x + cr.vx * dt).clamp(0.8, w - 0.8);
                    cr.vx *= (1.0 - dt * 1.8).max(0.0);
                    if cr.vx.abs() > 0.05 {
                        cr.facing = cr.vx.signum();
                    }
                    cr.y = -0.02;
                }
            }
        }
    }

    /// 虫のいる所の明るさ（ライトを含む）
    pub fn light_at(&self, x: f32, y: f32, light: bool) -> f32 {
        let a = self.ambient_at(y);
        if light {
            let (lx, ly) = (self.cap.x + self.cap.facing * 0.4, self.cap.y);
            let d = ((x - lx).powi(2) + (y - ly).powi(2)).sqrt();
            let l = (1.0 - d / 3.2).clamp(0.0, 1.0);
            (a + l * 0.95).min(1.0)
        } else {
            a
        }
    }

    /// 写真の採点
    pub fn shoot(&self, v: &View, light: bool) -> ShotResult {
        let (vx0, vy0, vx1, vy1) = (v.x0(), v.y0(), v.x1(), v.y1());
        let (vw, vh) = (vx1 - vx0, vy1 - vy0);
        let (vcx, vcy) = (v.cx, v.cy);
        let mut best: Vec<(usize, f32, Option<Stage>)> = Vec::new();
        let blur = if self.cap_speed() > 1.0 { 0.85 } else { 1.0 };
        for cr in &self.creatures {
            let [bx0, by0, bx1, by1] = cr.bounds();
            let (hw, hh) = ((bx1 - bx0) * 0.5, (by1 - by0) * 0.5);
            let (cx, cy) = ((bx0 + bx1) * 0.5, (by0 + by1) * 0.5);
            if cx < vx0 || cx > vx1 || cy < vy0 || cy > vy1 {
                continue;
            }
            let ix = ((cx + hw).min(vx1) - (cx - hw).max(vx0)).max(0.0);
            let iy = ((cy + hh).min(vy1) - (cy - hh).max(vy0)).max(0.0);
            let frac = (ix * iy) / (4.0 * hw * hh).max(1e-6);
            let size = ((2.0 * hw) / vw).max((2.0 * hh) / vh * 0.8);
            let size_pts = (size / 0.36).clamp(0.0, 1.0).powf(0.8) * 55.0;
            let dcx = ((cx - vcx) / (vw * 0.5)).abs();
            let dcy = ((cy - vcy) / (vh * 0.5)).abs();
            let center_pts = (1.0 - dcx.max(dcy)).max(0.0) * 25.0;
            // 葉・土・草にかくれている分、写りにくい
            let hidden = match (cr.rest, cr.winter) {
                (Some(Rest::UnderLeaf | Rest::Litter | Rest::Hideout | Rest::Nest), _) => 0.8,
                (_, Some(_)) => 0.8,
                _ => 1.0,
            };
            let inside = frac * 20.0 * hidden;
            let lit = self.light_at(cx, cy, light);
            let score = hidden * (size_pts + center_pts + inside) * (0.2 + 0.8 * lit.min(1.0).powf(0.7)) * blur;
            let stage = cr.winter.map(|w| w.0);
            match best.iter_mut().find(|b| b.0 == cr.sp) {
                Some(b) => {
                    if score > b.1 {
                        b.1 = score;
                        b.2 = stage;
                    }
                }
                None => best.push((cr.sp, score, stage)),
            }
        }
        best.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let stages = best.iter().map(|b| b.2).collect();
        ShotResult { subjects: best.into_iter().map(|b| (b.0, b.1)).collect(), stages }
    }

    /// カプセルの高さ・深さの説明
    pub fn depth_text(&self, y: f32) -> String {
        if self.has_water() && y > 0.0 {
            tf!("水深 {:.0}cm", "{:.0} cm deep", "水深 {:.0}厘米", y * CM_PER_UNIT)
        } else {
            let ground = if self.has_water() { 0.0 } else { self.floor_at(self.cap.x) + BAND * 0.45 };
            if !self.has_water() && y > ground {
                return tf!("地面の下 {:.0}cm", "{:.0} cm underground", "地下 {:.0}厘米", (y - ground) * CM_PER_UNIT);
            }
            tf!("地面から {:.0}cm", "{:.0} cm up", "离地 {:.0}厘米", ((ground - y) * CM_PER_UNIT).max(0.0))
        }
    }

    // ------------------------------------------------------------ 描画

    pub fn render(&self, g: &mut Gfx, v: &View, light: bool, hide_capsule: bool, backdrop: Option<&Texture>, units: &mut Units) {
        let (sw, sh) = (v.w, v.h);
        let sx = 2.0 * v.ppm / sw;
        let sy = -2.0 * v.ppm / sh;
        g.set_xform(sx, sy, -v.cx * sx, -v.cy * sy);
        let (x0, y0, x1, y1) = (v.x0() - 1.0, v.y0() - 1.0, v.x1() + 1.0, v.y1() + 1.0);
        self.render_back(g, x0, y0, x1, y1);
        if let Some(texture) = backdrop {
            let yh = if self.has_water() { 0.0 } else { self.floor_at((x0 + x1) * 0.5) };
            crate::artwork::draw(g, texture, [x0, y0, x1, yh], 0.35, crate::artwork::Fit::Cover);
        }
        self.render_trees(g, x0, x1);
        self.render_floor(g, x0, y0, x1, y1, units);
        self.render_plants(g, x0, x1, false);
        self.render_nests(g, x0, x1);
        self.render_logs(g, x0, x1);
        // 虫 → カプセル → 手前の草
        let mut order: Vec<(f32, usize)> = self.creatures.iter().enumerate().filter(|(_, c)| c.x > x0 - 2.0 && c.x < x1 + 2.0).map(|(i, c)| (self.depth_key(c), i)).collect();
        order.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let mut cap_drawn = false;
        for &(k, i) in &order {
            if !cap_drawn && k > 0.5 {
                if !hide_capsule {
                    self.render_capsule(g, light, units);
                }
                cap_drawn = true;
            }
            self.render_creature(g, &self.creatures[i], light, units);
        }
        if !cap_drawn && !hide_capsule {
            self.render_capsule(g, light, units);
        }
        self.render_plants(g, x0, x1, true);
        self.render_front(g, x0, y0, x1, y1, light);
    }

    fn depth_key(&self, c: &Creature) -> f32 {
        if SPECIES[c.sp].beh.grounded() && !c.airborne {
            c.z
        } else {
            0.5 + (c.seed % 100) as f32 * 0.0001
        }
    }

    /// 雪が積もっているか（冬の山・日光）
    pub fn snowy(&self) -> bool {
        matches!(self.month, 12 | 1 | 2 | 3) && (self.hab == Habitat::Mountain || self.spot == S_NIKKO && !self.has_water())
    }

    fn sky_cols(&self) -> ([f32; 3], [f32; 3]) {
        let (top, hor) = self.day_sky();
        match self.time {
            0 => (mix(top, [0.50, 0.58, 0.84], 0.6), mix(hor, [1.0, 0.80, 0.64], 0.7)),
            2 => (mix(top, [0.32, 0.28, 0.54], 0.7), mix(hor, [1.0, 0.56, 0.30], 0.75)),
            3 => ([0.02, 0.03, 0.10], [0.08, 0.10, 0.22]),
            _ => (top, hor),
        }
    }

    fn day_sky(&self) -> ([f32; 3], [f32; 3]) {
        if season_of(self.month) == 3 {
            return ([0.46, 0.60, 0.80], [0.84, 0.88, 0.92]);
        }
        match self.hab {
            Habitat::Forest | Habitat::Mountain => ([0.40, 0.58, 0.72], [0.74, 0.84, 0.80]),
            Habitat::Shore => ([0.30, 0.56, 0.88], [0.80, 0.90, 0.96]),
            _ => ([0.30, 0.54, 0.86], [0.78, 0.88, 0.92]),
        }
    }

    fn ground_cols(&self) -> ([f32; 3], [f32; 3]) {
        if self.snowy() {
            return ([0.94, 0.96, 0.98], [0.70, 0.74, 0.80]);
        }
        let season = season_of(self.month);
        if season == 3 && matches!(self.hab, Habitat::Meadow | Habitat::Park) {
            return ([0.58, 0.52, 0.34], [0.36, 0.28, 0.18]);
        }
        if season == 2 && self.hab == Habitat::Meadow {
            return ([0.56, 0.56, 0.28], [0.34, 0.28, 0.16]);
        }
        match self.hab {
            Habitat::Forest => ([0.40, 0.32, 0.20], [0.26, 0.20, 0.14]),
            Habitat::Mountain => ([0.36, 0.32, 0.24], [0.24, 0.20, 0.16]),
            Habitat::Meadow => ([0.42, 0.56, 0.24], [0.30, 0.26, 0.16]),
            Habitat::Park => ([0.46, 0.60, 0.28], [0.36, 0.28, 0.18]),
            Habitat::Shore => ([0.88, 0.80, 0.60], [0.66, 0.58, 0.42]),
            Habitat::Paddy => ([0.40, 0.34, 0.24], [0.26, 0.22, 0.16]),
            Habitat::Pond => ([0.34, 0.32, 0.22], [0.20, 0.18, 0.14]),
            Habitat::River => ([0.56, 0.54, 0.48], [0.36, 0.34, 0.30]),
        }
    }

    /// 木の葉の色（冬で葉が落ちていれば None）。春の公園はサクラ、秋は紅葉
    fn leaf_col(&self, tone: f32) -> Option<[f32; 3]> {
        let green = if self.hab == Habitat::Mountain { [0.18, 0.40, 0.22] } else { [0.24, 0.48, 0.22] };
        match self.month {
            12 | 1 | 2 => {
                // 神社の森や公園の常緑樹は冬も緑
                if self.hab == Habitat::Park || tone > 0.6 && self.hab == Habitat::Forest { Some(mix(green, [0.1, 0.2, 0.1], 0.3)) } else { None }
            }
            3 | 4 => {
                if self.hab == Habitat::Park && tone > 0.3 { Some([0.98, 0.80, 0.86]) } else { Some(mix(green, [0.6, 0.8, 0.4], 0.45)) }
            }
            10 | 11 => Some(mix([0.90, 0.48, 0.16], [0.95, 0.78, 0.22], tone)),
            9 => Some(mix(green, [0.80, 0.62, 0.20], 0.25)),
            _ => Some(green),
        }
    }

    fn render_back(&self, g: &mut Gfx, x0: f32, y0: f32, x1: f32, y1: f32) {
        let (top, hor) = self.sky_cols();
        let yh = if self.has_water() { 0.0 } else { self.floor_at((x0 + x1) * 0.5) - 0.3 };
        let sy0 = y0.min(yh - 6.0);
        g.rect_vgrad(x0, sy0, x1, yh + 0.5, c4(top, 1.0), c4(hor, 1.0));
        if !self.has_water() {
            g.rect(x0, yh, x1, y1, c4(hor, 1.0));
        }
        // 太陽（早朝は東の低い空、夕方は西の低い空）、夜は月と星
        match self.time {
            3 => {
                for i in 0..60 {
                    let h1 = crate::rng::hash2(i, 1, self.seed);
                    let h2 = crate::rng::hash2(i, 2, self.seed);
                    let tw = 0.5 + 0.5 * (self.t * 2.0 + i as f32).sin();
                    g.circle(x0 + h1 * (x1 - x0), y0.min(yh - 6.0) + h2 * (yh - y0.min(yh - 6.0) - 1.0), 0.02 + 0.015 * tw, 6, [1.0, 1.0, 0.92, 0.5 + 0.4 * tw]);
                }
                let mx = x0 + (x1 - x0) * 0.75;
                g.radial(mx, yh - 4.2, 1.2, 24, [0.85, 0.90, 1.0, 0.25], [0.85, 0.9, 1.0, 0.0]);
                g.circle(mx, yh - 4.2, 0.3, 20, [0.98, 0.97, 0.88, 1.0]);
                g.circle(mx + 0.12, yh - 4.28, 0.26, 20, [0.02, 0.03, 0.10, 1.0]);
            }
            t => {
                let (fx, h, col) = match t {
                    0 => (0.18, 1.2, [1.0, 0.86, 0.66]),
                    2 => (0.82, 0.9, [1.0, 0.62, 0.32]),
                    _ => (0.8, 4.0, [1.0, 0.98, 0.86]),
                };
                let sunx = x0 + (x1 - x0) * fx;
                g.radial(sunx, yh - h, if t == 1 { 1.6 } else { 2.6 }, 24, c4(col, 0.6), c4(col, 0.0));
                g.circle(sunx, yh - h, 0.35, 20, c4(mix(col, [1.0, 1.0, 1.0], 0.5), 0.95));
            }
        }
        let cloud = match self.time {
            0 => [1.0, 0.88, 0.84, 0.5],
            2 => [1.0, 0.70, 0.55, 0.5],
            3 => [0.30, 0.32, 0.42, 0.35],
            _ => [1.0, 1.0, 1.0, 0.55],
        };
        for i in 0..5 {
            let cx = ((i as f32 * 7.3 + self.t * 0.05 - (x0 + x1) * 0.4) % 40.0 + 40.0) % 40.0 + x0 - 5.0;
            let cy = yh - 3.6 - (i % 3) as f32 * 0.5;
            for k in 0..4 {
                g.circle(cx + k as f32 * 0.4, cy - (k % 2) as f32 * 0.2, 0.35 + (k % 2) as f32 * 0.15, 14, cloud);
            }
        }
        // 遠くの山・森・町（関東の景色）
        let far = match self.hab {
            Habitat::Mountain => [0.38, 0.50, 0.46],
            Habitat::Shore => [0.40, 0.62, 0.80],
            Habitat::Park => [0.62, 0.66, 0.70],
            _ => [0.44, 0.58, 0.48],
        };
        for layer in 0..3 {
            let k = layer as f32;
            let col = mix(hor, far, 0.3 + k * 0.22);
            let par = 0.15 + 0.2 * k;
            let amp = if self.hab == Habitat::Mountain { 1.6 } else if self.hab == Habitat::Shore { 0.0 } else { 0.6 };
            let mut x = x0 - 2.0;
            let base = yh + 1.0;
            while x < x1 + 2.0 {
                let wx = x - (x0 + x1) * 0.5 * par;
                let hgt = |xx: f32| {
                    if self.hab == Habitat::Park && layer == 2 {
                        // 町のビル
                        let i = (xx * 1.5).floor();
                        0.6 + crate::rng::hash2(i as i32, 3, self.seed) * 1.6
                    } else {
                        0.4 + fbm(xx * 0.12, k * 7.0, self.seed + 3, 4) * amp * 2.0 - k * 0.2
                    }
                };
                let (h0, h1) = (hgt(wx), hgt(wx + 0.5));
                g.tri((x, base), (x + 0.5, base), (x + 0.5, yh - h1), c4(col, 1.0));
                g.tri((x, base), (x + 0.5, yh - h1), (x, yh - h0), c4(col, 1.0));
                x += 0.5;
            }
        }
        if self.hab == Habitat::Shore {
            // 海（東京湾）
            g.rect(x0, yh - 0.6, x1, yh + 1.0, [0.26, 0.52, 0.72, 1.0]);
            let mut x = x0.floor();
            while x < x1 {
                let w = (x * 1.7 + self.t * 1.5).sin() * 0.03;
                g.line((x, yh - 0.3 + w), (x + 0.6, yh - 0.3 - w), 0.03, [1.0, 1.0, 1.0, 0.5]);
                x += 1.1;
            }
        }
        // 水（池・田んぼ・川）
        if self.has_water() && y1 > 0.0 {
            let (top_c, deep_c) = match self.hab {
                Habitat::Paddy => ([0.46, 0.52, 0.34], [0.36, 0.34, 0.22]),
                Habitat::River => ([0.40, 0.66, 0.72], [0.22, 0.40, 0.44]),
                _ => ([0.30, 0.52, 0.46], [0.14, 0.26, 0.22]),
            };
            g.rect_vgrad(x0, 0.0, x1, y1.max(0.1), c4(top_c, 0.95), c4(deep_c, 1.0));
            // 光のゆらぎ
            for i in 0..8 {
                let bx = ((i as f32 * 4.1 + self.seed as f32 * 0.37) % self.w) + (self.t * 0.3 + i as f32).sin() * 0.4;
                let a = 0.05 * (1.0 + (self.t * 0.6 + i as f32 * 2.0).sin()) + 0.02;
                g.tri3((bx, 0.0), (bx + 0.6, 0.0), (bx + 1.4, 2.4), [1.0, 1.0, 0.9, a], [1.0, 1.0, 0.9, a], [1.0, 1.0, 0.9, 0.0]);
            }
            // 川の流れ
            if self.hab == Habitat::River {
                for i in 0..30 {
                    let fx = ((i as f32 * 3.7 + self.t * 1.4) % (self.w + 4.0)) - 2.0;
                    let fy = 0.1 + (i % 7) as f32 * 0.13;
                    g.line((fx, fy), (fx + 0.5, fy), 0.02, [1.0, 1.0, 1.0, 0.25]);
                }
            }
        }
    }

    fn render_trees(&self, g: &mut Gfx, x0: f32, x1: f32) {
        for tr in &self.trees {
            if tr.x < x0 - 3.0 || tr.x > x1 + 3.0 {
                continue;
            }
            let f = self.floor_at(tr.x) + 0.1;
            let top = self.top() - 2.0;
            let bark = mix([0.34, 0.26, 0.20], [0.46, 0.38, 0.30], tr.tone);
            let hw = tr.w * 0.5;
            g.rect(tr.x - hw, top, tr.x + hw, f, c4(bark, 1.0));
            g.rect(tr.x - hw, top, tr.x - hw * 0.4, f, c4(mix(bark, [0.0, 0.0, 0.0], 0.25), 1.0));
            g.rect(tr.x + hw * 0.5, top, tr.x + hw, f, c4(mix(bark, [1.0, 1.0, 1.0], 0.1), 1.0));
            // 木の皮のすじ
            let mut y = top;
            let mut k = 0;
            while y < f {
                let off = crate::rng::hash2((tr.x * 10.0) as i32, k, self.seed) - 0.5;
                g.line((tr.x + off * tr.w * 0.6, y), (tr.x + off * tr.w * 0.5, y + 0.4), 0.02, c4(mix(bark, [0.0, 0.0, 0.0], 0.35), 0.8));
                y += 0.35;
                k += 1;
            }
            // 樹液（クヌギの樹液に虫が集まる）
            if self.hab != Habitat::Park || tr.tone > 0.5 {
                let sy = f - 1.6 - tr.tone;
                g.circle(tr.x + hw * 0.3, sy, 0.06, 10, [0.55, 0.32, 0.12, 0.9]);
                g.line((tr.x + hw * 0.3, sy), (tr.x + hw * 0.3, sy + 0.25), 0.04, [0.50, 0.30, 0.12, 0.8]);
            }
            // 枝と葉（上の方）
            let leaf = self.leaf_col(tr.tone);
            if leaf.is_none() {
                // 冬：葉の落ちた枝
                let top = top + 3.0;
                for i in 0..6 {
                    let a = (i as f32 - 2.5) * 0.45 + tr.tone;
                    let (bx, by) = (tr.x + a.sin() * 1.6, top - a.cos().abs() * 1.4);
                    g.line((tr.x, top + 0.8), (bx, by), 0.05, c4(mix(bark, [0.0, 0.0, 0.0], 0.15), 1.0));
                    g.line((bx, by), (bx + a.sin() * 0.5, by - 0.5), 0.025, c4(bark, 1.0));
                }
                if self.snowy() {
                    g.rect(tr.x - hw, top + 0.6, tr.x + hw, top + 0.75, [0.96, 0.97, 1.0, 1.0]);
                }
                continue;
            }
            let leaf = leaf.unwrap();
            for i in 0..7 {
                let a = i as f32 * 1.7 + tr.tone * 3.0;
                let lx = tr.x + a.sin() * 1.5;
                let ly = top + 2.4 + (i % 3) as f32 * 0.7;
                g.circle(lx, ly, 1.1 + (i % 2) as f32 * 0.4, 16, c4(mix(leaf, [0.5, 0.7, 0.3], (i % 3) as f32 * 0.2), 0.95));
            }
        }
        // 森の上は木の葉でおおわれる（冬は枝のすきまから空が見える）
        if let (Some(canopy), true) = (self.leaf_col(0.3), matches!(self.hab, Habitat::Forest | Habitat::Mountain)) {
            let canopy = mix(canopy, [0.0, 0.0, 0.0], 0.15);
            let top = self.top() - 1.0;
            let mut x = (x0 / 0.8).floor() * 0.8;
            while x < x1 + 1.0 {
                let h = crate::rng::hash2((x * 10.0) as i32, 1, self.seed);
                g.circle(x, top + h * 0.6, 0.9 + h * 0.5, 14, c4(canopy, 0.95));
                x += 0.8;
            }
        }
    }

    fn render_floor(&self, g: &mut Gfx, x0: f32, _y0: f32, x1: f32, y1: f32, units: &mut Units) {
        let (sc, deepc) = self.ground_cols();
        let mut x = (x0 / FSTEP).floor() * FSTEP;
        let bottom = y1 + 2.0;
        while x < x1 {
            let (a, b) = (x.max(0.0), (x + FSTEP).min(self.w + 3.0));
            let (fa, fb) = (self.floor_at(a), self.floor_at(b));
            let ca = c4(mix(sc, [0.0, 0.0, 0.0], 0.18), 1.0);
            let cb = c4(mix(sc, [1.0, 1.0, 1.0], 0.06), 1.0);
            g.tri3((a, fa), (b, fb), (b, fb + BAND), ca, ca, cb);
            g.tri3((a, fa), (b, fb + BAND), (a, fa + BAND), ca, cb, cb);
            let s0 = c4(mix(sc, deepc, 0.5), 1.0);
            let s1 = c4(deepc, 1.0);
            g.tri3((a, fa + BAND), (b, fb + BAND), (b, bottom), s0, s0, s1);
            g.tri3((a, fa + BAND), (b, bottom), (a, bottom), s0, s1, s1);
            x += FSTEP;
        }
        // 地面の模様（砂の絵を、その場所の土の色に染める）
        let tint = {
            let base = [0.80, 0.72, 0.56];
            [(sc[0] / base[0]).min(1.25) * 0.94, (sc[1] / base[1]).min(1.25) * 0.94, (sc[2] / base[2]).min(1.25) * 0.94, 0.85]
        };
        let mut vertices = Vec::new();
        let tile = 4.0;
        let mut start = (x0 / tile).floor() * tile;
        while start < x1 {
            let mut a = start.max(x0);
            let end = (start + tile).min(x1);
            while a < end {
                let b = (a + FSTEP).min(end);
                let (fa, fb) = (self.floor_at(a), self.floor_at(b));
                let (u0, u1) = ((a - start) / tile, (b - start) / tile);
                let vertex = |x, y, u, v| crate::gfx::Vertex { x, y, u, v, c: tint };
                vertices.extend_from_slice(&[vertex(a, fa, u0, 0.0), vertex(b, fb, u1, 0.0), vertex(b, fb + BAND, u1, 1.0), vertex(a, fa, u0, 0.0), vertex(b, fb + BAND, u1, 1.0), vertex(a, fa + BAND, u0, 1.0)]);
                a = b;
            }
            start += tile;
        }
        units.ground(g, &vertices);
        // 落ち葉（林）
        if matches!(self.hab, Habitat::Forest | Habitat::Mountain) {
            let mut x = (x0 / 0.2).floor() * 0.2;
            while x < x1 {
                let i = (x / 0.2).round() as i32;
                let h = crate::rng::hash2(i, 5, self.seed);
                if h > 0.4 {
                    let z = crate::rng::hash2(i, 6, self.seed);
                    let y = self.floor_at(x) + z * BAND;
                    let c = mix([0.62, 0.42, 0.20], [0.42, 0.30, 0.16], crate::rng::hash2(i, 7, self.seed));
                    g.tri((x - 0.07, y), (x + 0.07, y - 0.02), (x, y + 0.03), c4(c, 0.9));
                }
                x += 0.2;
            }
        }
        // 石
        for r in &self.rocks {
            if r.x < x0 - 1.0 || r.x > x1 + 1.0 {
                continue;
            }
            let y = self.floor_at(r.x) + r.z * BAND;
            if units.rock(g, r.x, y, r.r, r.tone) {
                continue;
            }
            let c = mix([0.50, 0.48, 0.44], [0.66, 0.64, 0.60], r.tone);
            g.circle(r.x, y - r.r * 0.4, r.r, 14, c4(c, 1.0));
        }
    }

    /// 草・花など。front=true は手前の半分（虫より前に描く）
    fn render_plants(&self, g: &mut Gfx, x0: f32, x1: f32, front: bool) {
        for p in &self.plants {
            if (p.z > 0.75) != front || p.x < x0 - 1.0 || p.x > x1 + 1.0 {
                continue;
            }
            let base = if self.has_water() && p.kind != 0 && p.kind != 1 { self.floor_at(p.x) + p.z * BAND } else { self.floor_at(p.x) + p.z * BAND };
            let sway = (self.t * 1.3 + p.x * 2.0).sin() * 0.04;
            let dark = mix(p.col, [0.0, 0.0, 0.0], 0.2);
            match p.kind {
                4 => {
                    // スイレン・ハスの葉（水面に浮かぶ）
                    let x = p.x + sway;
                    g.tri((x - 0.35, 0.0), (x + 0.35, 0.0), (x, -0.06), [0.30, 0.55, 0.26, 1.0]);
                    g.tri((x - 0.35, 0.0), (x + 0.35, 0.0), (x, 0.04), [0.22, 0.44, 0.20, 1.0]);
                    if p.z > 0.85 {
                        g.circle(x + 0.1, -0.1, 0.09, 10, [0.98, 0.70, 0.80, 1.0]);
                    }
                }
                3 => {
                    // イネ（水の中から立つ）。春は小さな苗、秋は黄金色の穂、冬は切り株
                    let (hk, col) = match self.month {
                        4 | 5 => (0.35, [0.50, 0.78, 0.36]),
                        9 | 10 => (1.0, [0.86, 0.72, 0.28]),
                        11 | 12 | 1 | 2 | 3 => (0.12, [0.66, 0.56, 0.36]),
                        _ => (1.0, mix(p.col, [0.6, 0.7, 0.3], 0.3)),
                    };
                    let hh = (p.h + 0.4) * hk;
                    for k in 0..4 {
                        let dx = (k as f32 - 1.5) * 0.04;
                        g.line((p.x + dx, base), (p.x + dx * 2.5 + sway * hk, base - hh), 0.025, c4(col, 1.0));
                    }
                    if matches!(self.month, 9 | 10) {
                        g.circle(p.x + 0.1 + sway, base - hh + 0.08, 0.06, 8, [0.92, 0.80, 0.36, 1.0]);
                    }
                }
                2 => {
                    // ヨシ
                    g.line((p.x, base), (p.x + sway * 2.0, base - p.h - 0.6), 0.03, c4(mix(p.col, [0.6, 0.6, 0.3], 0.4), 1.0));
                    g.line((p.x + sway * 2.0, base - p.h - 0.6), (p.x + sway * 2.0 + 0.04, base - p.h - 0.9), 0.06, [0.62, 0.48, 0.32, 1.0]);
                    g.line((p.x + sway, base - p.h * 0.5 - 0.3), (p.x + 0.35, base - p.h * 0.5 - 0.5), 0.025, c4(p.col, 1.0));
                }
                1 => {
                    g.line((p.x, base), (p.x + sway, base - p.h), 0.025, [0.30, 0.50, 0.22, 1.0]);
                    let (fx, fy) = (p.x + sway, base - p.h);
                    for k in 0..5 {
                        let a = k as f32 / 5.0 * std::f32::consts::TAU + self.t * 0.2;
                        g.circle(fx + a.cos() * 0.06, fy + a.sin() * 0.06, 0.05, 8, c4(p.col, 1.0));
                    }
                    g.circle(fx, fy, 0.04, 8, [0.98, 0.84, 0.30, 1.0]);
                }
                _ => {
                    for k in 0..3 {
                        let dx = (k as f32 - 1.0) * 0.06;
                        let h = p.h * (0.7 + k as f32 * 0.15);
                        g.tri((p.x + dx - 0.02, base), (p.x + dx + 0.02, base), (p.x + dx * 2.5 + sway, base - h), c4(if k == 1 { p.col } else { dark }, 1.0));
                    }
                }
            }
        }
    }

    /// 朽ち木（中が見える切り口つき）
    fn render_logs(&self, g: &mut Gfx, x0: f32, x1: f32) {
        for &(lx, half) in &self.logs {
            if lx + half < x0 - 1.0 || lx - half > x1 + 1.0 {
                continue;
            }
            let cy = self.floor_at(lx) + BAND * 0.5 - 0.3;
            let r = 0.34;
            g.rect(lx - half, cy - r, lx + half, cy + r, [0.40, 0.30, 0.20, 1.0]);
            g.circle(lx - half, cy, r, 16, [0.40, 0.30, 0.20, 1.0]);
            g.circle(lx + half, cy, r, 16, [0.62, 0.50, 0.34, 1.0]);
            g.circle(lx + half, cy, r * 0.7, 16, [0.70, 0.58, 0.40, 1.0]);
            // くさって柔らかくなった中（切り口から見える）
            g.rect(lx - half * 0.8, cy - r * 0.6, lx + half * 0.8, cy + r * 0.6, [0.58, 0.44, 0.28, 1.0]);
            for k in 0..5 {
                let x = lx - half * 0.7 + k as f32 * half * 0.35;
                g.line((x, cy - r * 0.5), (x + 0.15, cy + r * 0.5), 0.02, [0.46, 0.34, 0.22, 0.8]);
            }
            g.circle(lx - half * 0.3, cy - r * 0.95, 0.08, 10, [0.30, 0.55, 0.30, 1.0]);
        }
    }

    /// 巣（地面の穴・木のうろ・巣箱・つり下がった巣）
    fn render_nests(&self, g: &mut Gfx, x0: f32, x1: f32) {
        for n in &self.nests {
            if n.x < x0 - 2.0 || n.x > x1 + 2.0 {
                continue;
            }
            let r = 0.22;
            match n.kind {
                0 => {
                    let (gc, _) = self.ground_cols();
                    g.circle(n.x, n.y, r * 1.6, 18, c4(mix(gc, [0.4, 0.3, 0.2], 0.3), 1.0));
                    g.circle(n.x, n.y + 0.02, r * 0.6, 14, [0.06, 0.04, 0.03, 1.0]);
                }
                1 => {
                    if self.trees.iter().any(|t| (t.x - n.x).abs() < t.w) {
                        g.circle(n.x, n.y, r * 0.8, 16, [0.10, 0.07, 0.05, 1.0]);
                    } else {
                        // 切り株
                        g.rect(n.x - r * 1.6, n.y - r * 0.6, n.x + r * 1.6, n.y + r * 2.0, [0.46, 0.36, 0.26, 1.0]);
                        g.rect(n.x - r * 1.6, n.y - r * 0.8, n.x + r * 1.6, n.y - r * 0.5, [0.70, 0.58, 0.40, 1.0]);
                        g.circle(n.x, n.y + r * 0.3, r * 0.5, 14, [0.08, 0.06, 0.04, 1.0]);
                    }
                }
                2 => {
                    // 巣箱（養蜂の箱）
                    g.rect(n.x - r * 2.2, n.y - r * 1.6, n.x + r * 2.2, n.y + r * 1.4, [0.80, 0.70, 0.50, 1.0]);
                    g.rect(n.x - r * 2.5, n.y - r * 1.9, n.x + r * 2.5, n.y - r * 1.5, [0.56, 0.44, 0.30, 1.0]);
                    g.rect(n.x - r * 0.8, n.y + r * 0.9, n.x + r * 0.8, n.y + r * 1.2, [0.10, 0.08, 0.06, 1.0]);
                }
                _ => {
                    // つり下がった紙の巣（しま模様）
                    g.line((n.x, n.y - r * 3.2), (n.x, n.y - r * 1.4), 0.03, [0.40, 0.30, 0.20, 1.0]);
                    for k in 0..5 {
                        let c = if k % 2 == 0 { [0.70, 0.56, 0.36, 1.0] } else { [0.56, 0.42, 0.26, 1.0] };
                        g.circle(n.x, n.y - r * (1.0 - k as f32 * 0.35), r * (1.6 - (k as f32 - 2.0).abs() * 0.25), 18, c);
                    }
                    g.circle(n.x, n.y + r * 0.7, r * 0.35, 12, [0.08, 0.06, 0.04, 1.0]);
                }
            }
        }
    }

    fn render_creature(&self, g: &mut Gfx, c: &Creature, light: bool, units: &mut Units) {
        let spd = &SPECIES[c.sp];
        let len = spd.disp_len() * c.scale;
        let (x, y) = c.center();
        // 影
        if spd.beh.grounded() && !c.airborne && !self.has_water() {
            let fy = self.floor_at(c.x) + if spd.beh.grounded() { c.z * BAND } else { BAND * 0.5 };
            let a = (0.14 - (fy - c.y).abs() * 0.05).clamp(0.0, 0.14);
            if a > 0.01 {
                g.circle(c.x, fy, len * 0.35, 12, [0.0, 0.0, 0.0, a]);
            }
        }
        if let Some((stage, site)) = c.winter {
            self.render_wintering(g, c, stage, site);
            return;
        }
        let activity = c.activity();
        let drawn = units.creature(g, c.sp, Pose { x, y, len, facing: c.facing, pitch: c.pitch, time: c.phase + (c.seed % 997) as f32 * 0.07, activity });
        if !drawn {
            art::draw(g, c.sp, x, y, len, c.facing, c.pitch, c.phase, 1.0, activity, c.seed);
        }
        // 休む場所のかくれみの（葉・土・草）
        match c.rest {
            Some(Rest::UnderLeaf) => {
                let leaf = self.leaf_col(0.2).unwrap_or([0.55, 0.45, 0.25]);
                let (lx, ly) = (x + len * 0.08 * c.facing, y - len * 0.42);
                let lc = c4(mix(leaf, [0.0, 0.0, 0.0], 0.1), 0.97);
                let n = 14;
                for k in 0..n {
                    let a0 = k as f32 / n as f32 * std::f32::consts::TAU;
                    let a1 = (k + 1) as f32 / n as f32 * std::f32::consts::TAU;
                    g.tri((lx, ly), (lx + a0.cos() * len * 0.62, ly + a0.sin() * len * 0.26), (lx + a1.cos() * len * 0.62, ly + a1.sin() * len * 0.26), lc);
                }
                g.line((lx - len * 0.6, ly), (lx + len * 0.6, ly), len * 0.03, c4(mix(leaf, [1.0, 1.0, 1.0], 0.3), 0.8));
            }
            Some(Rest::Litter) => {
                let (gc, _) = self.ground_cols();
                let top = y - len * 0.02;
                g.tri((x - len * 0.75, top + len * 0.35), (x + len * 0.75, top + len * 0.35), (x, top - len * 0.08), c4(mix(gc, [0.0, 0.0, 0.0], 0.15), 1.0));
                g.rect(x - len * 0.75, top + len * 0.2, x + len * 0.75, top + len * 0.4, c4(mix(gc, [0.0, 0.0, 0.0], 0.15), 1.0));
                for k in 0..4 {
                    let lx = x + (k as f32 - 1.5) * len * 0.35;
                    let c = mix([0.62, 0.42, 0.20], [0.42, 0.30, 0.16], k as f32 / 3.0);
                    g.tri((lx - len * 0.2, top + len * 0.1), (lx + len * 0.2, top + len * 0.04), (lx, top + len * 0.2), c4(c, 1.0));
                }
            }
            Some(Rest::Hideout) => {
                let base = y + len * 0.3;
                for k in 0..5 {
                    let dx = (k as f32 - 2.0) * len * 0.22;
                    let col = if k % 2 == 0 { [0.30, 0.50, 0.22, 1.0] } else { [0.40, 0.58, 0.26, 1.0] };
                    g.tri((x + dx - len * 0.06, base), (x + dx + len * 0.06, base), (x + dx * 1.6, base - len * (0.55 + (k % 2) as f32 * 0.15)), col);
                }
            }
            Some(Rest::Nest) => {
                // 巣の入り口の縁を手前に
                if let Some(nest) = self.nests.iter().find(|n| (n.x - c.x).abs() < len * 1.5 && (n.y - c.y).abs() < len) {
                    if nest.kind == 0 {
                        let (gc, _) = self.ground_cols();
                        g.rect(nest.x - len * 0.9, nest.y + len * 0.05, nest.x + len * 0.9, nest.y + len * 0.25, c4(mix(gc, [0.0, 0.0, 0.0], 0.2), 1.0));
                    }
                }
            }
            _ => {}
        }
        // ホタルの光（暗いほど目立つ）
        if spd.art == crate::life::Art::Firefly && !light {
            let glow = (c.phase * 2.4).sin().max(0.0);
            let k = if self.time == 3 { 0.9 } else if self.time == 2 { 0.55 } else { 0.35 };
            g.radial(x, y, len * if self.time == 3 { 2.4 } else { 1.4 }, 14, [0.85, 1.0, 0.45, k * glow], [0.85, 1.0, 0.45, 0.0]);
        }
    }

    /// 冬越しの姿と、そのまわり（土の部屋・落ち葉・泥・石）
    fn render_wintering(&self, g: &mut Gfx, c: &Creature, stage: Stage, site: Site) {
        let len = c.len();
        let (x, y) = (c.x, c.y);
        let (gc, deep) = self.ground_cols();
        if site == Site::Soil {
            // 土の中の部屋
            let n = 16;
            let (rx, ry) = (len * 0.75, len * 0.5);
            for k in 0..n {
                let a0 = k as f32 / n as f32 * std::f32::consts::TAU;
                let a1 = (k + 1) as f32 / n as f32 * std::f32::consts::TAU;
                g.tri((x, y), (x + a0.cos() * rx, y + a0.sin() * ry), (x + a1.cos() * rx, y + a1.sin() * ry), c4(mix(deep, [0.0, 0.0, 0.0], 0.35), 1.0));
            }
            if stage == Stage::CicadaNymph {
                g.line((x + len * 0.5, y - len * 0.3), (x + len * 1.4, y - len * 1.4), len * 0.06, [0.62, 0.50, 0.34, 1.0]);
            }
        }
        art::draw_stage(g, c.sp, stage, x, y, len, c.facing, c.pitch, c.phase, 1.0, c.seed);
        match site {
            Site::Litter | Site::Bank => {
                // 落ち葉をかぶっている
                for k in 0..5 {
                    let lx = x + (k as f32 - 2.0) * len * 0.3;
                    let col = mix([0.66, 0.44, 0.20], [0.46, 0.32, 0.16], (k % 3) as f32 / 2.0);
                    g.tri((lx - len * 0.25, y - len * 0.02), (lx + len * 0.25, y - len * 0.1), (lx, y + len * 0.12), c4(col, 1.0));
                }
            }
            Site::WaterBottom => {
                let mud = [0.30, 0.26, 0.18];
                g.rect(x - len * 0.7, y + len * 0.1, x + len * 0.7, y + len * 0.3, c4(mud, 1.0));
                g.tri((x - len * 0.7, y + len * 0.12), (x + len * 0.7, y + len * 0.12), (x, y - len * 0.02), c4(mud, 0.9));
                let _ = gc;
            }
            Site::InTrunk => {
                // 幹の中：樹皮の穴から見える
                let bark = [0.30, 0.22, 0.16, 0.9];
                g.rect(x - len * 0.35, y - len * 0.75, x + len * 0.35, y - len * 0.45, bark);
                g.rect(x - len * 0.35, y + len * 0.45, x + len * 0.35, y + len * 0.75, bark);
            }
            Site::LeafUnder => {
                let leaf = [0.16, 0.36, 0.18, 0.97];
                let (lx, ly) = (x, y - len * 0.35);
                for k in 0..14 {
                    let a0 = k as f32 / 14.0 * std::f32::consts::TAU;
                    let a1 = (k + 1) as f32 / 14.0 * std::f32::consts::TAU;
                    g.tri((lx, ly), (lx + a0.cos() * len * 0.7, ly + a0.sin() * len * 0.28), (lx + a1.cos() * len * 0.7, ly + a1.sin() * len * 0.28), leaf);
                }
            }
            Site::UnderStone => {
                // 石を少し持ち上げたところ
                g.circle(x + len * 0.25, y - len * 0.25, len * 0.45, 14, [0.52, 0.50, 0.46, 1.0]);
            }
            _ => {}
        }
    }

    fn render_capsule(&self, g: &mut Gfx, light: bool, units: &mut Units) {
        let c = &self.cap;
        let (x, y) = (c.x, c.y + (self.t * 1.6).sin() * 0.015);
        let r = CAP_R;
        let f = c.facing;
        if light {
            let lx = x + f * r * 0.9;
            g.tri3((lx, y), (lx + f * 2.8, y - 1.1), (lx + f * 2.8, y + 1.1), [1.0, 0.98, 0.84, 0.32], [1.0, 0.98, 0.84, 0.0], [1.0, 0.98, 0.84, 0.0]);
        }
        if self.has_water() && y > 0.0 && self.cap_speed() > 0.3 {
            for i in 0..5 {
                let k = (self.t * 3.0 + i as f32 * 0.37) % 1.0;
                g.circle(x - f * (r + k * 0.6), y + 0.08 - k * 0.2 + (i as f32).sin() * 0.05, 0.02 + k * 0.02, 8, [0.9, 1.0, 1.0, 0.6 * (1.0 - k)]);
            }
        }
        if !self.has_water() || y < 0.0 {
            let fy = if self.has_water() { 0.0 } else { self.floor_at(x) + BAND * 0.5 };
            let a = (0.35 - (fy - y) * 0.04).clamp(0.05, 0.35);
            g.circle(x, fy, r * 0.9, 16, [0.0, 0.0, 0.0, a * 0.35]);
        }
        if units.capsule(g, x, y, r, f, c.tilt) {
            return;
        }
        g.circle(x, y, r, 20, [0.86, 0.88, 0.90, 1.0]);
        g.circle(x + f * r * 0.35, y - r * 0.1, r * 0.55, 20, [0.30, 0.70, 0.82, 0.95]);
        g.line((x - r * 1.2, y + r * 0.05), (x + r * 0.95, y + r * 0.05), 0.03, [0.98, 0.62, 0.20, 1.0]);
    }

    fn render_front(&self, g: &mut Gfx, x0: f32, y0: f32, x1: f32, y1: f32, light: bool) {
        // 漂う粒（花粉・ほこり・水の中のごみ）
        let vw = x1 - x0;
        let vh = y1 - y0;
        for m in &self.motes {
            let px = x0 + ((m.x * 97.0 + x0 * 0.3 * m.z).rem_euclid(1.0)) * vw;
            let py = y0 + ((m.y + y0 * 0.01 * m.z).rem_euclid(1.0)) * vh;
            let s = 0.015 * m.s * (0.5 + m.z) * (vw / 16.0).clamp(0.4, 2.0);
            if self.snowy() {
                g.circle(px, py, s * 2.2, 8, [1.0, 1.0, 1.0, 0.8 * m.z]);
            } else if matches!(self.month, 10 | 11) && matches!(self.hab, Habitat::Forest | Habitat::Mountain | Habitat::Park) && m.s > 1.1 {
                let c = mix([0.90, 0.40, 0.12], [0.95, 0.78, 0.20], m.z);
                let r = s * 3.0;
                g.tri((px - r, py), (px, py - r * 0.6), (px + r, py), c4(c, 0.9));
            } else {
                g.circle(px, py, s, 6, [1.0, 0.98, 0.85, 0.35 * m.z]);
            }
        }
        // 木かげの暗さ（ライトのまわりは明るい）
        let cy = (y0 + y1) * 0.5;
        let amb = self.ambient_at(cy.max(0.0).min(self.cap.y.max(-10.0)));
        let dark = ((1.0 - amb) * 0.8).clamp(0.0, 0.82);
        if dark > 0.02 {
            let col = if self.time == 3 { [0.01, 0.02, 0.07, dark] } else { [0.02, 0.05, 0.03, dark] };
            if light {
                let (lx, ly) = (self.cap.x + self.cap.facing * 0.8, self.cap.y);
                let r = 2.9;
                g.radial(lx, ly, r, 40, [0.02, 0.05, 0.03, 0.0], col);
                let big = vw.max(vh) * 2.0 + 20.0;
                g.ring(lx, ly, big, big - r, 40, col);
            } else {
                g.rect(x0, y0, x1, y1, col);
            }
        }
    }
}

fn self_floor(floor: &[f32], x: f32) -> f32 {
    let fx = (x / FSTEP).clamp(0.0, (floor.len() - 2) as f32);
    let i = fx as usize;
    let t = fx - i as f32;
    floor[i] * (1.0 - t) + floor[i + 1] * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_scene_runs_and_keeps_insects_in_bounds() {
        for (s, spot) in SPOTS.iter().enumerate() {
            for &h in spot.habs {
                let mut sc = Scene::new(s, h, [7, 1, 4, 10][s % 4], (s % 4) as u8, 7 + s as u32);
                for _ in 0..400 {
                    sc.update(1.0 / 30.0, 0.6, 0.1, None, true);
                }
                assert!(sc.cap.y <= sc.floor_at(sc.cap.x) + BAND + 1.2 + 1e-3);
                for c in &sc.creatures {
                    assert!(c.x.is_finite() && c.y.is_finite(), "{}", SPECIES[c.sp].slug);
                    assert!(c.y > sc.top() - 1.0 && c.y < sc.floor_at(c.x) + BAND + 1.3, "{} at {}", SPECIES[c.sp].slug, c.y);
                }
            }
        }
    }

    /// どの場面にも、季節の中ならどの時間帯にも虫がいる
    #[test]
    fn no_time_of_day_is_empty() {
        for (s, spot) in SPOTS.iter().enumerate() {
            for &h in spot.habs {
                for month in 1..=12u8 {
                    for time in 0..4u8 {
                        let sc = Scene::new(s, h, month, time, 3);
                        assert!(!sc.creatures.is_empty(), "{} {:?} month {} time {}", spot.name.0, h, month, time);
                        for c in &sc.creatures {
                            assert!(c.x.is_finite() && c.y.is_finite());
                            assert_eq!(c.winter.is_some(), !SPECIES[c.sp].in_season(month));
                        }
                        let any_month = Scene::new(s, h, month, 1, 3);
                        assert_eq!(sc.present, any_month.present, "time must not change who lives here");
                    }
                }
            }
        }
    }

    #[test]
    fn photo_scores_close_subject_higher() {
        let s = Scene::new(S_SHOWA, Habitat::Meadow, 7, 1, 11);
        let c = &s.creatures[0];
        let (x, y) = c.center();
        let near = View { cx: x, cy: y, ppm: 1600.0, w: 1600.0, h: 900.0 };
        let far = View { cx: x + 3.0, cy: y, ppm: 100.0, w: 1600.0, h: 900.0 };
        let sn = s.shoot(&near, true).subjects.iter().find(|q| q.0 == c.sp).map(|q| q.1).unwrap_or(0.0);
        let sf = s.shoot(&far, true).subjects.iter().find(|q| q.0 == c.sp).map(|q| q.1).unwrap_or(0.0);
        assert!(sn > sf, "near {} far {}", sn, sf);
        assert_eq!(stars_of(sn), 1);
    }
}
