//! 進み具合（図鑑の収録・宿題・時と場所）と、その記録の読み書き
use crate::life::n_entries;
use crate::mission::{mission_done, MISSIONS};
use crate::save::{RResult, Reader, Writer};

const MAGIC: &[u8; 4] = b"YYI1";
/// 2：いまの月を記録する版
const VERSION: u32 = 2;

#[derive(Clone)]
pub struct Progress {
    /// 図鑑の項目ごとの、収録済みか（1 が収録済み）
    pub stars: Vec<u8>,
    /// まだ図鑑で見ていない新しい項目
    pub fresh: Vec<bool>,
    /// 選んでいる宿題（どの順番でも取り組める）
    pub mission: usize,
    /// カプセルの場所（経緯度）
    pub lon: f32,
    pub lat: f32,
    /// いまの月（1〜12）
    pub month: u8,
    pub photos: u32,
    pub intro_done: bool,
    pub light: bool,
}

impl Progress {
    pub fn new() -> Progress {
        let s = &crate::earth::SPOTS[crate::earth::S_SHOWA];
        Progress { stars: vec![0; n_entries()], fresh: vec![false; n_entries()], mission: 0, lon: s.lon, lat: s.lat, month: 7, photos: 0, intro_done: false, light: false }
    }

    pub fn found(&self) -> usize {
        self.stars.iter().filter(|&&s| s > 0).count()
    }

    /// 写真を図鑑に収録する。(新しい項目か, 記録が変わったか)
    pub fn record(&mut self, entry: usize, stars: u8) -> (bool, bool) {
        let old = self.stars[entry];
        let stars = stars.min(1);
        if stars > old {
            self.stars[entry] = stars;
            if old == 0 {
                self.fresh[entry] = true;
            }
            (old == 0, true)
        } else {
            (false, false)
        }
    }

    /// その宿題に合格しているか（収録は消えないので、一度合格すればずっと合格）
    pub fn is_done(&self, i: usize) -> bool {
        mission_done(&MISSIONS[i], &self.stars)
    }

    /// 宿題ごとの合格（写真を撮る前に取っておき、`advance` に渡す）
    pub fn done_flags(&self) -> Vec<bool> {
        (0..MISSIONS.len()).map(|i| self.is_done(i)).collect()
    }

    pub fn done_count(&self) -> usize {
        (0..MISSIONS.len()).filter(|&i| self.is_done(i)).count()
    }

    /// 写真のあとで呼ぶ。新しく合格した宿題（どの順番でも）を返す。
    /// 選んでいた宿題が合格したら、次のまだ合格していない宿題を選ぶ
    pub fn advance(&mut self, before: &[bool]) -> Vec<usize> {
        let done: Vec<usize> = (0..MISSIONS.len()).filter(|&i| !before.get(i).copied().unwrap_or(false) && self.is_done(i)).collect();
        if self.is_done(self.mission) {
            self.select_next(1);
        }
        done
    }

    /// まだ合格していない宿題を、いまの宿題から d の向き（1 次、-1 前）に探して選ぶ。無ければそのまま
    pub fn select_next(&mut self, d: i32) {
        let n = MISSIONS.len() as i32;
        for k in 1..=n {
            let i = (self.mission as i32 + d.signum() * k).rem_euclid(n) as usize;
            if !self.is_done(i) {
                self.mission = i;
                return;
            }
        }
    }

    pub fn all_clear(&self) -> bool {
        self.done_count() >= MISSIONS.len()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.bytes(MAGIC);
        w.u32(VERSION);
        // 虫は slug で書く（あとで虫を足しても並びがずれない）
        let found: Vec<usize> = (0..self.stars.len()).filter(|&i| self.stars[i] > 0).collect();
        w.usize(found.len());
        for i in found {
            let slug = crate::life::SPECIES[i].slug.as_bytes();
            w.usize(slug.len());
            w.bytes(slug);
            w.bool(self.fresh[i]);
        }
        w.usize(self.mission);
        w.f32(self.lon);
        w.f32(self.lat);
        w.u8(self.month);
        w.u32(self.photos);
        w.bool(self.intro_done);
        w.bool(self.light);
        w.buf
    }

    pub fn from_bytes(data: &[u8]) -> RResult<Progress> {
        let mut r = Reader::new(data);
        if r.bytes(4)? != MAGIC {
            return Err(t!("昆虫図鑑の記録ではない", "This is not a YYInsects save", "这不是昆虫图鉴的存档").into());
        }
        let v = r.u32()?;
        if v > VERSION {
            return Err(t!("新しい版で作られた記録で、読み込めない", "This save was made by a newer version", "此存档由更新版本创建，无法读取").into());
        }
        let mut p = Progress::new();
        let n = r.len(4096)?;
        for _ in 0..n {
            let len = r.len(256)?;
            let slug = String::from_utf8_lossy(r.bytes(len)?).to_string();
            let f = r.bool()?;
            if let Some(j) = crate::life::SPECIES.iter().position(|s| s.slug == slug) {
                p.stars[j] = 1;
                p.fresh[j] = f;
            }
        }
        p.mission = r.usize()?.min(MISSIONS.len() - 1);
        p.lon = r.f32()?.clamp(-180.0, 180.0);
        p.lat = r.f32()?.clamp(-80.0, 80.0);
        if v >= 2 {
            p.month = r.u8()?.clamp(1, 12);
        }
        p.photos = r.u32()?;
        p.intro_done = r.bool()?;
        p.light = r.bool()?;
        if p.is_done(p.mission) {
            p.select_next(1);
        }
        if !p.lon.is_finite() || !p.lat.is_finite() {
            return Err(t!("記録が壊れている", "The save is damaged", "存档已损坏").into());
        }
        Ok(p)
    }

    /// 記録の見出し
    pub fn meta(&self) -> String {
        let m = if self.all_clear() { t!("全課題クリア", "All clear", "全部完成").to_string() } else { tf!("宿題 {}/{}合格", "Tasks {}/{}", "作业 {}/{}合格", self.done_count(), MISSIONS.len()) };
        tf!("{}月・{}・図鑑 {}/{}", "Month {} · {} · Guide {}/{}", "{}月·{}·图鉴 {}/{}", self.month, m, self.found(), n_entries())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let mut p = Progress::new();
        p.record(3, 1);
        p.record(n_entries() - 1, 1);
        p.lon = 139.5;
        p.lat = 35.25;
        p.month = 12;
        p.photos = 42;
        p.intro_done = true;
        let b = p.to_bytes();
        let q = Progress::from_bytes(&b).unwrap();
        assert_eq!(q.stars, p.stars);
        assert_eq!(q.fresh, p.fresh);
        assert_eq!(q.lon, 139.5);
        assert_eq!(q.month, 12);
        assert_eq!(q.photos, 42);
        assert!(q.intro_done);
        assert!(Progress::from_bytes(&b[..b.len() - 3]).is_err());
        assert!(Progress::from_bytes(b"NOPE").is_err());
    }
}
