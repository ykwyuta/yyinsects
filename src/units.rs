//! 観察の場面のスプライト（画像があれば使う。無ければ art.rs の手描き）。両方の版で共通。
use crate::art;
use crate::artwork::{find_unit, UNITS};
use crate::gfx::{Gfx, Texture, Vertex};
use crate::life::SPECIES;

pub struct Units {
    textures: Vec<Option<Texture>>,
    attempted: Vec<bool>,
    mesh: Vec<Vertex>,
    /// 虫ごとのスプライトの番号（無いものは手描き）
    species: Vec<Option<usize>>,
    capsule: Option<usize>,
    rock: Option<usize>,
    sand: Option<usize>,
}

#[derive(Clone, Copy)]
pub struct Pose {
    pub x: f32,
    pub y: f32,
    pub len: f32,
    pub facing: f32,
    pub pitch: f32,
    pub time: f32,
    pub activity: f32,
}

/// 虫の絵の四角（描画・画面外の判定・写真の採点で共通）。(x, y) は絵の中心、vertical は幹に縦にとまっているとき
pub fn bounds(sp: usize, x: f32, y: f32, len: f32, vertical: bool) -> [f32; 4] {
    let (ew, eh) = art::extent(sp);
    let (w, h) = match find_unit(SPECIES[sp].slug) {
        Some(i) => {
            let (_, _, iw, ih) = UNITS[i];
            let w = len * ew;
            (w, (w * ih as f32 / iw as f32).min(len * eh * 1.6))
        }
        None => (len * ew, len * eh),
    };
    let (w, h) = if vertical { (h, w) } else { (w, h) };
    [x - w * 0.5, y - h * 0.5, x + w * 0.5, y + h * 0.5]
}

impl Units {
    pub fn new() -> Self {
        Self {
            textures: (0..UNITS.len()).map(|_| None).collect(),
            attempted: vec![false; UNITS.len()],
            mesh: Vec::with_capacity(12 * 4 * 6),
            species: SPECIES.iter().map(|s| find_unit(s.slug)).collect(),
            capsule: find_unit("capsule"),
            rock: find_unit("rock"),
            sand: find_unit("seabed-sand"),
        }
    }

    fn load(&mut self, g: &Gfx, index: usize) {
        if self.attempted[index] {
            return;
        }
        self.attempted[index] = true;
        if let Ok(image) = image::load_from_memory_with_format(UNITS[index].1, image::ImageFormat::Png) {
            let rgba = image.to_rgba8();
            let pixels: Vec<u32> = rgba.as_raw().chunks_exact(4).map(|p| u32::from_le_bytes([p[0], p[1], p[2], p[3]])).collect();
            self.textures[index] = g.create_texture(rgba.width(), rgba.height(), &pixels).ok();
        }
    }

    /// 虫のスプライトを描く。スプライトが無ければ false（呼ぶ側が手描きで描く）
    pub fn creature(&mut self, g: &mut Gfx, sp: usize, pose: Pose) -> bool {
        let Some(index) = self.species[sp] else { return false };
        self.load(g, index);
        let Some(texture) = &self.textures[index] else { return false };
        let [x0, y0, x1, y1] = bounds(sp, pose.x, pose.y, pose.len, false);
        let (w, h) = (x1 - x0, y1 - y0);
        let (sn, cs) = (-pose.pitch).sin_cos();
        let flies = SPECIES[sp].art.flies() && pose.activity > 0.05;
        let flap = (pose.time * 20.0).sin();
        let vertex = |u: f32, v: f32| {
            // 飛ぶ虫は、はね（上半分）を上下にたたむ
            let v2 = if flies && v < 0.5 { 0.5 - (0.5 - v) * (0.55 + 0.45 * flap.abs()) } else { v };
            let (lx, ly) = ((u - 0.5) * w, (v2 - 0.5) * h);
            Vertex { x: pose.x + (lx * cs - ly * sn) * pose.facing, y: (y0 + y1) * 0.5 + lx * sn + ly * cs, u, v, c: [1.0, 1.0, 1.0, 1.0] }
        };
        self.mesh.clear();
        for row in 0..4 {
            for col in 0..12 {
                let (u0, u1) = (col as f32 / 12.0, (col + 1) as f32 / 12.0);
                let (v0, v1) = (row as f32 / 4.0, (row + 1) as f32 / 4.0);
                self.mesh.extend_from_slice(&[vertex(u0, v0), vertex(u1, v0), vertex(u1, v1), vertex(u0, v0), vertex(u1, v1), vertex(u0, v1)]);
            }
        }
        g.draw_textured_mesh(texture, &self.mesh);
        true
    }

    pub fn capsule(&mut self, g: &mut Gfx, x: f32, y: f32, r: f32, facing: f32, tilt: f32) -> bool {
        let Some(index) = self.capsule else { return false };
        self.load(g, index);
        let Some(texture) = &self.textures[index] else { return false };
        let w = r * 3.4;
        let h = w * texture.h as f32 / texture.w as f32;
        let (sn, cs) = tilt.sin_cos();
        let p = |u: f32, v: f32| (x + (u * cs - v * sn) * facing, y + u * sn + v * cs);
        g.draw_textured_quad(texture, [p(-w * 0.5, -h * 0.5), p(w * 0.5, -h * 0.5), p(w * 0.5, h * 0.5), p(-w * 0.5, h * 0.5)], [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)], [1.0, 1.0, 1.0, 1.0], true);
        true
    }

    pub fn rock(&mut self, g: &mut Gfx, x: f32, y: f32, r: f32, tone: f32) -> bool {
        let Some(index) = self.rock else { return false };
        self.load(g, index);
        let Some(texture) = &self.textures[index] else { return false };
        let w = r * 2.6;
        let h = w * texture.h as f32 / texture.w as f32;
        let tint = 0.78 + tone * 0.22;
        let uv = if tone > 0.5 { [(1.0, 0.0), (0.0, 0.0), (0.0, 1.0), (1.0, 1.0)] } else { [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)] };
        g.draw_textured_quad(texture, [(x - w * 0.5, y - h), (x + w * 0.5, y - h), (x + w * 0.5, y), (x - w * 0.5, y)], uv, [tint, tint, tint, 1.0], true);
        true
    }

    pub fn ground(&mut self, g: &mut Gfx, mesh: &[Vertex]) -> bool {
        let Some(index) = self.sand else { return false };
        self.load(g, index);
        let Some(texture) = &self.textures[index] else { return false };
        g.draw_textured_mesh(texture, mesh);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn assets_are_real_alpha_sprites() {
        for &(slug, bytes, w, h) in UNITS.iter() {
            let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png).unwrap().to_rgba8();
            assert_eq!(image.dimensions(), (w, h), "{}", slug);
        }
        for slug in ["capsule", "rock", "seabed-sand"] {
            assert!(find_unit(slug).is_some());
        }
    }
}
