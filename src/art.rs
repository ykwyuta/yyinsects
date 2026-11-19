//! 昆虫の絵（三角形で手続き的に描く。画像が無いときの絵）
//!
//! 体の座標は u（体の長さ方向、頭が +0.5・おしりが -0.5）と v（上）で書き、
//! `Pen` が場面（または画面）の座標に移す。y は下向きが正。絵の中心（u=0, v=0）を (x, y) に置く。

use crate::gfx::Gfx;
use crate::life::{Art, Stage, SPECIES};
use std::f32::consts::{PI, TAU};

#[derive(Clone, Copy)]
pub struct Pen {
    pub x: f32,
    pub y: f32,
    /// 体長（座標の単位）
    pub s: f32,
    /// 向き（+1 右、-1 左）
    pub fx: f32,
    /// 体の傾き（ラジアン、頭が上へ向くのが正）
    pub rot: f32,
    pub a: f32,
}

impl Pen {
    #[inline]
    pub fn p(&self, u: f32, v: f32) -> (f32, f32) {
        let (sn, cs) = self.rot.sin_cos();
        let (uu, vv) = (u * cs - v * sn, u * sn + v * cs);
        (self.x + uu * self.s * self.fx, self.y - vv * self.s)
    }
}

#[inline]
fn ca(c: [f32; 3], a: f32) -> [f32; 4] {
    [c[0], c[1], c[2], a]
}

#[inline]
fn mul(c: [f32; 3], k: f32) -> [f32; 3] {
    [(c[0] * k).min(1.0), (c[1] * k).min(1.0), (c[2] * k).min(1.0)]
}

#[inline]
fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

const INK: [f32; 3] = [0.06, 0.05, 0.05];
const WHITE: [f32; 3] = [1.0, 1.0, 1.0];

/// 楕円（体の座標。ang は体の座標での傾き）
fn ell(g: &mut Gfx, pen: &Pen, u: f32, v: f32, ru: f32, rv: f32, ang: f32, c: [f32; 3], a: f32) {
    let n = 18;
    let (sa, cs) = ang.sin_cos();
    let col = ca(c, a * pen.a);
    let ctr = pen.p(u, v);
    let mut prev = pen.p(u + ru * cs, v + ru * sa);
    for i in 1..=n {
        let th = i as f32 / n as f32 * TAU;
        let (st, ct) = th.sin_cos();
        let (lu, lv) = (ru * ct, rv * st);
        let q = pen.p(u + lu * cs - lv * sa, v + lu * sa + lv * cs);
        g.tri(ctr, prev, q, col);
        prev = q;
    }
}

fn circ(g: &mut Gfx, pen: &Pen, u: f32, v: f32, r: f32, c: [f32; 3], a: f32) {
    let (x, y) = pen.p(u, v);
    g.circle(x, y, (r * pen.s).max(0.0005), 12, ca(c, a * pen.a));
}

/// 多角形（凸に近いもの。最初の点から扇に塗る）
fn poly(g: &mut Gfx, pen: &Pen, pts: &[(f32, f32)], c: [f32; 3], a: f32) {
    if pts.len() < 3 {
        return;
    }
    let col = ca(c, a * pen.a);
    let p0 = pen.p(pts[0].0, pts[0].1);
    for w in pts[1..].windows(2) {
        g.tri(p0, pen.p(w[0].0, w[0].1), pen.p(w[1].0, w[1].1), col);
    }
}

fn seg(g: &mut Gfx, pen: &Pen, u0: f32, v0: f32, u1: f32, v1: f32, w: f32, c: [f32; 3], a: f32) {
    g.line(pen.p(u0, v0), pen.p(u1, v1), (w * pen.s).max(0.0006), ca(c, a * pen.a));
}

/// 折れ線
fn path(g: &mut Gfx, pen: &Pen, pts: &[(f32, f32)], w: f32, c: [f32; 3], a: f32) {
    for p in pts.windows(2) {
        seg(g, pen, p[0].0, p[0].1, p[1].0, p[1].1, w, c, a);
    }
}

/// 6本の脚（付け根の u、付け根の v、足先の v、歩く動き）
fn legs(g: &mut Gfx, pen: &Pen, roots: [f32; 3], v0: f32, foot: f32, reach: f32, t: f32, walk: f32, w: f32, c: [f32; 3]) {
    for (k, &u) in roots.iter().enumerate() {
        for side in 0..2 {
            let ph = t * 9.0 + k as f32 * 2.1 + side as f32 * PI;
            let sw = ph.sin() * walk * 0.06;
            let lift = (ph.cos().max(0.0)) * walk * 0.03;
            let dir = (k as f32 - 1.0) * reach;
            let shade = if side == 0 { 1.0 } else { 0.7 };
            let knee = (u + dir * 0.6 + sw * 0.5, v0 + (foot - v0) * 0.35 + 0.05);
            let tip = (u + dir + sw, foot + lift);
            path(g, pen, &[(u, v0), knee, tip], w, mul(c, shade), 1.0);
        }
    }
}

/// 触角
fn antenna(g: &mut Gfx, pen: &Pen, u: f32, v: f32, len: f32, up: f32, curl: f32, w: f32, c: [f32; 3], t: f32) {
    let wob = (t * 3.0).sin() * 0.03;
    let mut pts = Vec::with_capacity(6);
    for i in 0..6 {
        let f = i as f32 / 5.0;
        let ang = up + curl * f + wob * f;
        pts.push((u + ang.cos() * len * f, v + ang.sin() * len * f));
    }
    path(g, pen, &pts, w, c, 1.0);
}

/// 羽ばたき（0〜1：はねの開き）
fn flap(t: f32, speed: f32) -> f32 {
    0.15 + 0.85 * (t * speed).sin().abs()
}

/// 図鑑の絵の大きさ（体長に対する幅・高さ）
pub fn extent(sp: usize) -> (f32, f32) {
    match SPECIES[sp].art {
        Art::Butterfly(1) | Art::Butterfly(5) => (1.28, 1.62),
        Art::Butterfly(_) => (0.95, 1.6),
        Art::Dragonfly(_) => (1.22, 1.05),
        Art::Scarab(1) => (1.3, 0.9),
        Art::Scarab(_) => (1.06, 0.65),
        Art::Stag(_) => (1.35, 0.65),
        Art::Ladybug => (1.08, 0.86),
        Art::Firefly => (1.34, 0.55),
        Art::Longhorn => (1.62, 0.65),
        Art::TigerBeetle => (1.25, 0.68),
        Art::Jewel => (1.07, 0.52),
        Art::DivingBeetle => (1.17, 0.5),
        Art::Cicada(_) => (1.28, 0.56),
        Art::WaterBug(0) => (1.6, 0.45),
        Art::WaterBug(_) => (1.4, 0.5),
        Art::Strider => (2.27, 0.35),
        Art::StinkBug => (1.27, 0.6),
        Art::Hopper(0) => (1.24, 0.8),
        Art::Hopper(1) => (1.5, 0.8),
        Art::Hopper(2) => (2.3, 1.08),
        Art::Hopper(_) => (2.0, 0.8),
        Art::Mantis(_) => (1.16, 1.36),
        Art::Stick => (1.6, 0.48),
        Art::Bee(_) => (1.08, 1.04),
        Art::Ant => (1.27, 0.68),
        Art::Earwig => (1.38, 0.45),
    }
}

/// 足元から絵の中心までの高さ（体長に対する割合）
pub fn base_lift(sp: usize) -> f32 {
    extent(sp).1 * 0.5
}

/// 虫を描く。(x, y) は絵の中心。len は体長、facing は向き、pitch は傾き、t は動きの時間
#[allow(clippy::too_many_arguments)]
pub fn draw(g: &mut Gfx, sp: usize, x: f32, y: f32, len: f32, facing: f32, pitch: f32, t: f32, a: f32, activity: f32, seed: u32) {
    let s = &SPECIES[sp];
    let pen = Pen { x, y, s: len, fx: if facing < 0.0 { -1.0 } else { 1.0 }, rot: pitch, a };
    let (c, c2) = (s.col, s.col2);
    let t = t + (seed % 97) as f32 * 0.13;
    match s.art {
        Art::Butterfly(k) => butterfly(g, &pen, k, c, c2, t, activity),
        Art::Dragonfly(k) => dragonfly(g, &pen, k, c, c2, t, activity),
        Art::Scarab(k) => scarab(g, &pen, k, c, c2, t, activity),
        Art::Stag(k) => stag(g, &pen, k, c, c2, t, activity),
        Art::Ladybug => ladybug(g, &pen, c, c2, t, activity),
        Art::Firefly => firefly(g, &pen, c, c2, t, activity),
        Art::Longhorn => longhorn(g, &pen, c, c2, t, activity),
        Art::TigerBeetle => tiger(g, &pen, c, c2, t, activity),
        Art::Jewel => jewel(g, &pen, c, c2, t, activity),
        Art::DivingBeetle => diving(g, &pen, c, c2, t, activity),
        Art::Cicada(k) => cicada(g, &pen, k, c, c2, t),
        Art::WaterBug(k) => water_bug(g, &pen, k, c, c2, t, activity),
        Art::Strider => strider(g, &pen, c, c2, t, activity),
        Art::StinkBug => stink(g, &pen, c, c2, t, activity),
        Art::Hopper(k) => hopper(g, &pen, k, c, c2, t, activity),
        Art::Mantis(k) => mantis(g, &pen, k, c, c2, t, activity),
        Art::Stick => stick(g, &pen, c, c2, t, activity),
        Art::Bee(k) => bee(g, &pen, k, c, c2, t),
        Art::Ant => ant(g, &pen, c, c2, t, activity),
        Art::Earwig => earwig(g, &pen, c, c2, t, activity),
    }
}

// ---------------------------------------------------------------- 冬越しの姿（たまご・幼虫・さなぎ）

/// 冬越しの姿の絵の大きさ（体長に対する幅・高さ）
pub fn stage_extent(sp: usize, stage: Stage) -> (f32, f32) {
    match stage {
        Stage::Adult => extent(sp),
        Stage::Egg => (0.8, 0.4),
        Stage::EggCase => (0.95, 0.62),
        Stage::Caterpillar => (1.15, 0.4),
        Stage::Grub => (0.95, 0.75),
        Stage::Borer => (1.15, 0.32),
        Stage::Yago => (1.25, 0.55),
        Stage::CicadaNymph => (1.05, 0.65),
        Stage::FireflyLarva => (1.15, 0.32),
        Stage::BugNymph => (1.0, 0.62),
        Stage::Chrysalis => (0.95, 0.5),
        Stage::Cocoon => (0.95, 0.55),
    }
}

/// 冬越しの姿を描く。(x, y) は絵の中心
#[allow(clippy::too_many_arguments)]
pub fn draw_stage(g: &mut Gfx, sp: usize, stage: Stage, x: f32, y: f32, len: f32, facing: f32, pitch: f32, t: f32, a: f32, seed: u32) {
    if stage == Stage::Adult {
        draw(g, sp, x, y, len, facing, pitch, t, a, 0.0, seed);
        return;
    }
    let s = &SPECIES[sp];
    let pen = Pen { x, y, s: len, fx: if facing < 0.0 { -1.0 } else { 1.0 }, rot: pitch, a };
    let breathe = (t * 1.5).sin() * 0.01;
    match stage {
        Stage::Egg => {
            let c = mix([0.95, 0.90, 0.72], s.col, 0.15);
            for k in 0..8 {
                let u = -0.3 + (k % 4) as f32 * 0.2 + if k >= 4 { 0.1 } else { 0.0 };
                let v = if k >= 4 { 0.08 } else { -0.08 };
                ell(g, &pen, u, v, 0.075, 0.1, 0.3, c, 1.0);
                circ(g, &pen, u - 0.02, v + 0.04, 0.02, WHITE, 0.7);
            }
        }
        Stage::EggCase => {
            let c = [0.80, 0.70, 0.52];
            ell(g, &pen, 0.0, 0.0, 0.46, 0.28, 0.0, c, 1.0);
            for k in 0..6 {
                let u = -0.35 + k as f32 * 0.14;
                seg(g, &pen, u, -0.22, u + 0.04, 0.24, 0.025, mul(c, 0.8), 0.9);
            }
            ell(g, &pen, 0.0, 0.18, 0.36, 0.06, 0.0, mix(c, WHITE, 0.3), 0.7);
        }
        Stage::Caterpillar => {
            let c = mix(s.col, [0.40, 0.62, 0.26], 0.6);
            for k in 0..9 {
                let u = -0.48 + k as f32 * 0.11;
                let v = (t * 2.0 + k as f32 * 0.8).sin() * 0.012;
                ell(g, &pen, u, v, 0.075, 0.12, 0.0, if k % 2 == 0 { c } else { mul(c, 0.9) }, 1.0);
                seg(g, &pen, u, -0.1, u, -0.16, 0.02, mul(c, 0.6), 1.0);
            }
            circ(g, &pen, 0.5, 0.02, 0.08, mul(c, 0.75), 1.0);
            if s.slug == "great-purple-emperor" || s.slug == "hestina-persimilis" || s.slug == "red-ring-skirt" {
                // オオムラサキの幼虫の頭の角
                seg(g, &pen, 0.5, 0.06, 0.58, 0.2, 0.02, mul(c, 0.7), 1.0);
                seg(g, &pen, 0.48, 0.06, 0.5, 0.21, 0.02, mul(c, 0.7), 1.0);
            }
            for k in 0..4 {
                circ(g, &pen, -0.3 + k as f32 * 0.2, 0.06, 0.02, mix(s.col2, WHITE, 0.4), 0.8);
            }
        }
        Stage::Grub => {
            // C の字に丸まった白い幼虫
            let c = [0.94, 0.90, 0.80];
            for k in 0..10 {
                let th = PI * 0.15 + k as f32 / 9.0 * PI * 1.25;
                let r = 0.24;
                let w = 0.13 - k as f32 * 0.004;
                ell(g, &pen, th.cos() * r, th.sin() * r + breathe, w, w, 0.0, if k % 2 == 0 { c } else { mul(c, 0.94) }, 1.0);
            }
            circ(g, &pen, (PI * 0.15f32).cos() * 0.24 + 0.05, (PI * 0.15f32).sin() * 0.24, 0.1, [0.62, 0.40, 0.20], 1.0);
            for k in 0..3 {
                seg(g, &pen, 0.05 + k as f32 * 0.04, 0.0, 0.0 + k as f32 * 0.05, -0.1, 0.02, [0.62, 0.40, 0.20], 1.0);
            }
        }
        Stage::Borer => {
            let c = [0.95, 0.92, 0.82];
            for k in 0..9 {
                let u = -0.46 + k as f32 * 0.1;
                ell(g, &pen, u, breathe, 0.065, 0.1 - (k as f32 - 6.0).max(0.0) * 0.005, 0.0, if k % 2 == 0 { c } else { mul(c, 0.95) }, 1.0);
            }
            ell(g, &pen, 0.46, 0.0, 0.09, 0.11, 0.0, [0.92, 0.80, 0.50], 1.0);
            circ(g, &pen, 0.53, 0.0, 0.035, [0.30, 0.20, 0.12], 1.0);
        }
        Stage::Yago => {
            let c = [0.42, 0.38, 0.26];
            legs(g, &pen, [0.15, 0.05, -0.05], -0.02, -0.24, 0.2, t, 0.3, 0.025, mul(c, 0.8));
            ell(g, &pen, -0.22, 0.0, 0.34, 0.13, 0.0, c, 1.0);
            ell(g, &pen, 0.12, 0.02, 0.14, 0.11, 0.0, mul(c, 1.1), 1.0);
            ell(g, &pen, 0.04, 0.12, 0.14, 0.04, 0.15, mul(c, 0.8), 1.0);
            circ(g, &pen, 0.3, 0.04, 0.09, mul(c, 1.05), 1.0);
            circ(g, &pen, 0.34, 0.08, 0.045, [0.15, 0.15, 0.10], 1.0);
        }
        Stage::CicadaNymph => {
            let c = [0.66, 0.50, 0.30];
            legs(g, &pen, [0.12, 0.02, -0.08], -0.04, -0.28, 0.1, t, 0.0, 0.03, mul(c, 0.8));
            path(g, &pen, &[(0.22, 0.0), (0.38, -0.12), (0.46, -0.04)], 0.06, mul(c, 0.9), 1.0);
            ell(g, &pen, -0.15, 0.02, 0.32, 0.2, 0.0, c, 1.0);
            ell(g, &pen, 0.18, 0.06, 0.16, 0.17, 0.0, mul(c, 1.1), 1.0);
            ell(g, &pen, 0.0, 0.16, 0.16, 0.05, 0.1, mul(c, 0.8), 1.0);
            circ(g, &pen, 0.32, 0.06, 0.06, [0.85, 0.80, 0.70], 1.0);
        }
        Stage::FireflyLarva => {
            let c = [0.18, 0.16, 0.14];
            for k in 0..9 {
                let u = -0.46 + k as f32 * 0.1;
                ell(g, &pen, u, 0.0, 0.065, 0.1, 0.0, if k % 2 == 0 { c } else { mul(c, 1.3) }, 1.0);
                circ(g, &pen, u, 0.05, 0.02, [0.85, 0.75, 0.55], 0.8);
            }
            circ(g, &pen, 0.48, 0.0, 0.05, [0.30, 0.25, 0.20], 1.0);
            let glow = 0.4 + 0.3 * (t * 1.8).sin();
            circ(g, &pen, -0.46, 0.0, 0.05, [0.85, 1.0, 0.45], glow);
        }
        Stage::BugNymph => {
            legs(g, &pen, [0.12, 0.0, -0.12], -0.04, -0.24, 0.1, t, 0.0, 0.02, INK);
            ell(g, &pen, 0.0, 0.04, 0.34, 0.2, 0.0, mix(s.col, [0.5, 0.4, 0.3], 0.4), 1.0);
            for k in 0..3 {
                circ(g, &pen, -0.18 + k as f32 * 0.16, 0.1, 0.04, s.col2, 1.0);
            }
            circ(g, &pen, 0.34, 0.02, 0.07, s.col2, 1.0);
        }
        Stage::Chrysalis => {
            // チョウのさなぎ（角ばった形と、体を支える糸）
            let c = if (s.col[1] > 0.6) || s.slug.contains("swallowtail") { [0.62, 0.70, 0.40] } else { [0.58, 0.48, 0.34] };
            poly(g, &pen, &[(-0.44, 0.0), (-0.2, 0.16), (0.1, 0.2), (0.3, 0.12), (0.44, 0.18), (0.42, 0.0), (0.3, -0.12), (-0.1, -0.16)], c, 1.0);
            seg(g, &pen, -0.3, 0.06, 0.3, 0.1, 0.015, mul(c, 0.75), 0.8);
            seg(g, &pen, 0.05, 0.2, -0.1, 0.32, 0.01, WHITE, 0.6);
        }
        Stage::Cocoon => {
            let c = if s.slug == "winter-moth" { [0.42, 0.30, 0.20] } else { [0.70, 0.62, 0.44] };
            ell(g, &pen, 0.0, 0.0, 0.44, 0.24, 0.0, c, 1.0);
            for k in 0..5 {
                let u = -0.32 + k as f32 * 0.16;
                seg(g, &pen, u, -0.2, u + 0.06, 0.2, 0.012, mix(c, WHITE, 0.25), 0.7);
            }
        }
        Stage::Adult => {}
    }
}

// ---------------------------------------------------------------- チョウ・ガ

fn butterfly(g: &mut Gfx, pen: &Pen, k: u8, c: [f32; 3], c2: [f32; 3], t: f32, activity: f32) {
    // はねを立てて横から見た形。羽ばたくと、はねの面が縦につぶれる
    let f = if activity > 0.05 { flap(t, 7.0) } else { 0.85 + 0.15 * (t * 0.8).sin() };
    let body = [0.18, 0.15, 0.12];
    let base = (0.06, 0.04);
    // 後ろばね
    let hw: Vec<(f32, f32)> = match k {
        1 => vec![base, (-0.10, 0.55 * f), (-0.30, 0.50 * f), (-0.42, 0.30 * f), (-0.56, 0.16 * f), (-0.44, 0.10 * f), (-0.30, 0.02)],
        5 => vec![base, (-0.10, 0.52 * f), (-0.32, 0.46 * f), (-0.48, 0.22 * f), (-0.62, -0.10), (-0.40, 0.06), (-0.22, 0.0)],
        3 => vec![base, (-0.06, 0.42 * f), (-0.24, 0.38 * f), (-0.30, 0.18 * f), (-0.20, 0.02)],
        4 => vec![base, (-0.08, 0.52 * f), (-0.30, 0.50 * f), (-0.40, 0.26 * f), (-0.26, 0.02)],
        _ => vec![base, (-0.08, 0.50 * f), (-0.30, 0.46 * f), (-0.36, 0.22 * f), (-0.24, 0.02)],
    };
    // 前ばね
    let fw: Vec<(f32, f32)> = match k {
        3 => vec![base, (0.10, 0.48 * f), (0.08, 0.66 * f), (-0.12, 0.62 * f), (-0.16, 0.30 * f)],
        4 => vec![base, (0.18, 0.66 * f), (0.10, 0.82 * f), (-0.16, 0.72 * f), (-0.16, 0.30 * f)],
        _ => vec![base, (0.16, 0.62 * f), (0.10, 0.78 * f), (-0.14, 0.70 * f), (-0.18, 0.34 * f)],
    };
    let dark = mul(c, 0.82);
    poly(g, pen, &hw, dark, 1.0);
    poly(g, pen, &fw, c, 1.0);
    // 模様
    match k {
        0 => {
            circ(g, pen, -0.02, 0.48 * f, 0.045, c2, 0.9);
            circ(g, pen, 0.02, 0.68 * f, 0.03, c2, 0.8);
        }
        1 => {
            // しま模様とふちの黒
            for i in 0..4 {
                let u = 0.08 - i as f32 * 0.07;
                seg(g, pen, u, 0.1 * f, u - 0.06, 0.66 * f, 0.03, c2, 0.85);
            }
            path(g, pen, &[(0.10, 0.78 * f), (-0.14, 0.70 * f), (-0.30, 0.50 * f), (-0.42, 0.30 * f)], 0.045, c2, 0.9);
            circ(g, pen, -0.40, 0.22 * f, 0.035, [0.85, 0.25, 0.20], 1.0);
            seg(g, pen, -0.42, 0.20 * f, -0.60, 0.10 * f, 0.035, c2, 1.0);
        }
        2 => {
            path(g, pen, &[(0.08, 0.20 * f), (-0.02, 0.50 * f), (-0.20, 0.42 * f)], 0.06, c2, 0.8);
            for i in 0..3 {
                circ(g, pen, -0.06 - i as f32 * 0.09, 0.30 * f, 0.03, c2, 0.9);
            }
        }
        3 => {
            path(g, pen, &[(0.08, 0.66 * f), (-0.12, 0.62 * f), (-0.24, 0.38 * f)], 0.03, mul(c2, 0.6), 0.9);
            circ(g, pen, -0.04, 0.40 * f, 0.025, mul(c2, 0.4), 0.8);
        }
        4 => {
            // マダラ：黒いすじの網目
            path(g, pen, &[(0.06, 0.06), (0.06, 0.60 * f)], 0.02, c2, 0.8);
            path(g, pen, &[(0.06, 0.06), (-0.06, 0.66 * f)], 0.02, c2, 0.8);
            path(g, pen, &[(0.04, 0.06), (-0.14, 0.40 * f)], 0.02, c2, 0.8);
            path(g, pen, &[(0.10, 0.80 * f), (-0.16, 0.72 * f), (-0.30, 0.50 * f), (-0.40, 0.26 * f)], 0.05, c2, 0.9);
        }
        5 => {
            circ(g, pen, -0.02, 0.48 * f, 0.05, mix(c2, WHITE, 0.2), 0.9);
            circ(g, pen, -0.24, 0.30 * f, 0.04, mix(c2, WHITE, 0.2), 0.9);
            path(g, pen, &[(0.16, 0.62 * f), (0.10, 0.78 * f)], 0.03, c2, 0.7);
        }
        _ => {}
    }
    // 胴と頭
    ell(g, pen, -0.10, 0.0, 0.24, if k == 5 { 0.07 } else { 0.045 }, 0.0, body, 1.0);
    ell(g, pen, 0.14, 0.02, 0.08, 0.055, 0.0, body, 1.0);
    circ(g, pen, 0.24, 0.03, 0.045, body, 1.0);
    circ(g, pen, 0.26, 0.04, 0.02, mix(body, WHITE, 0.3), 1.0);
    if k == 5 {
        // ガのくし形の触角
        antenna(g, pen, 0.25, 0.06, 0.18, 1.0, -0.3, 0.035, mix(c2, body, 0.5), t);
    } else {
        antenna(g, pen, 0.25, 0.06, 0.30, 1.1, -0.2, 0.012, body, t);
        circ(g, pen, 0.25 + (1.1f32 - 0.2).cos() * 0.30, 0.06 + (0.9f32).sin() * 0.30, 0.018, body, 1.0);
    }
    legs(g, pen, [0.18, 0.13, 0.08], -0.02, -0.18, 0.12, t, activity * 0.3, 0.012, body);
}

// ---------------------------------------------------------------- トンボ

fn dragonfly(g: &mut Gfx, pen: &Pen, k: u8, c: [f32; 3], c2: [f32; 3], t: f32, activity: f32) {
    let f = if activity > 0.05 { flap(t, if k >= 3 { 6.0 } else { 16.0 }) } else { 0.55 };
    let thin = if k == 2 { 0.03 } else if k == 1 { 0.055 } else { 0.045 };
    let wing_col = match k {
        3 => [0.08, 0.08, 0.08],
        4 => mix(c2, [0.10, 0.08, 0.20], 0.4),
        _ => [0.85, 0.92, 0.98],
    };
    let wa = if k >= 3 { 0.92 } else { 0.38 };
    let wl = if k == 2 { 0.36 } else if k == 4 { 0.40 } else { 0.48 };
    let ww = if k == 4 { 0.18 } else if k == 3 { 0.10 } else { 0.09 };
    // 4枚のはね（奥の2枚・手前の2枚）。羽ばたくと上下に開く
    for (du, side) in [(0.06, 0.7), (-0.02, 0.7), (0.06, 1.0), (-0.02, 1.0)] {
        let ang = (0.35 + 0.9 * f) * if side < 1.0 { 1.0 } else { 0.8 };
        let (cx, cy) = (0.20 + du, 0.06);
        let tip = (cx - wl * 0.5 * (1.0 - ang.sin() * 0.3), cy + ang.sin() * wl);
        let mid = ((cx + tip.0) * 0.5, (cy + tip.1) * 0.5);
        ell(g, pen, mid.0, mid.1, wl * 0.5, ww * 0.5, (tip.1 - cy).atan2(tip.0 - cx), mul(wing_col, side), wa * if side < 1.0 { 0.8 } else { 1.0 });
    }
    // おなか（細長い）
    let tail = -0.62;
    for i in 0..10 {
        let f0 = i as f32 / 10.0;
        let u = 0.12 + (tail - 0.12) * f0;
        let col = if i % 2 == 0 { c } else { mix(c, c2, 0.6) };
        ell(g, pen, u, 0.0, 0.045, thin * (1.0 - f0 * 0.25), 0.0, col, 1.0);
    }
    if k == 1 {
        ell(g, pen, 0.06, 0.0, 0.06, thin * 1.2, 0.0, c2, 1.0);
    }
    // 胸と頭（大きな目）
    ell(g, pen, 0.22, 0.02, 0.10, 0.07, 0.2, c, 1.0);
    if k == 2 || k == 3 {
        circ(g, pen, 0.34, 0.04, 0.035, mul(c, 0.5), 1.0);
        circ(g, pen, 0.35, 0.07, 0.02, INK, 1.0);
    } else {
        circ(g, pen, 0.35, 0.05, 0.075, if k == 1 { mix(c, [0.2, 0.6, 0.4], 0.5) } else { mix(c, c2, 0.5) }, 1.0);
        circ(g, pen, 0.37, 0.08, 0.025, WHITE, 0.6);
    }
    legs(g, pen, [0.26, 0.22, 0.18], -0.03, -0.14, 0.06, t, 0.0, 0.012, INK);
}

// ---------------------------------------------------------------- 甲虫

/// 甲虫のふつうの体（横から見たドーム形）
fn beetle_body(g: &mut Gfx, pen: &Pen, c: [f32; 3], c2: [f32; 3], len: f32, height: f32, shine: f32) {
    let n = 12;
    let mut pts = vec![(-len * 0.5, -0.06)];
    for i in 0..=n {
        let a = i as f32 / n as f32 * PI;
        pts.push((-a.cos() * len * 0.5, -0.06 + a.sin() * height));
    }
    poly(g, pen, &pts, c, 1.0);
    // つやと上ばねの合わせ目
    ell(g, pen, len * 0.05, -0.06 + height * 0.62, len * 0.25, height * 0.18, 0.15, mix(c, WHITE, 0.45), shine);
    seg(g, pen, -len * 0.48, -0.04, len * 0.42, -0.06 + height * 0.35, 0.012, mul(c2, 0.8), 0.5);
}

fn scarab(g: &mut Gfx, pen: &Pen, k: u8, c: [f32; 3], c2: [f32; 3], t: f32, activity: f32) {
    legs(g, pen, [0.18, 0.04, -0.12], -0.04, -0.30, 0.14, t, activity, 0.035, mul(c, 0.7));
    let shine = if k == 2 { 0.7 } else { 0.4 };
    beetle_body(g, pen, if k == 2 { mix(c, c2, 0.5 + 0.3 * (t * 0.5).sin()) } else { c }, c2, 0.66, 0.30, shine);
    // 胸
    ell(g, pen, 0.28, 0.05, 0.11, 0.12, 0.0, mul(c, 1.05), 1.0);
    // 頭
    ell(g, pen, 0.40, -0.02, 0.07, 0.06, 0.0, mul(c, 0.8), 1.0);
    if k == 1 {
        // カブトムシの角
        let horn = [(0.40, 0.0), (0.50, 0.06), (0.56, 0.20), (0.56, 0.34)];
        path(g, pen, &horn, 0.06, mul(c, 0.9), 1.0);
        path(g, pen, &[(0.56, 0.30), (0.62, 0.40)], 0.035, mul(c, 0.9), 1.0);
        path(g, pen, &[(0.56, 0.30), (0.50, 0.42)], 0.035, mul(c, 0.9), 1.0);
        // 胸の小さな角
        path(g, pen, &[(0.30, 0.15), (0.40, 0.22)], 0.04, mul(c, 0.9), 1.0);
    }
    antenna(g, pen, 0.44, 0.0, 0.08, 0.6, 0.0, 0.015, INK, t);
}

fn stag(g: &mut Gfx, pen: &Pen, k: u8, c: [f32; 3], c2: [f32; 3], t: f32, activity: f32) {
    legs(g, pen, [0.10, -0.04, -0.18], -0.04, -0.30, 0.14, t, activity, 0.03, mul(c, 0.7));
    let _ = c2;
    beetle_body(g, pen, c, c2, 0.56, 0.24, 0.35);
    ell(g, pen, 0.20, 0.04, 0.10, 0.10, 0.0, mul(c, 1.05), 1.0);
    // 頭（ミヤマは耳のような出っぱり）
    ell(g, pen, 0.33, 0.04, 0.09, 0.08, 0.0, mul(c, 0.9), 1.0);
    if k == 2 {
        poly(g, pen, &[(0.28, 0.10), (0.34, 0.16), (0.38, 0.10)], mul(c, 0.9), 1.0);
        ell(g, pen, 0.0, 0.10, 0.2, 0.06, 0.0, mix(c, c2, 0.7), 0.35);
    }
    // 大あご
    let (len, curve) = match k {
        0 => (0.14, 0.4),
        1 => (0.26, 0.9),
        _ => (0.24, 0.5),
    };
    for side in [0.0f32, 1.0] {
        let col = mul(c, if side == 0.0 { 0.8 } else { 0.6 });
        let v0 = 0.05 - side * 0.02;
        let mut pts = Vec::new();
        for i in 0..6 {
            let f = i as f32 / 5.0;
            pts.push((0.40 + f * len, v0 + (f * PI * 0.5).sin() * len * 0.3 * curve - side * f * 0.04));
        }
        path(g, pen, &pts, 0.045 * (1.0 - side * 0.2), col, 1.0);
        if k == 1 {
            for i in 1..4 {
                let (pu, pv) = pts[i];
                seg(g, pen, pu, pv, pu + 0.01, pv - 0.04, 0.02, col, 1.0);
            }
        }
    }
    antenna(g, pen, 0.38, 0.06, 0.1, 1.2, -0.8, 0.015, INK, t);
}

fn ladybug(g: &mut Gfx, pen: &Pen, c: [f32; 3], c2: [f32; 3], t: f32, activity: f32) {
    legs(g, pen, [0.16, 0.0, -0.16], -0.06, -0.34, 0.1, t, activity, 0.04, INK);
    beetle_body(g, pen, c, c2, 0.86, 0.48, 0.55);
    for &(u, v) in &[(-0.22, 0.12), (0.0, 0.32), (0.14, 0.14), (-0.10, 0.04), (0.22, 0.30)] {
        ell(g, pen, u, v, 0.075, 0.06, 0.0, c2, 1.0);
    }
    // 頭（黒に白い点）
    ell(g, pen, 0.42, 0.02, 0.10, 0.11, 0.0, INK, 1.0);
    circ(g, pen, 0.40, 0.08, 0.035, WHITE, 1.0);
}

fn firefly(g: &mut Gfx, pen: &Pen, c: [f32; 3], c2: [f32; 3], t: f32, activity: f32) {
    let glow = 0.5 + 0.5 * (t * 2.4).sin().max(0.0);
    // おしりの光
    let (x, y) = pen.p(-0.40, 0.0);
    g.radial(x, y, pen.s * 0.25, 16, [0.85, 1.0, 0.45, 0.55 * glow * pen.a], [0.85, 1.0, 0.45, 0.0]);
    legs(g, pen, [0.18, 0.06, -0.06], -0.04, -0.24, 0.1, t, activity * 0.4, 0.02, INK);
    ell(g, pen, -0.06, 0.04, 0.40, 0.12, 0.0, c, 1.0);
    ell(g, pen, -0.40, 0.0, 0.10, 0.08, 0.0, mix([0.95, 1.0, 0.6], c, 0.3 - glow * 0.3), 1.0);
    // 赤い胸（十字の黒）
    ell(g, pen, 0.34, 0.04, 0.10, 0.10, 0.0, c2, 1.0);
    seg(g, pen, 0.34, -0.04, 0.34, 0.13, 0.025, INK, 0.8);
    circ(g, pen, 0.44, 0.0, 0.05, INK, 1.0);
    antenna(g, pen, 0.46, 0.02, 0.2, 0.7, -0.2, 0.014, INK, t);
}

fn longhorn(g: &mut Gfx, pen: &Pen, c: [f32; 3], c2: [f32; 3], t: f32, activity: f32) {
    legs(g, pen, [0.18, 0.02, -0.14], -0.04, -0.30, 0.15, t, activity, 0.025, mix(c, INK, 0.3));
    ell(g, pen, -0.10, 0.06, 0.36, 0.12, 0.0, c, 1.0);
    for &(u, v) in &[(-0.34, 0.08), (-0.16, 0.12), (0.04, 0.08), (-0.24, 0.0), (0.10, 0.12)] {
        ell(g, pen, u, v, 0.06, 0.045, 0.0, c2, 1.0);
    }
    ell(g, pen, 0.30, 0.06, 0.08, 0.08, 0.0, c, 1.0);
    circ(g, pen, 0.40, 0.04, 0.06, mul(c, 0.8), 1.0);
    // 長い触角（しま模様）
    for (side, w) in [(0.0, 1.0), (1.0, 0.8)] {
        let mut pts = Vec::new();
        for i in 0..10 {
            let f = i as f32 / 9.0;
            let ang = 1.2 - f * 2.2 - side * 0.25 + (t * 2.0).sin() * 0.03 * f;
            pts.push((0.42 + ang.cos() * 0.75 * f - f * 0.2, 0.08 + ang.sin() * 0.45 * f + f * 0.25));
        }
        for (i, p) in pts.windows(2).enumerate() {
            let col = if i % 2 == 0 { mul(c, w) } else { mix(c2, c, 0.3) };
            seg(g, pen, p[0].0, p[0].1, p[1].0, p[1].1, 0.022, col, 1.0);
        }
    }
}

fn tiger(g: &mut Gfx, pen: &Pen, c: [f32; 3], c2: [f32; 3], t: f32, activity: f32) {
    legs(g, pen, [0.16, 0.02, -0.12], -0.02, -0.32, 0.22, t, activity, 0.02, mix(c, c2, 0.3));
    ell(g, pen, -0.12, 0.04, 0.32, 0.12, 0.0, c, 1.0);
    ell(g, pen, -0.20, 0.08, 0.12, 0.05, 0.0, c2, 1.0);
    for &(u, v) in &[(-0.36, 0.06), (0.02, 0.10), (-0.08, 0.02)] {
        circ(g, pen, u, v, 0.03, [0.98, 0.95, 0.85], 1.0);
    }
    ell(g, pen, 0.24, 0.04, 0.08, 0.07, 0.0, mix(c, c2, 0.5), 1.0);
    ell(g, pen, 0.38, 0.04, 0.08, 0.08, 0.0, c, 1.0);
    circ(g, pen, 0.41, 0.07, 0.04, INK, 1.0);
    path(g, pen, &[(0.44, 0.0), (0.54, -0.02), (0.56, 0.03)], 0.025, [0.95, 0.92, 0.85], 1.0);
    antenna(g, pen, 0.42, 0.08, 0.22, 1.0, -0.4, 0.012, INK, t);
}

fn jewel(g: &mut Gfx, pen: &Pen, c: [f32; 3], c2: [f32; 3], t: f32, activity: f32) {
    legs(g, pen, [0.16, 0.02, -0.12], -0.04, -0.24, 0.1, t, activity, 0.02, mul(c, 0.6));
    let shimmer = 0.5 + 0.5 * (t * 0.7).sin();
    poly(g, pen, &[(0.30, 0.0), (0.26, 0.16), (-0.10, 0.18), (-0.40, 0.10), (-0.50, 0.0), (-0.36, -0.04), (0.20, -0.04)], mix(c, [0.4, 0.9, 0.6], shimmer * 0.3), 1.0);
    seg(g, pen, 0.26, 0.10, -0.44, 0.06, 0.05, c2, 0.95);
    ell(g, pen, -0.05, 0.14, 0.2, 0.03, 0.0, WHITE, 0.4);
    ell(g, pen, 0.34, 0.06, 0.07, 0.08, 0.0, mix(c, c2, 0.3), 1.0);
    circ(g, pen, 0.42, 0.04, 0.05, mul(c, 0.8), 1.0);
    antenna(g, pen, 0.44, 0.06, 0.1, 0.8, 0.0, 0.012, INK, t);
}

fn diving(g: &mut Gfx, pen: &Pen, c: [f32; 3], c2: [f32; 3], t: f32, activity: f32) {
    // オールのような後ろ足
    let row = (t * 6.0).sin() * (0.3 + activity);
    for side in [0.7f32, 1.0] {
        path(g, pen, &[(-0.10, -0.02), (-0.30, -0.10 - row * 0.05), (-0.55, -0.06 + row * 0.12)], 0.05, mul(c, side), 1.0);
    }
    legs(g, pen, [0.22, 0.10, 0.10], -0.02, -0.16, 0.08, t, activity * 0.5, 0.02, mul(c, 0.7));
    poly(g, pen, &[(0.40, 0.0), (0.30, 0.13), (0.0, 0.17), (-0.30, 0.14), (-0.46, 0.04), (-0.40, -0.06), (0.0, -0.08), (0.30, -0.06)], c, 1.0);
    path(g, pen, &[(0.30, 0.13), (0.0, 0.17), (-0.30, 0.14), (-0.46, 0.04)], 0.035, c2, 1.0);
    ell(g, pen, -0.02, 0.10, 0.22, 0.03, 0.0, WHITE, 0.3);
    circ(g, pen, 0.36, 0.04, 0.03, c2, 1.0);
}

// ---------------------------------------------------------------- セミ・カメムシ

fn cicada(g: &mut Gfx, pen: &Pen, k: u8, c: [f32; 3], c2: [f32; 3], t: f32) {
    // 鳴いているとおなかがふるえる
    let buzz = (t * 40.0).sin() * 0.004;
    legs(g, pen, [0.20, 0.10, 0.02], -0.06, -0.26, 0.08, t, 0.0, 0.025, mul(c, 0.7));
    ell(g, pen, -0.04, buzz, 0.30, 0.14, 0.0, mul(c, 0.9), 1.0);
    ell(g, pen, 0.22, 0.06, 0.14, 0.15, 0.0, c, 1.0);
    // 胸の模様
    ell(g, pen, 0.22, 0.12, 0.08, 0.05, 0.0, mix(c, c2, 0.4), 0.8);
    circ(g, pen, 0.36, 0.04, 0.08, mul(c, 0.85), 1.0);
    circ(g, pen, 0.38, 0.10, 0.04, mix(c, [0.6, 0.2, 0.1], 0.5), 1.0);
    // 屋根のように閉じたはね
    let wing = [(0.24, 0.16), (0.10, 0.26), (-0.30, 0.22), (-0.62, 0.06), (-0.56, -0.02), (-0.10, 0.04)];
    let (wc, wa) = match k {
        0 => (c2, 0.45),
        1 => (c2, 0.97),
        _ => (c2, 0.92),
    };
    poly(g, pen, &wing, wc, wa);
    if k == 2 {
        for i in 0..6 {
            circ(g, pen, -0.02 - i as f32 * 0.09, 0.10 + (i % 3) as f32 * 0.04, 0.035, mul(c2, 0.55), 0.9);
        }
    }
    // はねのすじ
    let vein = if k == 0 { mul(c, 0.8) } else { mul(c2, 0.6) };
    path(g, pen, &[(0.20, 0.18), (-0.20, 0.20), (-0.58, 0.04)], 0.012, vein, 0.9);
    path(g, pen, &[(0.10, 0.12), (-0.30, 0.10), (-0.50, 0.0)], 0.01, vein, 0.7);
    seg(g, pen, -0.25, 0.21, -0.28, 0.08, 0.01, vein, 0.7);
}

fn water_bug(g: &mut Gfx, pen: &Pen, k: u8, c: [f32; 3], c2: [f32; 3], t: f32, activity: f32) {
    let row = (t * 4.0).sin() * activity;
    match k {
        0 => {
            // 細長い体・長い呼吸管・カマのような前足
            seg(g, pen, -0.40, 0.0, 0.40, 0.0, 0.05, c, 1.0);
            seg(g, pen, -0.40, 0.0, -0.78, 0.02, 0.012, c2, 1.0);
            for (u, r) in [(0.10, 0.0), (-0.12, 1.0)] {
                path(g, pen, &[(u, 0.0), (u - 0.12 - row * 0.03, -0.12), (u - 0.30, -0.20 + r * 0.02)], 0.015, c2, 1.0);
            }
            path(g, pen, &[(0.34, 0.0), (0.48, 0.08), (0.58, -0.02), (0.50, -0.04)], 0.022, c, 1.0);
            circ(g, pen, 0.42, 0.01, 0.025, mul(c, 0.6), 1.0);
        }
        1 => {
            legs(g, pen, [0.10, -0.04, -0.16], -0.02, -0.18, 0.18, t, activity, 0.018, c2);
            ell(g, pen, -0.08, 0.02, 0.36, 0.10, 0.0, c, 1.0);
            seg(g, pen, -0.42, 0.02, -0.68, 0.04, 0.014, c2, 1.0);
            path(g, pen, &[(0.30, 0.02), (0.44, 0.10), (0.56, 0.02), (0.50, -0.02)], 0.035, c, 1.0);
            circ(g, pen, 0.32, 0.04, 0.04, mul(c, 0.6), 1.0);
        }
        _ => {
            for side in [0.7f32, 1.0] {
                path(g, pen, &[(-0.06, -0.02), (-0.26, -0.10 - row * 0.05), (-0.46, -0.04)], 0.035, mul(c, side), 1.0);
            }
            legs(g, pen, [0.18, 0.06, 0.06], -0.02, -0.12, 0.08, t, activity * 0.4, 0.02, mul(c, 0.7));
            poly(g, pen, &[(0.40, 0.0), (0.30, 0.12), (-0.10, 0.16), (-0.42, 0.06), (-0.40, -0.04), (0.20, -0.06)], c, 1.0);
            // 背中のたまご
            for i in 0..6 {
                circ(g, pen, -0.24 + i as f32 * 0.07, 0.17 + (i % 2) as f32 * 0.03, 0.04, c2, 1.0);
            }
            path(g, pen, &[(0.36, 0.0), (0.48, 0.02), (0.52, -0.06)], 0.03, c, 1.0);
        }
    }
}

fn strider(g: &mut Gfx, pen: &Pen, c: [f32; 3], c2: [f32; 3], t: f32, activity: f32) {
    let row = (t * 5.0).sin() * activity;
    // とても長い中脚・後脚（水面に広げる）
    for side in [0.75f32, 1.0] {
        path(g, pen, &[(0.04, 0.0), (0.0, 0.14), (-0.30 - row * 0.1, 0.02), (-0.70 - row * 0.15, -0.10)], 0.014, mul(c2, side), 1.0);
        path(g, pen, &[(-0.06, 0.0), (-0.16, 0.12), (-0.50, 0.0), (-0.95, -0.10)], 0.012, mul(c2, side), 1.0);
        path(g, pen, &[(0.22, 0.0), (0.32, 0.04), (0.40, -0.10)], 0.014, mul(c2, side), 1.0);
    }
    ell(g, pen, 0.0, 0.0, 0.30, 0.05, 0.0, c, 1.0);
    circ(g, pen, 0.30, 0.01, 0.04, c, 1.0);
    antenna(g, pen, 0.32, 0.02, 0.16, 0.5, -0.3, 0.01, c, t);
    // 足元の水面のくぼみ
    for u in [-0.70, -0.95, 0.40] {
        ell(g, pen, u - row * 0.1, -0.11, 0.07, 0.015, 0.0, WHITE, 0.5);
    }
}

fn stink(g: &mut Gfx, pen: &Pen, c: [f32; 3], c2: [f32; 3], t: f32, activity: f32) {
    legs(g, pen, [0.16, 0.02, -0.12], -0.04, -0.28, 0.1, t, activity, 0.025, INK);
    poly(g, pen, &[(0.30, 0.02), (0.22, 0.24), (-0.10, 0.26), (-0.40, 0.14), (-0.46, 0.0), (-0.30, -0.06), (0.20, -0.06)], c, 1.0);
    for i in 0..3 {
        let v = 0.04 + i as f32 * 0.08;
        seg(g, pen, 0.24, v, -0.42, v * 0.6, 0.035, c2, 1.0);
    }
    circ(g, pen, 0.36, 0.04, 0.06, c2, 1.0);
    antenna(g, pen, 0.40, 0.06, 0.22, 0.8, -0.5, 0.014, INK, t);
}

// ---------------------------------------------------------------- バッタ・カマキリ・ナナフシ

fn hopper(g: &mut Gfx, pen: &Pen, k: u8, c: [f32; 3], c2: [f32; 3], t: f32, activity: f32) {
    let pointed = k == 1;
    let dark = mul(c, 0.75);
    // 前脚・中脚
    legs(g, pen, [0.28, 0.16, 0.16], -0.04, -0.34, 0.08, t, activity, 0.02, dark);
    // おなか
    ell(g, pen, -0.20, 0.0, 0.32, 0.10, 0.05, c, 1.0);
    // はね（キリギリス・トノサマバッタ）
    if k != 3 {
        poly(g, pen, &[(0.20, 0.12), (0.04, 0.16), (-0.46, 0.10), (-0.56, 0.04), (0.10, 0.06)], mix(c, c2, 0.5), 0.95);
        if k == 0 {
            for i in 0..4 {
                circ(g, pen, -0.10 - i as f32 * 0.1, 0.10, 0.025, mul(c2, 0.6), 0.9);
            }
        }
    } else {
        poly(g, pen, &[(0.20, 0.12), (-0.40, 0.10), (-0.42, 0.0), (0.10, 0.04)], mul(c2, 0.8), 0.9);
        seg(g, pen, -0.46, 0.0, -0.60, -0.04, 0.012, dark, 1.0);
        seg(g, pen, -0.46, 0.0, -0.60, 0.04, 0.012, dark, 1.0);
    }
    // 胸
    ell(g, pen, 0.22, 0.04, 0.12, 0.11, 0.0, c, 1.0);
    // 頭
    if pointed {
        poly(g, pen, &[(0.30, 0.10), (0.56, 0.10), (0.34, -0.06)], c, 1.0);
        circ(g, pen, 0.42, 0.07, 0.03, mix(c2, INK, 0.5), 1.0);
        poly(g, pen, &[(0.56, 0.10), (0.68, 0.14), (0.58, 0.08)], c, 1.0);
    } else {
        ell(g, pen, 0.36, 0.02, 0.08, 0.10, -0.3, c, 1.0);
        circ(g, pen, 0.38, 0.06, 0.03, mix(c2, INK, 0.5), 1.0);
    }
    // 大きな後ろ脚（跳ぶとのびる）
    let jump = (1.0 - activity).clamp(0.0, 1.0);
    let knee = (-0.10 - (1.0 - jump) * 0.2, 0.22 * jump + 0.02);
    for side in [0.7f32, 1.0] {
        let col = mul(c, side);
        path(g, pen, &[(0.10, 0.0), knee], 0.06, col, 1.0);
        path(g, pen, &[knee, (-0.30 - (1.0 - jump) * 0.3, -0.34 * jump - 0.04)], 0.025, mul(col, 0.85), 1.0);
    }
    let alen = match k {
        2 => 0.9,
        3 => 0.7,
        1 => 0.12,
        _ => 0.18,
    };
    antenna(g, pen, if pointed { 0.62 } else { 0.42 }, 0.08, alen, if k >= 2 { 0.9 } else { 0.5 }, if k >= 2 { -1.6 } else { 0.0 }, 0.012, dark, t);
}

fn mantis(g: &mut Gfx, pen: &Pen, k: u8, c: [f32; 3], c2: [f32; 3], t: f32, activity: f32) {
    let wide = k == 1;
    let sway = (t * 1.2).sin() * 0.02;
    // 中脚・後脚
    for (u, d) in [(0.0, 0.06), (-0.08, -0.12)] {
        for side in [0.7f32, 1.0] {
            let w = (t * 6.0 + u * 10.0).sin() * activity * 0.05;
            path(g, pen, &[(u, 0.0), (u + d + w, 0.12), (u + d * 2.0 + w, -0.36)], 0.018, mul(c, side), 1.0);
        }
    }
    // おなか
    ell(g, pen, -0.26, 0.04, 0.30, if wide { 0.14 } else { 0.09 }, 0.0, c, 1.0);
    poly(g, pen, &[(0.0, 0.10), (-0.50, 0.12), (-0.52, 0.04), (0.0, 0.02)], mix(c, c2, 0.3), 0.85);
    if wide {
        circ(g, pen, -0.10, 0.10, 0.03, c2, 1.0);
    }
    // 長い胸（立ち上がる）
    seg(g, pen, 0.0, 0.04, 0.26, 0.30 + sway, 0.05, c, 1.0);
    // 三角の頭と大きな目
    let (hu, hv) = (0.30, 0.34 + sway);
    poly(g, pen, &[(hu - 0.05, hv + 0.06), (hu + 0.08, hv + 0.05), (hu + 0.04, hv - 0.06)], c, 1.0);
    circ(g, pen, hu + 0.04, hv + 0.05, 0.028, mix(c, c2, 0.4), 1.0);
    antenna(g, pen, hu + 0.04, hv + 0.06, 0.24, 1.2, 0.3, 0.008, mul(c, 0.7), t);
    // カマ（折りたたんだ前脚）
    let strike = (t * 0.6).sin().max(0.9) - 0.9;
    path(g, pen, &[(0.22, 0.24), (0.36 + strike, 0.10), (0.26, 0.20)], 0.035, mul(c, 1.05), 1.0);
    for i in 0..3 {
        let f = i as f32 / 3.0;
        seg(g, pen, 0.22 + 0.14 * f, 0.24 - 0.14 * f, 0.22 + 0.14 * f, 0.20 - 0.14 * f, 0.008, INK, 0.6);
    }
}

fn stick(g: &mut Gfx, pen: &Pen, c: [f32; 3], c2: [f32; 3], t: f32, activity: f32) {
    let sway = (t * 0.9).sin() * 0.02;
    for (u, d) in [(0.30, 0.18), (0.06, 0.06), (-0.14, -0.14)] {
        for side in [0.75f32, 1.0] {
            let w = (t * 4.0 + u * 7.0).sin() * activity * 0.04;
            path(g, pen, &[(u, 0.0), (u + d * 0.5 + w, 0.08), (u + d + w, -0.22)], 0.012, mul(mix(c, c2, 0.4), side), 1.0);
        }
    }
    seg(g, pen, -0.70, sway, 0.42, 0.0, 0.035, c, 1.0);
    for i in 0..6 {
        let u = 0.30 - i as f32 * 0.17;
        circ(g, pen, u, sway * (1.0 - u), 0.022, mix(c, c2, 0.3), 1.0);
    }
    circ(g, pen, 0.44, 0.0, 0.03, c, 1.0);
    antenna(g, pen, 0.46, 0.0, 0.32, 0.1, 0.1, 0.008, c, t);
}

// ---------------------------------------------------------------- ハチ・アリ・ハサミムシ

fn bee(g: &mut Gfx, pen: &Pen, k: u8, c: [f32; 3], c2: [f32; 3], t: f32) {
    let f = flap(t, 30.0);
    // はね（すき通る・速い）
    for (du, a) in [(0.0, 0.25), (-0.06, 0.35)] {
        let ang = 0.3 + 1.1 * f;
        let tip = (0.10 + du - 0.40 * (1.0 - ang.sin() * 0.4), 0.10 + ang.sin() * 0.42);
        let mid = ((0.12 + du + tip.0) * 0.5, (0.10 + tip.1) * 0.5);
        ell(g, pen, mid.0, mid.1, 0.24, 0.07, (tip.1 - 0.10).atan2(tip.0 - 0.12 - du), [0.85, 0.90, 0.95], a + 0.2);
    }
    legs(g, pen, [0.18, 0.10, 0.02], -0.06, -0.30, 0.08, t, 0.0, 0.02, mul(c2, 1.2));
    match k {
        2 => {
            // クマバチ：丸い黒いおなか・黄色い胸
            ell(g, pen, -0.16, -0.02, 0.26, 0.20, 0.0, c, 1.0);
            ell(g, pen, 0.14, 0.04, 0.16, 0.17, 0.0, c2, 1.0);
            circ(g, pen, 0.34, 0.02, 0.10, c, 1.0);
        }
        _ => {
            // しまのおなか
            ell(g, pen, -0.20, -0.02, 0.26, 0.14, -0.15, c, 1.0);
            for i in 0..3 {
                let u = -0.10 - i as f32 * 0.10;
                ell(g, pen, u, -0.02 + i as f32 * 0.01, 0.025, 0.14 - i as f32 * 0.02, 0.0, c2, 1.0);
            }
            if k == 1 {
                poly(g, pen, &[(-0.44, -0.06), (-0.52, -0.10), (-0.44, -0.10)], c2, 1.0);
            }
            ell(g, pen, 0.12, 0.04, 0.13, 0.12, 0.0, if k == 1 { mix(c2, c, 0.25) } else { mix(c, c2, 0.5) }, 1.0);
            let head = if k == 1 { c } else { c2 };
            circ(g, pen, 0.30, 0.02, if k == 1 { 0.11 } else { 0.08 }, head, 1.0);
            circ(g, pen, 0.33, 0.05, 0.035, INK, 1.0);
        }
    }
    antenna(g, pen, 0.36, 0.08, 0.16, 1.2, -0.6, 0.015, INK, t);
}

fn ant(g: &mut Gfx, pen: &Pen, c: [f32; 3], c2: [f32; 3], t: f32, activity: f32) {
    legs(g, pen, [0.10, 0.04, -0.02], 0.0, -0.30, 0.22, t, activity, 0.02, c2);
    ell(g, pen, -0.28, 0.04, 0.18, 0.14, 0.15, c, 1.0);
    circ(g, pen, -0.07, 0.04, 0.04, c, 1.0);
    ell(g, pen, 0.06, 0.06, 0.12, 0.07, 0.2, c, 1.0);
    ell(g, pen, 0.28, 0.08, 0.11, 0.10, 0.0, c, 1.0);
    ell(g, pen, -0.30, 0.10, 0.08, 0.03, 0.0, WHITE, 0.25);
    // ひじのように曲がった触角
    let w = (t * 4.0).sin() * 0.03;
    path(g, pen, &[(0.34, 0.14), (0.40, 0.32), (0.56 + w, 0.22)], 0.014, c2, 1.0);
    path(g, pen, &[(0.34, 0.12), (0.44, 0.28), (0.58 - w, 0.16)], 0.012, mul(c2, 0.7), 1.0);
}

fn earwig(g: &mut Gfx, pen: &Pen, c: [f32; 3], c2: [f32; 3], t: f32, activity: f32) {
    legs(g, pen, [0.22, 0.10, -0.02], -0.02, -0.20, 0.1, t, activity, 0.018, c2);
    for i in 0..7 {
        let u = 0.10 - i as f32 * 0.08;
        ell(g, pen, u, 0.02, 0.05, 0.08 - i as f32 * 0.003, 0.0, if i % 2 == 0 { c } else { mul(c, 1.2) }, 1.0);
    }
    // おしりのハサミ
    path(g, pen, &[(-0.44, 0.02), (-0.60, 0.08), (-0.66, 0.02)], 0.03, mix(c, c2, 0.4), 1.0);
    path(g, pen, &[(-0.44, 0.0), (-0.60, -0.06), (-0.66, 0.0)], 0.025, mix(c, c2, 0.3), 1.0);
    ell(g, pen, 0.22, 0.03, 0.08, 0.07, 0.0, c, 1.0);
    circ(g, pen, 0.34, 0.03, 0.06, mul(c, 0.8), 1.0);
    antenna(g, pen, 0.38, 0.05, 0.24, 0.6, -0.3, 0.012, c2, t);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 冬越しの姿の絵も、有限で枠に収まる
    #[test]
    fn stage_art_fits() {
        let mut g = Gfx::new(800, 600);
        for sp in 0..SPECIES.len() {
            let Some((stage, _)) = SPECIES[sp].winter() else { continue };
            g.verts.clear();
            draw_stage(&mut g, sp, stage, 0.0, 0.0, 1.0, 1.0, 0.0, 0.37, 1.0, 3);
            assert!(!g.verts.is_empty());
            let (w, h) = stage_extent(sp, stage);
            for v in &g.verts {
                assert!(v.x.is_finite() && v.y.is_finite());
                assert!(v.x.abs() <= w * 0.5 + 0.12 && v.y.abs() <= h * 0.5 + 0.12, "{} {:?} at {},{} outside {}x{}", SPECIES[sp].slug, stage, v.x, v.y, w, h);
            }
        }
    }

    /// 手描きの絵が有限で、図鑑の枠（extent）にだいたい収まる
    #[test]
    fn art_is_finite_and_fits_extent() {
        let mut g = Gfx::new(800, 600);
        for sp in 0..SPECIES.len() {
            for act in [0.0, 1.0] {
                g.verts.clear();
                draw(&mut g, sp, 0.0, 0.0, 1.0, 1.0, 0.0, 0.37, 1.0, act, 3);
                assert!(!g.verts.is_empty());
                let (w, h) = extent(sp);
                for v in &g.verts {
                    assert!(v.x.is_finite() && v.y.is_finite());
                    assert!(v.x.abs() <= w * 0.5 + 0.12 && v.y.abs() <= h * 0.5 + 0.12, "{} at {},{} outside {}x{}", SPECIES[sp].slug, v.x, v.y, w, h);
                }
            }
        }
    }
}
