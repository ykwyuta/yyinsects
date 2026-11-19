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
}

impl Creature {
    /// 動いている度合い（0〜1）。はばたき・脚の動き
    pub fn activity(&self) -> f32 {
        let beh = SPECIES[self.sp].beh;
        if matches!(beh, Beh::Flutter | Beh::Fly) || self.airborne {
            return 1.0;
        }
        (self.vx.abs() / SPECIES[self.sp].speed.max(0.01)).clamp(0.0, 1.0)
    }

    /// 絵の中心
    pub fn center(&self) -> (f32, f32) {
        let sp = &SPECIES[self.sp];
        if sp.beh.grounded() && !self.airborne {
            (self.x, self.y - art::base_lift(self.sp) * sp.disp_len() * self.scale)
        } else {
            (self.x, self.y)
        }
    }

    pub fn bounds(&self) -> [f32; 4] {
        let (cx, cy) = self.center();
        units::bounds(self.sp, cx, cy, SPECIES[self.sp].disp_len() * self.scale, self.vertical)
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
    pub w: f32,
    /// 地面（水のある場面は水底。帯の奥の縁）の y
    floor: Vec<f32>,
    pub creatures: Vec<Creature>,
    pub rocks: Vec<Rock>,
    pub motes: Vec<Mote>,
    pub trees: Vec<Tree>,
    pub plants: Vec<Plant>,
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
    pub fn new(spot: usize, hab: Habitat, seed: u32) -> Scene {
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
            w,
            floor,
            creatures: Vec::new(),
            rocks: Vec::new(),
            motes: Vec::new(),
            trees: Vec::new(),
            plants: Vec::new(),
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
            let col = if kind == 1 { flower_cols[rng.below(flower_cols.len() as u32) as usize] } else { mix([0.30, 0.55, 0.22], [0.50, 0.66, 0.28], rng.f32()) };
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
            if !lives_here(sp, spot, hab) {
                continue;
            }
            s.present.push(i);
            let count = ((sp.rarity * 1.4 * rng.range(0.7, 1.3)).round() as usize).clamp(1, 10);
            for _ in 0..count {
                s.spawn(i);
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
        });
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
        let base = match self.hab {
            Habitat::Forest => 0.5,
            Habitat::Mountain => 0.6,
            _ => 1.0,
        };
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
                            cr.y += cr.vy * dt;
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
        let mut best: Vec<(usize, f32)> = Vec::new();
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
            let inside = frac * 20.0;
            let lit = self.light_at(cx, cy, light);
            let score = (size_pts + center_pts + inside) * (0.2 + 0.8 * lit.min(1.0).powf(0.7)) * blur;
            match best.iter_mut().find(|b| b.0 == cr.sp) {
                Some(b) => b.1 = b.1.max(score),
                None => best.push((cr.sp, score)),
            }
        }
        best.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        ShotResult { subjects: best }
    }

    /// カプセルの高さ・深さの説明
    pub fn depth_text(&self, y: f32) -> String {
        if self.has_water() && y > 0.0 {
            tf!("水深 {:.0}cm", "{:.0} cm deep", "水深 {:.0}厘米", y * CM_PER_UNIT)
        } else {
            let ground = if self.has_water() { 0.0 } else { self.floor_at(self.cap.x) + BAND * 0.45 };
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

    fn sky_cols(&self) -> ([f32; 3], [f32; 3]) {
        match self.hab {
            Habitat::Forest | Habitat::Mountain => ([0.40, 0.58, 0.72], [0.74, 0.84, 0.80]),
            Habitat::Shore => ([0.30, 0.56, 0.88], [0.80, 0.90, 0.96]),
            _ => ([0.30, 0.54, 0.86], [0.78, 0.88, 0.92]),
        }
    }

    fn ground_cols(&self) -> ([f32; 3], [f32; 3]) {
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

    fn render_back(&self, g: &mut Gfx, x0: f32, y0: f32, x1: f32, y1: f32) {
        let (top, hor) = self.sky_cols();
        let yh = if self.has_water() { 0.0 } else { self.floor_at((x0 + x1) * 0.5) - 0.3 };
        let sy0 = y0.min(yh - 6.0);
        g.rect_vgrad(x0, sy0, x1, yh + 0.5, c4(top, 1.0), c4(hor, 1.0));
        if !self.has_water() {
            g.rect(x0, yh, x1, y1, c4(hor, 1.0));
        }
        // 太陽と雲
        let sunx = x0 + (x1 - x0) * 0.8;
        g.radial(sunx, yh - 4.0, 1.6, 24, [1.0, 0.98, 0.86, 0.6], [1.0, 0.9, 0.7, 0.0]);
        g.circle(sunx, yh - 4.0, 0.35, 20, [1.0, 0.99, 0.92, 0.95]);
        for i in 0..5 {
            let cx = ((i as f32 * 7.3 + self.t * 0.05 - (x0 + x1) * 0.4) % 40.0 + 40.0) % 40.0 + x0 - 5.0;
            let cy = yh - 3.6 - (i % 3) as f32 * 0.5;
            for k in 0..4 {
                g.circle(cx + k as f32 * 0.4, cy - (k % 2) as f32 * 0.2, 0.35 + (k % 2) as f32 * 0.15, 14, [1.0, 1.0, 1.0, 0.55]);
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
            let leaf = if self.hab == Habitat::Mountain { [0.18, 0.40, 0.22] } else { [0.24, 0.48, 0.22] };
            for i in 0..7 {
                let a = i as f32 * 1.7 + tr.tone * 3.0;
                let lx = tr.x + a.sin() * 1.5;
                let ly = top + 1.0 + (i % 3) as f32 * 0.7;
                g.circle(lx, ly, 1.1 + (i % 2) as f32 * 0.4, 16, c4(mix(leaf, [0.5, 0.7, 0.3], (i % 3) as f32 * 0.2), 0.95));
            }
        }
        // 森の上は木の葉でおおわれる
        if matches!(self.hab, Habitat::Forest | Habitat::Mountain) {
            let top = self.top() - 1.0;
            let mut x = (x0 / 0.8).floor() * 0.8;
            while x < x1 + 1.0 {
                let h = crate::rng::hash2((x * 10.0) as i32, 1, self.seed);
                g.circle(x, top + h * 0.6, 0.9 + h * 0.5, 14, [0.20, 0.40, 0.20, 0.95]);
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
                    // イネ（水の中から立つ）
                    for k in 0..4 {
                        let dx = (k as f32 - 1.5) * 0.04;
                        g.line((p.x + dx, base), (p.x + dx * 2.5 + sway, base - p.h - 0.4), 0.025, c4(mix(p.col, [0.6, 0.7, 0.3], 0.3), 1.0));
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
        let activity = c.activity();
        let drawn = units.creature(g, c.sp, Pose { x, y, len, facing: c.facing, pitch: c.pitch, time: c.phase + (c.seed % 997) as f32 * 0.07, activity });
        if !drawn {
            art::draw(g, c.sp, x, y, len, c.facing, c.pitch, c.phase, 1.0, activity, c.seed);
        }
        // ホタルの光（暗いほど目立つ）
        if spd.art == crate::life::Art::Firefly && !light {
            let glow = (c.phase * 2.4).sin().max(0.0);
            g.radial(x, y, len * 1.4, 14, [0.85, 1.0, 0.45, 0.35 * glow], [0.85, 1.0, 0.45, 0.0]);
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
            g.circle(px, py, s, 6, [1.0, 0.98, 0.85, 0.35 * m.z]);
        }
        // 木かげの暗さ（ライトのまわりは明るい）
        let cy = (y0 + y1) * 0.5;
        let amb = self.ambient_at(cy.max(0.0).min(self.cap.y.max(-10.0)));
        let dark = ((1.0 - amb) * 0.75).clamp(0.0, 0.8);
        if dark > 0.02 {
            let col = [0.02, 0.05, 0.03, dark];
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
                let mut sc = Scene::new(s, h, 7 + s as u32);
                assert!(!sc.present.is_empty());
                for _ in 0..400 {
                    sc.update(1.0 / 30.0, 0.6, 0.1, None, true);
                }
                assert!(sc.cap.y <= sc.floor_at(sc.cap.x) + BAND * 0.45 - CAP_R + 1e-3);
                for c in &sc.creatures {
                    assert!(c.x.is_finite() && c.y.is_finite(), "{}", SPECIES[c.sp].slug);
                    assert!(c.y > sc.top() - 1.0 && c.y < sc.floor_at(c.x) + BAND + 0.5, "{} at {}", SPECIES[c.sp].slug, c.y);
                }
            }
        }
    }

    #[test]
    fn photo_scores_close_subject_higher() {
        let s = Scene::new(S_SHOWA, Habitat::Meadow, 11);
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
