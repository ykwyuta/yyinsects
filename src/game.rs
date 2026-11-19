//! 画面と操作：タイトル、地図（上空）、観察（降りる）、図鑑、宿題、記録（両方の版で共通）
//!
//! 地図は日本を中心にしたメルカトル図法。関東地方の観察スポットを選んでカプセルで飛び、
//! その場所の林・草はら・水辺などに降りて虫を撮影し、よく写っていれば図鑑に収録する。
//! 関東の外を選ぶと「準備中（待っててね）」と知らせる。

use crate::art;
use crate::artwork::{Artwork, Fit};
use crate::dive::*;
use crate::earth::*;
use crate::gfx::{Font, Gfx, Texture};
use crate::lang::{self, L};
use crate::life::*;
use crate::mission::*;
use crate::platform;
use crate::progress::Progress;
use crate::Ev;

const VK_BACK: u32 = 0x08;
const VK_RETURN: u32 = 0x0d;
const VK_SHIFT: u32 = 0x10;
const VK_CONTROL: u32 = 0x11;
const VK_ESCAPE: u32 = 0x1b;
const VK_SPACE: u32 = 0x20;
const VK_LEFT: u32 = 0x25;
const VK_UP: u32 = 0x26;
const VK_RIGHT: u32 = 0x27;
const VK_DOWN: u32 = 0x28;
const VK_F1: u32 = 0x70;
const VK_F2: u32 = 0x71;
const VK_F3: u32 = 0x72;
const VK_F11: u32 = 0x7a;
const VK_PLUS: u32 = 0xbb;
const VK_MINUS: u32 = 0xbd;
const VK_ADD: u32 = 0x6b;
const VK_SUB: u32 = 0x6d;
const VK_COMMA: u32 = 0xbc;
const VK_PERIOD: u32 = 0xbe;

// 色
const BG: [f32; 4] = [0.04, 0.11, 0.08, 0.90];
const BG2: [f32; 4] = [0.07, 0.18, 0.12, 0.94];
const EDGE: [f32; 4] = [0.55, 0.85, 0.50, 0.55];
const TEXT: [f32; 4] = [0.95, 0.98, 0.93, 1.0];
const DIM: [f32; 4] = [0.70, 0.80, 0.70, 1.0];
const ACCENT: [f32; 4] = [0.98, 0.70, 0.30, 1.0];
const GOLD: [f32; 4] = [1.0, 0.84, 0.32, 1.0];
const GOOD: [f32; 4] = [0.55, 0.92, 0.62, 1.0];

/// 地図の拡大の上限
const MAX_ZOOM: f32 = 80.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Screen {
    Title,
    Map,
    Dive,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Panel {
    None,
    Guide,
    Entry(usize),
    Missions,
    Help,
    Intro(u8),
    Done(usize),
    /// 関東の外：準備中（待っててね）
    Soon,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Act {
    Voice,
    ReplayVoice,
    /// キャッシュを消して再読み込み（ブラウザ版）
    Reload,
    NewGame,
    Continue,
    Import,
    Export,
    Lang,
    Fullscreen,
    /// いるスポットのその場面に降りる
    Dive(Habitat),
    Surface,
    Shoot,
    Guide,
    Missions,
    Help,
    Close,
    Entry(usize),
    Goto(Target),
    /// 前・次の観察スポットへ
    StepSpot(i32),
    /// その月へ（季節を変える）
    Month(u8),
    /// 前・次の月へ
    StepMonth(i32),
    /// その時間帯へ（0=早朝 1=昼 2=夕方 3=夜）
    TimeOfDay(u8),
    /// 次・前の時間帯へ
    StepTime(i32),
    /// 関東の全体を見る
    ShowKanto,
    EntryPage(i32),
    /// 宿題を選ぶ（どの順番でも）
    SelectMission(usize),
    /// まだ合格していない次・前の宿題へ
    CycleMission(i32),
    /// レーダーで図鑑の項目を探す
    Radar(usize),
    RadarOff,
    /// レーダーの光る場所のうち、いちばん近いところへ
    RadarGo,
    GuideTab(usize),
    Light,
    Lens(i32),
    /// 地図の拡大・縮小
    MapZoom(i32),
    ToTitle,
    IntroNext,
    NextPhoto,
    MissionCard,
    Boost,
}

/// 計測結果は画面サイズと言語が変わるまで再利用する。
struct EntryBook {
    key: (usize, lang::Lang, u32, u32),
    pages: Vec<(String, String)>,
}

/// UTF-8 の文字境界で区切り、各ページの全文が表示領域に収まるようにする。
fn reading_pages(gfx: &mut Gfx, sections: &[crate::guide::Section], width: f32, height: f32) -> Vec<(String, String)> {
    let mut pages = Vec::new();
    for section in sections {
        let mut rest = section.body.as_str();
        while !rest.is_empty() {
            let mut boundaries: Vec<usize> = rest.char_indices().map(|(i, _)| i).collect();
            boundaries.push(rest.len());
            let (mut low, mut high) = (0, boundaries.len() - 1);
            while low < high {
                let mid = (low + high + 1) / 2;
                if gfx.measure_h(&rest[..boundaries[mid]], Font::Normal, width) <= height {
                    low = mid;
                } else {
                    high = mid - 1;
                }
            }
            // 通常は複数行が入る。極小ウィンドウでも必ず前へ進む。
            let mut end = boundaries[low.max(1)];
            if end < rest.len() {
                // ページ末尾付近の文末を優先し、できるだけ文を途中で切らない。
                if let Some((i, c)) = rest[..end].char_indices().rev().find(|(i, c)| *i >= end / 2 && matches!(c, '。' | '！' | '？' | '\n')) {
                    end = i + c.len_utf8();
                }
            }
            pages.push((section.title.to_string(), rest[..end].to_string()));
            rest = &rest[end..];
        }
    }
    pages
}

struct Button {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    act: Act,
    label: String,
    enabled: bool,
}

struct PhotoCard {
    /// (図鑑の項目, 収録したか（1）, 新しい, 記録が変わった)
    lines: Vec<(usize, u8, bool, bool)>,
    /// 写りが悪かったときの助言
    advice: Option<&'static str>,
    left: f32,
    age: f32,
    index: usize,
    done_missions: Vec<usize>,
}

pub struct Game {
    gfx: Gfx,
    earth: Earth,
    prog: Progress,
    screen: Screen,
    panel: Panel,
    ocean_tex: Texture,
    cloud_tex: Texture,
    /// 世界・日本・関東の地図の絵
    map_tex: Vec<(Texture, [f32; 4])>,
    artwork: Artwork,
    units: crate::units::Units,
    // 地図
    cam: (f32, f32),
    zoom: f32,
    follow: bool,
    cap: (f32, f32),
    cap_target: Option<(f32, f32)>,
    /// 着いたら降りる場面（「行く」で向かうとき）
    pending_hab: Option<Habitat>,
    /// 準備中の地域の名前
    soon_name: String,
    /// いちばん新しい「〇月になった」の知らせ
    month_notice: String,
    // 観察
    scene: Option<Scene>,
    lens: f32,
    dcam: (f32, f32),
    steer: Option<(f32, f32)>,
    dive_goto: Option<(f32, f32)>,
    boost_btn: bool,
    // 入力
    mouse: (f32, f32),
    keys: [bool; 256],
    ldown: Option<(f32, f32, bool)>,
    drag: Option<(f32, f32, (f32, f32))>,
    touch: bool,
    buttons: Vec<Button>,
    // 表示
    notices: Vec<(String, f32)>,
    photo: Option<PhotoCard>,
    flash: f32,
    clock: f32,
    guide_scroll: f32,
    guide_max: f32,
    entry_page: usize,
    entry_book: Option<EntryBook>,
    /// 図鑑で開いているなかま（タブ）
    guide_group: usize,
    /// レーダーで探している図鑑の項目
    radar: Option<usize>,
    /// 会えるスポットの場所（経緯度）
    radar_cells: Vec<(f32, f32)>,
    radar_key: Option<usize>,
    mission_open: bool,
    have_save: bool,
    confirm_new: f32,
    /// 「キャッシュを消して再読み込み」の確認（もう一度押すまでの残り秒）
    confirm_reload: f32,
    save_cool: f32,
    save_dirty: bool,
    title_t: f32,
    tip: Option<String>,
    voice: bool,
    spoken: String,
    dialog_voice: String,
    hover_voice: Option<(String, f32)>,
    voice_mouse: (f32, f32),
    seed: u32,
}

fn mix4(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t, a[3] + (b[3] - a[3]) * t]
}

/// なかまのタブ（小さな画面でも読めるように折り返す）
fn group_tabs(width: f32) -> (usize, usize, f32) {
    let cols = ((width / 150.0).floor() as usize).clamp(1, GROUPS.len());
    (cols, GROUPS.len().div_ceil(cols), width / cols as f32)
}

fn entry_name(e: usize) -> &'static str {
    SPECIES[e].name.get()
}

fn size_text(cm: f32) -> String {
    if cm < 1.0 {
        tf!("約{:.0}mm", "about {:.0} mm", "约{:.0}毫米", cm * 10.0)
    } else {
        tf!("約{:.1}cm", "about {:.1} cm", "约{:.1}厘米", cm)
    }
}

/// 観察の場面の遠景の画像（assets/artwork/backdrop-<場面>.jpg があれば）
fn backdrop_slug(sc: &Scene) -> Option<&'static str> {
    let slug = match sc.hab {
        Habitat::Forest => "backdrop-forest",
        Habitat::Mountain => "backdrop-mountain",
        Habitat::Meadow => "backdrop-meadow",
        Habitat::Park => "backdrop-park",
        Habitat::Pond => "backdrop-pond",
        Habitat::River => "backdrop-river",
        Habitat::Paddy => "backdrop-paddy",
        Habitat::Shore => "backdrop-shore",
    };
    crate::artwork::find(slug).map(|_| slug)
}

/// 降りるボタンの言葉
fn dive_label(h: Habitat) -> String {
    if h.is_water() {
        tf!("{}へ降りる", "Land at the {}", "降落到{}", h.name())
    } else {
        tf!("{}へ降りる", "Land in the {}", "降落到{}", h.name())
    }
}

impl Game {
    pub fn new(gfx: Gfx) -> Game {
        let earth = Earth::new();
        let (ow, oh, opx) = ocean_texture();
        let ocean_tex = gfx.create_texture(ow, oh, &opx).expect("texture");
        let (cw, ch, cpx) = cloud_texture();
        let cloud_tex = gfx.create_texture(cw, ch, &cpx).expect("texture");
        let map_tex = vec![
            (gfx.create_texture(earth.world.w, earth.world.h, &earth.world_pixels()).expect("texture"), earth.world.rect),
            (gfx.create_texture(earth.japan.w, earth.japan.h, &earth.japan_pixels()).expect("texture"), earth.japan.rect),
            (gfx.create_texture(earth.kanto.w, earth.kanto.h, &earth.kanto_pixels()).expect("texture"), earth.kanto.rect),
        ];
        let prog = Progress::new();
        let cap = lonlat_to_map(prog.lon, prog.lat);
        let mut g = Game {
            gfx,
            earth,
            prog,
            screen: Screen::Title,
            panel: Panel::None,
            ocean_tex,
            cloud_tex,
            map_tex,
            artwork: Artwork::new(),
            units: crate::units::Units::new(),
            cam: (MAP_W * 0.5, MAP_H * 0.5),
            zoom: 0.4,
            follow: true,
            cap,
            cap_target: None,
            pending_hab: None,
            soon_name: String::new(),
            month_notice: String::new(),
            scene: None,
            lens: 1.0,
            dcam: (0.0, 0.0),
            steer: None,
            dive_goto: None,
            boost_btn: false,
            mouse: (-100.0, -100.0),
            keys: [false; 256],
            ldown: None,
            drag: None,
            touch: false,
            buttons: Vec::new(),
            notices: Vec::new(),
            photo: None,
            flash: 0.0,
            clock: 0.0,
            guide_scroll: 0.0,
            guide_max: 0.0,
            entry_page: 0,
            entry_book: None,
            guide_group: 0,
            radar: None,
            radar_cells: Vec::new(),
            radar_key: None,
            mission_open: true,
            have_save: platform::slot_info(0).is_some(),
            confirm_new: 0.0,
            confirm_reload: 0.0,
            save_cool: 0.0,
            save_dirty: false,
            title_t: 0.0,
            tip: None,
            voice: platform::voice_enabled(),
            spoken: String::new(),
            dialog_voice: String::new(),
            hover_voice: None,
            voice_mouse: (0.0, 0.0),
            seed: platform::random_u32(),
        };
        g.fit_world();
        g
    }

    fn w(&self) -> f32 {
        self.gfx.width as f32
    }
    fn h(&self) -> f32 {
        self.gfx.height as f32
    }
    fn compact(&self) -> bool {
        self.gfx.compact
    }
    fn top_h(&self) -> f32 {
        if self.compact() {
            40.0
        } else {
            46.0
        }
    }

    fn min_zoom(&self) -> f32 {
        (self.w() / MAP_W).min(self.h() / MAP_H) * 0.98
    }

    /// 関東地方が画面に収まる拡大
    fn kanto_zoom(&self) -> f32 {
        let r = self.earth.kanto.rect;
        ((self.w() - 40.0) / (r[2] - r[0])).min((self.h() - self.top_h() - 40.0) / (r[3] - r[1])).max(self.min_zoom())
    }

    /// 関東の全体を見る
    fn show_kanto(&mut self) {
        let r = self.earth.kanto.rect;
        self.zoom = self.kanto_zoom();
        self.cam = ((r[0] + r[2]) * 0.5, (r[1] + r[3]) * 0.5);
        self.follow = false;
        self.clamp_cam();
    }

    /// カプセルがいる観察スポット（着いていれば）
    fn at_spot(&self) -> Option<usize> {
        if self.cap_target.is_some() {
            return None;
        }
        let (lon, lat) = map_to_lonlat(self.cap.0, self.cap.1);
        SPOTS.iter().position(|s| km_dist((s.lon, s.lat), (lon, lat)) < 0.5)
    }

    fn fit_world(&mut self) {
        self.zoom = (self.w() / MAP_W).max(self.h() / MAP_H);
        self.cam = lonlat_to_map(140.0, 35.0);
        self.clamp_cam();
    }

    fn clamp_cam(&mut self) {
        let zmin = self.min_zoom();
        self.zoom = self.zoom.clamp(zmin, MAX_ZOOM);
        let hw = self.w() * 0.5 / self.zoom;
        let hh = self.h() * 0.5 / self.zoom;
        self.cam.0 = if hw * 2.0 >= MAP_W { MAP_W * 0.5 } else { self.cam.0.clamp(hw, MAP_W - hw) };
        self.cam.1 = if hh * 2.0 >= MAP_H { MAP_H * 0.5 } else { self.cam.1.clamp(hh, MAP_H - hh) };
    }

    fn screen_to_map(&self, sx: f32, sy: f32) -> (f32, f32) {
        ((sx - self.w() * 0.5) / self.zoom + self.cam.0, (sy - self.h() * 0.5) / self.zoom + self.cam.1)
    }

    fn map_to_screen(&self, x: f32, y: f32) -> (f32, f32) {
        ((x - self.cam.0) * self.zoom + self.w() * 0.5, (y - self.cam.1) * self.zoom + self.h() * 0.5)
    }

    fn dive_view(&self) -> View {
        let ppm = self.h() / 4.0 * self.lens;
        View { cx: self.dcam.0, cy: self.dcam.1, ppm, w: self.w(), h: self.h() }
    }

    fn notify_text(&mut self, s: String) {
        self.notices.push((s, 5.0));
        if self.notices.len() > 4 {
            self.notices.remove(0);
        }
    }

    /// 保存・読み込みの結果など（ブラウザ側・デスクトップ版の記録の層から）
    pub fn notify(&mut self, s: &str) {
        self.notify_text(s.to_string());
    }

    pub fn apply_loaded(&mut self, data: &[u8]) {
        match Progress::from_bytes(data) {
            Ok(p) => {
                self.prog = p;
                self.cap = lonlat_to_map(self.prog.lon, self.prog.lat);
                self.cap_target = None;
                self.pending_hab = None;
                self.scene = None;
                self.photo = None;
                self.screen = Screen::Map;
                self.panel = Panel::None;
                self.zoom = self.kanto_zoom() * 1.6;
                self.cam = self.cap;
                self.follow = true;
                self.clamp_cam();
                self.notify_text(t!("記録を読み込んだ", "Save loaded", "已读取存档").to_string());
            }
            Err(e) => self.notify_text(tf!("読み込めなかった：{}", "Could not load: {}", "无法读取：{}", e)),
        }
    }

    pub fn state_text(&self) -> String {
        format!("screen={:?} panel={:?} mission={} found={}", self.screen, self.panel, self.prog.mission, self.prog.found())
    }

    pub fn crash_bytes(&self) -> Option<Vec<u8>> {
        if self.screen == Screen::Title {
            None
        } else {
            Some(self.prog.to_bytes())
        }
    }

    fn autosave(&mut self, now: bool) {
        self.save_dirty = true;
        if now {
            self.save_cool = 0.0;
        }
    }

    fn save_tick(&mut self, dt: f32) {
        self.save_cool -= dt;
        if self.save_dirty && self.save_cool <= 0.0 && self.screen != Screen::Title {
            let (lon, lat) = map_to_lonlat(self.cap.0, self.cap.1);
            self.prog.lon = lon;
            self.prog.lat = lat;
            let data = self.prog.to_bytes();
            platform::save_slot(0, &data, &self.prog.meta());
            self.have_save = true;
            self.save_dirty = false;
            self.save_cool = 4.0;
        }
    }

    pub fn hit_kind(&self, x: f32, y: f32) -> u32 {
        let (x, y) = (x / self.gfx.ui, y / self.gfx.ui);
        if self.panel == Panel::Guide {
            let (x0, y0, x1, y1, _, _, _) = self.guide_layout();
            if x >= x0 && x <= x1 && y >= y0 && y <= y1 {
                return 3;
            }
        }
        if self.button_at(x, y).is_some() {
            return 1;
        }
        if self.panel != Panel::None || self.screen == Screen::Title || self.photo.is_some() {
            return 1;
        }
        if self.over_ui(x, y) {
            return 1;
        }
        0
    }

    fn button_at(&self, x: f32, y: f32) -> Option<Act> {
        self.buttons.iter().rev().find(|b| b.enabled && x >= b.x0 && x <= b.x1 && y >= b.y0 && y <= b.y1).map(|b| b.act)
    }

    fn over_ui(&self, x: f32, y: f32) -> bool {
        if y < self.top_h() {
            return true;
        }
        if self.screen == Screen::Map {
            let (mx0, my0, mx1, my1) = self.month_rect();
            if x >= mx0 && x <= mx1 && y >= my0 && y <= my1 {
                return true;
            }
            let (lx0, ly0, lx1, ly1) = self.loc_rect();
            if x >= lx0 && x <= lx1 && y >= ly0 && y <= ly1 {
                return true;
            }
        }
        if let Some((mx0, my0, mx1, my1)) = self.mission_rect() {
            if x >= mx0 && x <= mx1 && y >= my0 && y <= my1 {
                return true;
            }
        }
        false
    }

    // ------------------------------------------------------------ 1フレーム

    pub fn frame(&mut self, evs: &[Ev], dt: f32, focused: bool) -> bool {
        if let Some(data) = platform::take_loaded() {
            self.apply_loaded(&data);
        }
        for n in platform::take_notes() {
            self.notify(&n);
        }
        let _ = focused;
        self.clock += dt;
        let ui = self.gfx.ui;
        for e in evs {
            let e = match *e {
                Ev::MouseMove(x, y) => Ev::MouseMove(x / ui, y / ui),
                Ev::Down(b, x, y) => Ev::Down(b, x / ui, y / ui),
                Ev::Up(b, x, y) => Ev::Up(b, x / ui, y / ui),
                Ev::Scroll(d) => Ev::Scroll(d / ui),
                o => o,
            };
            if self.handle(e) {
                return true;
            }
        }
        self.update(dt);
        self.save_tick(dt);
        self.dialog_voice.clear();
        self.render();
        self.voice_tick(dt);
        false
    }

    fn voice_text(&self) -> String {
        if let Some(p) = &self.photo {
            if let Some(&(e, _, fresh, _)) = p.lines.get(p.index) {
                return tf!("{}。{}。下のボタンで次へ進もう。", "{}. {}. Press the button below to continue.", "{}。{}。按下方按钮继续。",
                    if fresh { t!("新発見", "New discovery", "新发现") } else { t!("撮影成功", "Photo taken", "拍摄成功") }, entry_name(e));
            }
            return p.advice.unwrap_or("").to_string();
        }
        if !self.dialog_voice.is_empty() { return self.dialog_voice.clone(); }
        let m = &MISSIONS[self.prog.mission];
        match self.panel {
            Panel::Entry(e) => {
                if self.prog.stars[e] == 0 {
                    return t!("まだ撮影していない。会える場所を調べて、探しに行こう。", "Not photographed yet. Check where it lives, then go exploring.", "尚未拍摄。查看它在哪里，出发寻找吧。").to_string();
                }
                if let Some(book) = &self.entry_book {
                    if let Some((title, body)) = book.pages.get(self.entry_page) {
                        return format!("{}。{}。{}", entry_name(e), title, body);
                    }
                }
                entry_name(e).to_string()
            }
            Panel::Missions => format!("{}。{}", m.title.get(), m.brief.get()),
            Panel::Guide => t!("虫の絵を押すと、お話が聞けるよ。", "Choose an insect's picture to hear about it.", "点击昆虫图片，听听它的故事吧。").to_string(),
            Panel::Help => t!("地図で光っている観察スポットを押そう。着いたら、降りるボタンで林や草はらへ。虫に近づいて、撮影ボタンで写真を撮ろう。左上のスピーカーで声を切り替え、丸い矢印でもう一度聞けるよ。", "Choose a glowing spot on the map. When you arrive, press a Land button. Move closer to an insect and press Photo. The speaker turns voice on or off; the round arrow repeats the guide.", "点击地图上发光的观察点。到达后按降落按钮。靠近昆虫，按拍摄。左上角喇叭开关语音，圆箭头可以重听。").to_string(),
            _ => match self.screen {
                Screen::Title => t!("昆虫図鑑へようこそ！ はじめからのボタンで観察に出発しよう。左上の丸い矢印で、もう一度聞けるよ。", "Welcome to YYInsects! Press New journey to start exploring. The round arrow at the top left repeats the guide.", "欢迎来到昆虫图鉴！按从头开始出发观察。左上角圆箭头可以重听。").to_string(),
                Screen::Map => tf!("{}。{}。右上の宿題の、行くボタンを押してみよう。着いたら降りるボタンで、虫を探そう。", "{}. {}. Press Go on the task card at the top right. Then press a Land button to look for insects.", "{}。{}。点击右上角作业卡的前往。到达后按降落寻找昆虫。", m.title.get(), m.brief.get()),
                Screen::Dive => t!("虫を見つけよう。進みたい方を押し続けると動くよ。虫に近づいて、撮影ボタンを押そう。暗いときはライトをつけてね。", "Look for insects. Hold the direction you want to move. Get closer, then press Photo. Turn on the light if it is dark.", "找找昆虫吧。按住想去的方向移动。靠近昆虫后按拍摄。太暗时记得开灯。").to_string(),
            }
        }
    }

    fn voice_tick(&mut self, dt: f32) {
        if !self.voice { return; }
        let text = self.voice_text();
        // Include photo count so taking the same photograph still gives feedback.
        let key = format!("{}:{:?}:{}:{}", lang::lang() as u8, self.panel, self.prog.photos, text);
        if key != self.spoken {
            self.spoken = key;
            self.hover_voice = None;
            self.voice_mouse = self.mouse;
            platform::speak(&text);
            return;
        }
        if self.mouse == self.voice_mouse { return; }
        let label = if self.touch { None } else {
            self.buttons.iter().rev().find(|b| b.enabled && self.mouse.0 >= b.x0 && self.mouse.0 <= b.x1 && self.mouse.1 >= b.y0 && self.mouse.1 <= b.y1).map(|b| match b.act {
                Act::Voice => t!("声をつける、消す", "Turn voice on or off", "开关语音").to_string(),
                Act::ReplayVoice => t!("もう一度聞く", "Listen again", "再听一次").to_string(),
                Act::CycleMission(d) => if d < 0 { t!("前の宿題", "Previous task", "上一个作业") } else { t!("次の宿題", "Next task", "下一个作业") }.to_string(),
                Act::StepTime(_) => t!("次の時間帯", "Next time of day", "下一个时间段").to_string(),
                Act::StepMonth(d) => if d < 0 { t!("前の月", "Previous month", "上个月") } else { t!("次の月", "Next month", "下个月") }.to_string(),
                Act::StepSpot(d) => if d < 0 { t!("前のスポット", "Previous spot", "上一个观察点") } else { t!("次のスポット", "Next spot", "下一个观察点") }.to_string(),
                Act::Lens(d) => if d < 0 { t!("小さく見る", "Zoom out", "缩小") } else { t!("大きく見る", "Zoom in", "放大") }.to_string(),
                Act::MissionCard => t!("宿題を開く、閉じる", "Open or close the task", "展开或收起作业").to_string(),
                _ => b.label.clone(),
            })
        };
        if let Some(label) = label {
            match &mut self.hover_voice {
                Some((old, age)) if *old == label => {
                    *age += dt;
                    if *age >= 0.7 && *age - dt < 0.7 { platform::speak(&label); }
                }
                _ => self.hover_voice = Some((label, 0.0)),
            }
        } else { self.hover_voice = None; }
    }

    fn update(&mut self, dt: f32) {
        self.confirm_new = (self.confirm_new - dt).max(0.0);
        self.confirm_reload = (self.confirm_reload - dt).max(0.0);
        self.flash = (self.flash - dt * 2.2).max(0.0);
        for n in &mut self.notices {
            n.1 -= dt;
        }
        self.notices.retain(|n| n.1 > 0.0);
        if let Some(p) = &mut self.photo {
            p.age += dt;
            if !self.voice { p.left -= dt; }
            if p.left <= 0.0 {
                self.next_photo();
            }
        }
        match self.screen {
            Screen::Title => {
                self.title_t += dt;
                let r = self.earth.kanto.rect;
                self.zoom = self.kanto_zoom() * 0.55;
                self.cam = ((r[0] + r[2]) * 0.5 + (self.title_t * 0.05).sin() * 6.0, (r[1] + r[3]) * 0.5 + (self.title_t * 0.04).cos() * 3.0);
                self.clamp_cam();
            }
            Screen::Map => {
                self.update_map(dt);
                self.update_radar();
            }
            Screen::Dive => self.update_dive(dt),
        }
    }

    fn update_map(&mut self, dt: f32) {
        // 矢印キーで地図を動かす
        if self.panel == Panel::None && self.photo.is_none() {
            let mut p = (0.0f32, 0.0f32);
            if self.keys[VK_LEFT as usize] || self.keys[0x41] {
                p.0 -= 1.0;
            }
            if self.keys[VK_RIGHT as usize] || self.keys[0x44] {
                p.0 += 1.0;
            }
            if self.keys[VK_UP as usize] || self.keys[0x57] {
                p.1 -= 1.0;
            }
            if self.keys[VK_DOWN as usize] || self.keys[0x53] {
                p.1 += 1.0;
            }
            if p != (0.0, 0.0) {
                let sp = 700.0 / self.zoom * dt;
                self.cam.0 += p.0 * sp;
                self.cam.1 += p.1 * sp;
                self.follow = false;
            }
        }
        if let Some(tg) = self.cap_target {
            let (dx, dy) = (tg.0 - self.cap.0, tg.1 - self.cap.1);
            let d = (dx * dx + dy * dy).sqrt();
            let v = (d * 2.4).clamp(2.0, 60.0) * dt;
            if d <= v {
                self.cap = tg;
                self.cap_target = None;
                self.autosave(false);
                if let Some(s) = self.at_spot() {
                    self.notify_text(tf!("{}に着いた！ 降りる場所を選ぼう", "Arrived at {}! Choose where to land", "到达{}！选择降落地点吧", SPOTS[s].name.get()));
                }
            } else {
                self.cap.0 += dx / d * v;
                self.cap.1 += dy / d * v;
            }
        }
        if self.follow {
            let k = (dt * 3.0).min(1.0);
            self.cam.0 += (self.cap.0 - self.cam.0) * k;
            self.cam.1 += (self.cap.1 - self.cam.1) * k;
        }
        self.clamp_cam();
        // 先生のことば（初めて）
        if !self.prog.intro_done && self.panel == Panel::None {
            self.panel = Panel::Intro(0);
        }
    }

    fn update_radar(&mut self) {
        let Some(e) = self.radar else { return };
        if self.radar_key != Some(e) {
            self.radar_key = Some(e);
            let mut cells: Vec<(f32, f32)> = places_of(e).into_iter().map(|(s, _)| (SPOTS[s].lon, SPOTS[s].lat)).collect();
            cells.dedup();
            self.radar_cells = cells;
        }
    }

    fn update_dive(&mut self, dt: f32) {
        let blocked = self.panel != Panel::None || self.photo.is_some();
        let mut ax = 0.0;
        let mut ay = 0.0;
        if !blocked {
            if self.keys[0x41] || self.keys[VK_LEFT as usize] {
                ax -= 1.0;
            }
            if self.keys[0x44] || self.keys[VK_RIGHT as usize] {
                ax += 1.0;
            }
            if self.keys[0x57] || self.keys[VK_UP as usize] {
                ay -= 1.0;
            }
            if self.keys[0x53] || self.keys[VK_DOWN as usize] {
                ay += 1.0;
            }
            if self.keys[0x51] {
                self.lens = (self.lens * (1.0 - dt * 1.2)).max(1.0);
            }
            if self.keys[0x45] {
                self.lens = (self.lens * (1.0 + dt * 1.2)).min(8.0);
            }
        }
        if ax != 0.0 || ay != 0.0 {
            self.dive_goto = None;
        }
        let v = self.dive_view();
        let target = if blocked {
            None
        } else if let Some((sx, sy)) = self.steer {
            Some(v.to_world(sx, sy))
        } else {
            self.dive_goto
        };
        let boost = self.keys[VK_SHIFT as usize] || self.boost_btn;
        let lens = self.lens;
        let Some(sc) = &mut self.scene else { return };
        sc.update(dt, ax, ay, target, boost);
        if let Some(g) = self.dive_goto {
            let (dx, dy) = (g.0 - sc.cap.x, g.1 - sc.cap.y);
            if dx * dx + dy * dy < 0.09 {
                self.dive_goto = None;
            }
        }
        // カメラ：カプセルの前（レンズ）を追う
        let lead = if lens > 1.7 { 0.45 } else { 0.25 };
        let tx = sc.cap.x + sc.cap.facing * lead + sc.cap.vx * 0.25;
        let ty = sc.cap.y + sc.cap.vy * 0.2;
        let k = (dt * 5.0).min(1.0);
        self.dcam.0 += (tx - self.dcam.0) * k;
        self.dcam.1 += (ty - self.dcam.1) * k;
    }

    // ------------------------------------------------------------ 入力

    fn handle(&mut self, e: Ev) -> bool {
        match e {
            Ev::Resize(w, h) => {
                self.gfx.resize(w, h);
                self.clamp_cam();
            }
            Ev::Compact(c) => {
                self.gfx.set_compact(c);
                self.clamp_cam();
            }
            Ev::Touch(t) => self.touch = t,
            Ev::MouseMove(x, y) => {
                self.mouse = (x, y);
                if let Some((sx, sy, cam0)) = self.drag {
                    if self.screen == Screen::Map && self.panel == Panel::None {
                        self.cam = (cam0.0 - (x - sx) / self.zoom, cam0.1 - (y - sy) / self.zoom);
                        self.follow = false;
                        self.clamp_cam();
                    }
                }
                if let Some((lx, ly, moved)) = self.ldown {
                    let far = (x - lx).abs() + (y - ly).abs() > 8.0;
                    if self.screen == Screen::Map && !moved && far && self.panel == Panel::None && self.photo.is_none() {
                        self.ldown = Some((lx, ly, true));
                        self.drag = Some((lx, ly, self.cam));
                    }
                }
                if self.screen == Screen::Dive && self.steer.is_some() {
                    self.steer = Some((x, y));
                }
            }
            Ev::Down(b, x, y) => {
                self.mouse = (x, y);
                if b == 0 {
                    if let Some(a) = self.button_at(x, y) {
                        self.ldown = None;
                        self.do_act(a);
                        return false;
                    }
                    if self.photo.is_some() {
                        self.next_photo();
                        return false;
                    }
                    match self.panel {
                        Panel::Intro(_) => {
                            self.do_act(Act::IntroNext);
                            return false;
                        }
                        Panel::Done(_) | Panel::Soon => {
                            self.panel = Panel::None;
                            return false;
                        }
                        Panel::None => {}
                        _ => return false,
                    }
                    if self.screen == Screen::Map && !self.over_ui(x, y) {
                        self.ldown = Some((x, y, false));
                    } else if self.screen == Screen::Dive && !self.over_ui(x, y) {
                        self.steer = Some((x, y));
                        self.ldown = Some((x, y, false));
                    }
                } else if b == 2 || (b == 1 && self.screen == Screen::Map) {
                    if self.screen == Screen::Map && self.panel == Panel::None {
                        self.drag = Some((x, y, self.cam));
                    } else if self.screen == Screen::Dive && self.panel == Panel::None && self.photo.is_none() {
                        self.steer = Some((x, y));
                    }
                } else if b == 1 && self.screen == Screen::Dive && self.panel == Panel::None {
                    self.do_act(Act::Shoot);
                }
            }
            Ev::Up(b, x, y) => {
                self.mouse = (x, y);
                if b == 0 {
                    if let Some((lx, ly, moved)) = self.ldown.take() {
                        if self.screen == Screen::Map && !moved {
                            self.map_click(lx, ly);
                        }
                        if self.screen == Screen::Dive {
                            let short = (x - lx).abs() + (y - ly).abs() < 12.0;
                            if short {
                                self.dive_goto = Some(self.dive_view().to_world(x, y));
                            }
                        }
                    }
                    self.drag = None;
                    self.steer = None;
                    self.boost_btn = false;
                } else {
                    self.drag = None;
                    if self.screen == Screen::Dive {
                        self.steer = None;
                    }
                }
            }
            Ev::Wheel(d) => {
                if self.panel == Panel::Guide {
                    self.guide_scroll = (self.guide_scroll - d * 60.0).clamp(0.0, self.guide_max);
                } else if matches!(self.panel, Panel::Entry(_)) {
                    if d != 0.0 { self.do_act(Act::EntryPage(if d < 0.0 { 1 } else { -1 })); }
                } else if self.panel == Panel::None {
                    match self.screen {
                        Screen::Map => {
                            let before = self.screen_to_map(self.mouse.0, self.mouse.1);
                            self.zoom *= 1.15f32.powf(d);
                            self.clamp_cam();
                            let after = self.screen_to_map(self.mouse.0, self.mouse.1);
                            self.cam.0 += before.0 - after.0;
                            self.cam.1 += before.1 - after.1;
                            self.follow = false;
                            self.clamp_cam();
                        }
                        Screen::Dive => self.lens = (self.lens * 1.15f32.powf(d)).clamp(1.0, 8.0),
                        _ => {}
                    }
                }
            }
            Ev::Scroll(d) => {
                if self.panel == Panel::Guide {
                    self.guide_scroll = (self.guide_scroll - d).clamp(0.0, self.guide_max);
                }
            }
            Ev::Key(k, down) => {
                if (k as usize) < 256 {
                    self.keys[k as usize] = down;
                }
                if down {
                    self.key_down(k);
                }
            }
            Ev::Char(_) => {}
        }
        false
    }

    fn key_down(&mut self, k: u32) {
        if k == 0x73 { self.do_act(Act::ReplayVoice); return; }
        if k == 0x74 { self.do_act(Act::Voice); return; }
        let ctrl = self.keys[VK_CONTROL as usize];
        if ctrl && (k == VK_PLUS || k == VK_ADD) {
            self.gfx.adjust_scale(1);
            return;
        }
        if ctrl && (k == VK_MINUS || k == VK_SUB) {
            self.gfx.adjust_scale(-1);
            return;
        }
        if k == VK_F11 {
            self.gfx.toggle_fullscreen();
            return;
        }
        if self.photo.is_some() {
            if k == VK_ESCAPE {
                self.close_photo();
            } else if k == VK_SPACE || k == VK_RETURN {
                self.next_photo();
            }
            return;
        }
        match self.panel {
            Panel::Intro(_) => {
                if k == VK_SPACE || k == VK_RETURN || k == VK_ESCAPE {
                    self.do_act(Act::IntroNext);
                }
                return;
            }
            Panel::Done(_) | Panel::Soon => {
                if k == VK_SPACE || k == VK_RETURN || k == VK_ESCAPE {
                    self.panel = Panel::None;
                }
                return;
            }
            Panel::Entry(i) => {
                if k == VK_LEFT || k == 0x41 {
                    self.do_act(Act::Entry((i + n_entries() - 1) % n_entries()));
                } else if k == VK_RIGHT || k == 0x44 {
                    self.do_act(Act::Entry((i + 1) % n_entries()));
                } else if k == VK_DOWN || k == 0x22 || k == VK_SPACE {
                    self.do_act(Act::EntryPage(1));
                } else if k == VK_UP || k == 0x21 {
                    self.do_act(Act::EntryPage(-1));
                } else if k == VK_ESCAPE || k == VK_BACK || k == VK_F2 || k == 0x47 {
                    self.do_act(Act::Close);
                }
                return;
            }
            Panel::Guide | Panel::Missions | Panel::Help => {
                if k == VK_ESCAPE || (k == VK_F2 && self.panel == Panel::Guide) || (k == VK_F1 && self.panel == Panel::Missions) || (k == VK_F3 && self.panel == Panel::Help) || k == 0x47 && self.panel == Panel::Guide || k == 0x4d && self.panel == Panel::Missions || k == 0x48 && self.panel == Panel::Help {
                    self.panel = Panel::None;
                }
                return;
            }
            Panel::None => {}
        }
        match self.screen {
            Screen::Title => {
                if k == VK_F3 || k == 0x48 {
                    self.do_act(Act::Help);
                } else if k == VK_RETURN || k == VK_SPACE {
                    if self.have_save {
                        self.do_act(Act::Continue);
                    } else {
                        self.do_act(Act::NewGame);
                    }
                }
            }
            Screen::Map => match k {
                VK_RETURN => {
                    if let Some(s) = self.at_spot() {
                        let h = self.pending_hab.filter(|h| SPOTS[s].habs.contains(h)).unwrap_or(SPOTS[s].habs[0]);
                        self.do_act(Act::Dive(h));
                    }
                }
                VK_F1 | 0x4d => self.do_act(Act::Missions),
                VK_F2 | 0x47 => self.do_act(Act::Guide),
                VK_F3 | 0x48 => self.do_act(Act::Help),
                VK_COMMA => self.do_act(Act::StepMonth(-1)),
                VK_PERIOD => self.do_act(Act::StepMonth(1)),
                0xdb => self.do_act(Act::StepSpot(-1)),
                0xdd => self.do_act(Act::StepSpot(1)),
                0x4b => self.do_act(Act::ShowKanto),
                0x54 => self.do_act(Act::StepTime(1)),
                VK_PLUS | VK_ADD => {
                    self.zoom *= 1.25;
                    self.clamp_cam();
                }
                VK_MINUS | VK_SUB => {
                    self.zoom /= 1.25;
                    self.clamp_cam();
                }
                VK_ESCAPE => self.do_act(Act::Help),
                _ => {}
            },
            Screen::Dive => match k {
                VK_RETURN | VK_ESCAPE => self.do_act(Act::Surface),
                VK_SPACE => self.do_act(Act::Shoot),
                0x4c => self.do_act(Act::Light),
                VK_F1 | 0x4d => self.do_act(Act::Missions),
                VK_F2 | 0x47 => self.do_act(Act::Guide),
                VK_F3 | 0x48 => self.do_act(Act::Help),
                VK_PLUS | VK_ADD => self.do_act(Act::Lens(1)),
                VK_MINUS | VK_SUB => self.do_act(Act::Lens(-1)),
                _ => {}
            },
        }
    }

    fn do_act(&mut self, a: Act) {
        platform::crumb(format!("act {:?}", a));
        match a {
            Act::Voice => {
                self.voice = !self.voice;
                platform::save_voice(self.voice);
                platform::speak("");
                self.spoken.clear();
            }
            Act::ReplayVoice => {
                if !self.voice { self.voice = true; platform::save_voice(true); }
                self.spoken.clear();
            }
            Act::NewGame => {
                if self.have_save && self.confirm_new <= 0.0 {
                    self.confirm_new = 4.0;
                    return;
                }
                self.prog = Progress::new();
                self.cap = lonlat_to_map(self.prog.lon, self.prog.lat);
                self.cap_target = None;
                self.pending_hab = None;
                self.screen = Screen::Map;
                self.panel = Panel::Intro(0);
                self.show_kanto();
                self.follow = false;
                self.autosave(true);
            }
            Act::Continue => platform::load_slot(0),
            Act::Import => platform::import_file(),
            Act::Export => platform::export_slot(0),
            Act::Reload => {
                if self.confirm_reload <= 0.0 {
                    self.confirm_reload = 4.0;
                    return;
                }
                self.confirm_reload = 0.0;
                if self.screen != Screen::Title {
                    self.autosave(true);
                    self.save_tick(0.0);
                }
                self.notify_text(t!("キャッシュを消して読み込み直しています…", "Clearing the cache and reloading…", "正在清除缓存并重新加载……").to_string());
                platform::reload_clear();
            }
            Act::Lang => {
                let l = lang::lang().next();
                lang::set_lang(l);
                platform::save_lang(l);
                self.gfx.reload_fonts();
            }
            Act::Fullscreen => self.gfx.toggle_fullscreen(),
            Act::Dive(h) => self.enter_dive(h),
            Act::Surface => {
                if self.screen == Screen::Dive {
                    self.scene = None;
                    self.screen = Screen::Map;
                    self.steer = None;
                    self.dive_goto = None;
                    self.autosave(false);
                }
            }
            Act::Shoot => self.shoot(),
            Act::NextPhoto => self.next_photo(),
            Act::Guide => {
                self.panel = if self.panel == Panel::Guide { Panel::None } else { Panel::Guide };
                if self.panel == Panel::Guide {
                    // 新しく収録した項目のあるなかま、なければいまのタブを開く
                    if let Some(e) = (0..n_entries()).find(|&e| self.prog.fresh[e]) {
                        self.guide_group = GROUPS.iter().position(|&g| g == entry_group(e)).unwrap_or(0);
                    }
                    self.guide_scroll = 0.0;
                }
            }
            Act::Missions => {
                self.panel = if self.panel == Panel::Missions { Panel::None } else { Panel::Missions };
            }
            Act::Help => {
                self.panel = if self.panel == Panel::Help { Panel::None } else { Panel::Help };
            }
            Act::Close => {
                self.panel = match self.panel {
                    Panel::Entry(_) => Panel::Guide,
                    _ => Panel::None,
                };
            }
            Act::Entry(i) => {
                if i < n_entries() {
                    self.prog.fresh[i] = false;
                    self.entry_page = 0;
                    self.entry_book = None;
                    self.panel = Panel::Entry(i);
                }
            }
            Act::Goto(tg) => self.goto(tg),
            Act::StepSpot(d) => {
                let (lon, lat) = map_to_lonlat(self.cap.0, self.cap.1);
                let cur = SPOTS.iter().enumerate().min_by(|a, b| km_dist((a.1.lon, a.1.lat), (lon, lat)).partial_cmp(&km_dist((b.1.lon, b.1.lat), (lon, lat))).unwrap()).map_or(0, |x| x.0);
                let n = SPOTS.len() as i32;
                self.fly_to_spot((cur as i32 + d).rem_euclid(n) as usize, None);
            }
            Act::ShowKanto => self.show_kanto(),
            Act::Month(m) => self.set_month(m),
            Act::TimeOfDay(t) => self.set_time(t),
            Act::StepTime(d) => {
                let t = (self.prog.time as i32 + d).rem_euclid(4) as u8;
                self.set_time(t);
            }
            Act::StepMonth(d) => {
                let m = (self.prog.month as i32 - 1 + d).rem_euclid(12) as u8 + 1;
                self.set_month(m);
            }
            Act::EntryPage(delta) => {
                if let Panel::Entry(e) = self.panel {
                    if self.prog.stars[e] > 0 {
                        if let Some(book) = &self.entry_book {
                            self.entry_page = (self.entry_page as i32 + delta).clamp(0, book.pages.len().saturating_sub(1) as i32) as usize;
                        }
                    }
                }
            }
            Act::SelectMission(i) => {
                if i < MISSIONS.len() {
                    self.prog.mission = i;
                    self.mission_open = true;
                    self.autosave(true);
                }
            }
            Act::CycleMission(d) => {
                self.prog.select_next(d);
                self.autosave(true);
            }
            Act::Radar(e) => {
                if e < n_entries() {
                    self.radar = Some(e);
                    self.radar_key = None;
                    self.panel = Panel::None;
                    // 観察中で、ここにいないなら地図に戻る
                    let here = self.scene.as_ref().map_or(false, |sc| sc.present.contains(&e));
                    if self.screen == Screen::Dive && !here {
                        self.do_act(Act::Surface);
                    }
                    if self.screen == Screen::Map {
                        self.show_kanto();
                    }
                    self.notify(&tf!("レーダー：{} を探す", "Radar: searching for {}", "雷达：寻找 {}", entry_name(e)));
                }
            }
            Act::RadarOff => {
                self.radar = None;
                self.radar_cells.clear();
                self.radar_key = None;
            }
            Act::RadarGo => {
                if let Some(e) = self.radar {
                    self.goto(Target::Sp(e));
                }
            }
            Act::GuideTab(i) => {
                let g = i.min(GROUPS.len() - 1);
                if g != self.guide_group {
                    self.guide_group = g;
                    self.guide_scroll = 0.0;
                }
            }
            Act::Light => {
                self.prog.light = !self.prog.light;
                if self.voice { platform::speak(if self.prog.light { t!("ライトをつけたよ", "Light on", "灯打开了") } else { t!("ライトを消したよ", "Light off", "灯关上了") }); }
            }
            Act::Lens(d) => {
                self.lens = (self.lens * 1.35f32.powi(d)).clamp(1.0, 8.0);
            }
            Act::ToTitle => {
                self.autosave(true);
                self.save_tick(0.0);
                self.screen = Screen::Title;
                self.scene = None;
                self.panel = Panel::None;
                self.photo = None;
            }
            Act::IntroNext => {
                if let Panel::Intro(n) = self.panel {
                    if n >= 3 {
                        self.panel = Panel::None;
                        self.prog.intro_done = true;
                        self.mission_open = true;
                        self.autosave(true);
                    } else {
                        self.panel = Panel::Intro(n + 1);
                    }
                }
            }
            Act::MapZoom(d) => {
                self.zoom *= 1.4f32.powi(d);
                self.follow = false;
                self.clamp_cam();
            }
            Act::MissionCard => self.mission_open = !self.mission_open,
            Act::Boost => self.boost_btn = true,
        }
    }

    /// 月（季節）を変える。観察中は地図に戻ってから
    fn set_month(&mut self, m: u8) {
        let m = m.clamp(1, 12);
        if m == self.prog.month {
            return;
        }
        if self.screen == Screen::Dive {
            self.do_act(Act::Surface);
        }
        let before = season_of(self.prog.month);
        self.prog.month = m;
        let label = if season_of(m) != before {
            tf!("{}になった（{}）", "It's now {} ({})", "到了{}（{}）", season_name(m), month_name(m))
        } else {
            tf!("{}になった", "It's now {}", "到了{}", month_name(m))
        };
        // 月を続けて変えたときは、知らせを一つだけにする
        let prev = std::mem::take(&mut self.month_notice);
        self.notices.retain(|n| n.0 != prev);
        self.month_notice = label.clone();
        self.notify_text(label);
        self.autosave(false);
    }

    /// 時間帯を変える。観察中は地図に戻ってから
    fn set_time(&mut self, t: u8) {
        let t = t % 4;
        if t == self.prog.time {
            return;
        }
        if self.screen == Screen::Dive {
            self.do_act(Act::Surface);
        }
        self.prog.time = t;
        let label = tf!("{}になった", "It's now {}", "到了{}", time_name(t));
        let prev = std::mem::take(&mut self.month_notice);
        self.notices.retain(|n| n.0 != prev);
        self.month_notice = label.clone();
        self.notify_text(label);
        self.autosave(false);
    }

    /// 観察スポットへ飛ぶ（hab：着いたら降りる場面）
    fn fly_to_spot(&mut self, s: usize, hab: Option<Habitat>) {
        if self.screen == Screen::Dive {
            self.do_act(Act::Surface);
        }
        self.panel = Panel::None;
        let sp = &SPOTS[s];
        self.cap_target = Some(lonlat_to_map(sp.lon, sp.lat));
        if self.at_spot_is(s) {
            self.cap_target = None;
        }
        self.pending_hab = hab;
        self.follow = true;
        self.zoom = self.zoom.max(self.kanto_zoom() * 1.3);
    }

    fn at_spot_is(&self, s: usize) -> bool {
        let (lon, lat) = map_to_lonlat(self.cap.0, self.cap.1);
        km_dist((SPOTS[s].lon, SPOTS[s].lat), (lon, lat)) < 0.5
    }

    /// 地図を押した：観察スポットなら飛ぶ。関東の外なら「準備中」
    fn map_click(&mut self, sx: f32, sy: f32) {
        // いちばん近いスポット（画面で 28px 以内）
        let near = SPOTS.iter().enumerate().map(|(i, s)| {
            let (mx, my) = lonlat_to_map(s.lon, s.lat);
            let (px, py) = self.map_to_screen(mx, my);
            (i, (px - sx).hypot(py - sy))
        }).min_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        if let Some((i, d)) = near {
            if d < 28.0 {
                self.fly_to_spot(i, None);
                return;
            }
        }
        let (mx, my) = self.screen_to_map(sx, sy);
        let (lon, lat) = map_to_lonlat(mx, my);
        match self.earth.place(lon, lat) {
            Place::Kanto(_) => self.notify_text(t!("光っている観察スポットを選んでね", "Choose one of the glowing observation spots", "请选择发光的观察点").to_string()),
            Place::Sea => self.notify_text(t!("そこは海の上。関東の観察スポットを選んでね", "That's the sea. Choose an observation spot in Kanto", "那里是海上。请选择关东的观察点").to_string()),
            Place::Japan(code) => {
                self.soon_name = if (8..=14).contains(&code) {
                    // 伊豆諸島・小笠原諸島
                    t!("東京の島々", "the Tokyo islands", "东京的岛屿").to_string()
                } else if code == 0 {
                    t!("この地域", "this area", "这个地区").to_string()
                } else {
                    tf!("{}（{}）", "{} ({})", "{}（{}）", district_name(code), pref_name(code))
                };
                self.panel = Panel::Soon;
            }
            Place::Abroad => {
                self.soon_name = abroad_name(lon, lat).to_string();
                self.panel = Panel::Soon;
            }
        }
    }

    /// 宿題の対象のところへ行く（スポットと場面）
    fn goto(&mut self, tg: Target) {
        let Target::Sp(i) = tg;
        let near = map_to_lonlat(self.cap.0, self.cap.1);
        let (s, h, m, t) = where_is(i, near, self.prog.month, self.prog.time);
        self.fly_to_spot(s, Some(h));
        if m != self.prog.month {
            self.set_month(m);
        }
        if t != self.prog.time {
            self.set_time(t);
        }
        if self.cap_target.is_none() {
            self.notify_text(tf!("ここの「{}」で会えるよ", "You can find it in the {} here", "在这里的{}能遇到", h.name()));
        }
    }

    fn enter_dive(&mut self, hab: Habitat) {
        if self.screen != Screen::Map {
            return;
        }
        let Some(s) = self.at_spot() else { return };
        if !SPOTS[s].habs.contains(&hab) {
            return;
        }
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let sc = Scene::new(s, hab, self.prog.month, self.prog.time, self.seed);
        self.dcam = (sc.cap.x + 0.5, sc.cap.y);
        self.scene = Some(sc);
        self.lens = 1.0;
        self.screen = Screen::Dive;
        self.steer = None;
        self.dive_goto = None;
        self.ldown = None;
        self.pending_hab = None;
        if self.scene.as_ref().map_or(false, |sc| sc.present.is_empty()) {
            self.notify_text(tf!("{}の{}は、ここで見られる虫がいない。ほかの季節や時間帯に来てみよう", "No insects here in {} ({}). Try another season or time of day", "{}的{}这里看不到昆虫。换个季节或时间再来吧", month_name(self.prog.month), time_name(self.prog.time)));
        } else if self.prog.time == 3 && !self.prog.light {
            self.notify_text(t!("夜は暗い。ライト（L）をつけよう", "It's dark at night. Turn on your light (L)", "夜里很暗。打开灯（L）吧").to_string());
        } else if matches!(hab, Habitat::Forest | Habitat::Mountain) && !self.prog.light && season_of(self.prog.month) != 3 {
            self.notify_text(t!("木かげは暗い。ライト（L）をつけると写真がきれいに撮れる", "It's shady under the trees. Turn on your light (L) for better photos", "树荫下很暗。打开灯（L）能拍得更清楚").to_string());
        }
    }

    // ------------------------------------------------------------ 撮影

    fn shoot(&mut self) {
        if self.photo.is_some() || self.panel != Panel::None {
            return;
        }
        let mut found: Vec<(usize, f32)> = Vec::new();
        let mut advice = None;
        match self.screen {
            Screen::Dive => {
                let v = self.dive_view();
                let Some(sc) = &self.scene else { return };
                let r = sc.shoot(&v, self.prog.light);
                for &(sp, s) in &r.subjects {
                    found.push((sp, s));
                }
                let best = r.subjects.first().map_or(0.0, |b| b.1);
                if r.subjects.is_empty() {
                    advice = Some(t!("何も写っていない。虫に近づいて撮ろう", "Nothing in the shot. Get closer to an insect.", "什么也没拍到。靠近昆虫再拍吧。"));
                } else if best < 30.0 {
                    let dark = sc.ambient_at(sc.cap.y) < 0.7 && !self.prog.light;
                    advice = Some(if dark {
                        t!("暗くてよく写らない。ライト（L）をつけよう", "Too dark. Turn on your light (L).", "太暗了，拍不清楚。打开灯（L）吧。")
                    } else {
                        t!("小さすぎる。近づくか、ズーム（ホイール・Q/E）で大きく撮ろう", "Too small. Move closer or zoom in (wheel, Q/E).", "太小了。靠近一些或用变焦（滚轮、Q/E）放大。")
                    });
                }
            }
            Screen::Map => {
                self.notify_text(t!("虫は、観察スポットに降りてから撮ろう", "Land at an observation spot to photograph insects", "降落到观察点后再拍昆虫吧").to_string());
                return;
            }
            Screen::Title => return,
        }
        self.flash = 1.0;
        let before_done = self.prog.done_flags();
        self.prog.photos += 1;
        let mut lines = Vec::new();
        for (e, score) in found {
            let st = stars_of(score);
            if st == 0 {
                continue;
            }
            let (new, better) = self.prog.record(e, st);
            lines.push((e, st, new, better));
        }
        lines.sort_by(|a, b| b.1.cmp(&a.1));
        lines.truncate(6);
        if lines.is_empty() && advice.is_none() {
            advice = Some(t!("写りが悪かった。もっと大きく、真ん中に", "Poor shot. Make the subject bigger and centred.", "拍得不好。拍大一点、放在中间。"));
        }
        let done = self.prog.advance(&before_done);
        if !lines.is_empty() || !done.is_empty() {
            self.autosave(true);
        }
        self.photo = Some(PhotoCard { lines, advice, left: crate::reveal::HOLD_SECONDS, age: 0.0, index: 0, done_missions: done });
    }

    fn next_photo(&mut self) {
        if let Some(p) = &mut self.photo {
            if !p.lines.is_empty() && p.age < 0.7 { return; }
            if p.index + 1 < p.lines.len() {
                p.index += 1;
                p.age = 0.0;
                p.left = crate::reveal::HOLD_SECONDS;
                return;
            }
        }
        self.close_photo();
    }

    fn close_photo(&mut self) {
        if let Some(p) = self.photo.take() {
            if let Some(&m) = p.done_missions.last() {
                self.panel = Panel::Done(m);
            }
        }
    }

    // ------------------------------------------------------------ 描画

    fn render(&mut self) {
        self.buttons.clear();
        self.tip = None;
        self.gfx.begin([0.02, 0.05, 0.09, 1.0]);
        match self.screen {
            Screen::Title => {
                self.render_map_world();
                self.gfx.set_screen_xform();
                let (w, h) = (self.w(), self.h());
                self.artwork.draw(&mut self.gfx, "title", [0.0, 0.0, w, h], 1.0, Fit::Cover);
                for i in 0..32 {
                    let t = i as f32 / 32.0;
                    self.gfx.rect(w * t, 0.0, w * (t + 1.0 / 32.0), h, [0.01, 0.04, 0.02, 0.6 * (1.0 - t)]);
                }
            }
            Screen::Map => self.render_map_world(),
            Screen::Dive => {
                let v = self.dive_view();
                let hide = self.lens >= 1.8;
                let backdrop = self.scene.as_ref().and_then(backdrop_slug);
                let backdrop = backdrop.and_then(|slug| self.artwork.texture_slug(&self.gfx, slug));
                if let Some(sc) = &self.scene {
                    sc.render(&mut self.gfx, &v, self.prog.light, hide, backdrop, &mut self.units);
                }
            }
        }
        self.gfx.set_screen_xform();
        // 画面部品のうち、絵を含むもの（WebGL / Direct3D で描く）
        match self.panel {
            Panel::Guide => self.render_guide_gl(),
            Panel::Entry(i) => self.render_entry_gl(i),
            _ => {}
        }
        self.render_photo_gl();
        if self.flash > 0.0 {
            let (w, h) = (self.w(), self.h());
            self.gfx.rect(0.0, 0.0, w, h, [1.0, 1.0, 1.0, self.flash.min(1.0) * 0.85]);
        }
        self.gfx.flush();
        if self.gfx.d2d_begin() {
            let full = self.photo.is_some() || matches!(self.panel, Panel::Guide | Panel::Entry(_) | Panel::Missions | Panel::Help);
            if !full {
                match self.screen {
                    Screen::Title => self.render_title(),
                    Screen::Map => self.render_map_ui(),
                    Screen::Dive => self.render_dive_ui(),
                }
            }
            if self.screen != Screen::Title && self.photo.is_none() {
                if matches!(self.panel, Panel::Missions | Panel::Help) {
                    let (w, h) = (self.w(), self.h());
                    self.gfx.fill_rect(0.0, 0.0, w, h, [0.0, 0.03, 0.06, 0.5]);
                }
                self.render_top();
                if !full {
                    self.render_mission_card();
                }
            }
            match self.panel {
                Panel::Guide => self.render_guide_ui(),
                Panel::Soon => self.render_soon(),
                Panel::Entry(i) => self.render_entry_ui(i),
                Panel::Missions => self.render_missions(),
                Panel::Help => self.render_help(),
                Panel::Intro(n) => self.render_intro(n),
                Panel::Done(m) => self.render_done(m),
                Panel::None => {}
            }
            self.render_photo();
            if self.photo.is_none() { self.render_notices(); }
            self.render_tip();
            let y = 4.0;
            let voice_h = self.top_h() - 8.0;
            self.add_button(8.0, y, 56.0, y + voice_h, if self.voice { "🔊" } else { "🔇" }, Act::Voice, true, self.voice);
            self.add_button(60.0, y, 108.0, y + voice_h, "↻", Act::ReplayVoice, true, false);
            self.gfx.d2d_end();
        }
        self.gfx.present();
    }

    fn add_button(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, label: &str, act: Act, enabled: bool, hot: bool) {
        let hover = self.mouse.0 >= x0 && self.mouse.0 <= x1 && self.mouse.1 >= y0 && self.mouse.1 <= y1 && !self.touch;
        let bg = if !enabled {
            [0.10, 0.14, 0.18, 0.85]
        } else if hot {
            if hover {
                [0.98, 0.76, 0.38, 1.0]
            } else {
                [0.92, 0.62, 0.24, 1.0]
            }
        } else if hover {
            [0.16, 0.36, 0.44, 0.98]
        } else {
            [0.08, 0.24, 0.32, 0.96]
        };
        let r = ((y1 - y0) * 0.3).min(10.0);
        self.gfx.fill_round(x0, y0, x1, y1, r, bg);
        self.gfx.stroke_round(x0, y0, x1, y1, r, 1.0, if hot { [1.0, 0.9, 0.7, 0.8] } else { EDGE });
        let tc = if !enabled {
            [0.5, 0.56, 0.6, 1.0]
        } else if hot {
            [0.12, 0.08, 0.04, 1.0]
        } else {
            TEXT
        };
        let f = if y1 - y0 >= 40.0 { Font::BigCenter } else { Font::NormalCenter };
        self.gfx.text(label, x0 + 4.0, y0, x1 - x0 - 8.0, y1 - y0, f, tc);
        self.buttons.push(Button { x0, y0, x1, y1, act, label: label.to_string(), enabled });
    }

    fn panel_box(&mut self, x0: f32, y0: f32, x1: f32, y1: f32) {
        self.gfx.fill_round(x0, y0, x1, y1, 12.0, BG);
        self.gfx.stroke_round(x0, y0, x1, y1, 12.0, 1.5, EDGE);
    }

    // ------------------------------------------------------------ 地図

    fn render_map_world(&mut self) {
        let (w, h) = (self.w(), self.h());
        let z = self.zoom;
        let sx = 2.0 * z / w;
        let sy = -2.0 * z / h;
        self.gfx.set_xform(sx, sy, -self.cam.0 * sx, -self.cam.1 * sy);
        let g = &mut self.gfx;
        g.draw_textured(&self.ocean_tex, 0.0, 0.0, MAP_W, MAP_H, [1.0, 1.0, 1.0, 1.0], true);
        // 経緯線（10度ごと）
        let lw = (1.2 / z).max(0.05);
        for k in 0..36 {
            let x = k as f32 * 10.0 * DEG;
            g.line((x, 0.0), (x, MAP_H), lw, [0.6, 0.85, 1.0, 0.07]);
        }
        for k in -7..=7 {
            let (_, y) = lonlat_to_map(0.0, k as f32 * 10.0);
            g.line((0.0, y), (MAP_W, y), lw, [0.6, 0.85, 1.0, if k == 0 { 0.16 } else { 0.07 }]);
        }
        g.flush();
        for (tex, r) in &self.map_tex {
            self.gfx.draw_textured(tex, r[0], r[1], r[2], r[3], [1.0, 1.0, 1.0, 1.0], true);
        }
        // 時間帯の色（早朝・夕方は色づき、夜は暗い）
        let tint = match self.prog.time {
            0 => Some([1.0, 0.72, 0.52, 0.12]),
            2 => Some([0.95, 0.42, 0.18, 0.20]),
            3 => Some([0.0, 0.02, 0.12, 0.50]),
            _ => None,
        };
        if let (Some(c), true) = (tint, self.screen != Screen::Title) {
            self.gfx.rect(0.0, 0.0, MAP_W, MAP_H, c);
        }
        let ms = 1.0 / z;
        // レーダーの光る場所
        if self.screen == Screen::Map && self.radar.is_some() {
            let pulse = (self.clock * 2.4).sin() * 0.5 + 0.5;
            let g = &mut self.gfx;
            for &(lon, lat) in &self.radar_cells {
                let (mx, my) = lonlat_to_map(lon, lat);
                g.ring(mx, my, (16.0 + pulse * 10.0) * ms, 3.0 * ms, 24, [1.0, 0.86, 0.3, 0.9 - pulse * 0.5]);
                g.radial(mx, my, 26.0 * ms, 16, [1.0, 0.86, 0.3, 0.35 + 0.25 * pulse], [1.0, 0.8, 0.3, 0.0]);
            }
        }
        // 雲（拡大すると消える）
        let ca = if self.screen == Screen::Title { 0.25 } else { (0.35 - z * 0.2).clamp(0.0, 0.35) };
        if ca > 0.01 {
            let off = (self.clock * 6.0) % MAP_W;
            let u = off / MAP_W;
            let cw = [1.0, 1.0, 1.0, ca];
            self.gfx.draw_textured_quad(&self.cloud_tex, [(0.0, 0.0), (MAP_W - off, 0.0), (MAP_W - off, MAP_H), (0.0, MAP_H)], [(u, 0.0), (1.0, 0.0), (1.0, 1.0), (u, 1.0)], cw, true);
            self.gfx.draw_textured_quad(&self.cloud_tex, [(MAP_W - off, 0.0), (MAP_W, 0.0), (MAP_W, MAP_H), (MAP_W - off, MAP_H)], [(0.0, 0.0), (u, 0.0), (u, 1.0), (0.0, 1.0)], cw, true);
        }
        if self.screen == Screen::Title {
            return;
        }
        // 観察スポット
        let g = &mut self.gfx;
        for (i, s) in SPOTS.iter().enumerate() {
            let (mx, my) = lonlat_to_map(s.lon, s.lat);
            let p = (self.clock * 2.0 + i as f32).sin() * 0.5 + 0.5;
            g.ring(mx, my, (10.0 + p * 4.0) * ms, 2.0 * ms, 24, [0.75, 1.0, 0.55, 0.9]);
            g.circle(mx, my, 5.0 * ms, 14, [0.95, 1.0, 0.75, 1.0]);
            g.circle(mx, my, 2.5 * ms, 10, [0.30, 0.60, 0.20, 1.0]);
        }
        // カプセル
        if let Some(tg) = self.cap_target {
            g.line(self.cap, tg, 1.5 * ms, [1.0, 0.8, 0.5, 0.6]);
            g.ring(tg.0, tg.1, 7.0 * ms, 1.5 * ms, 16, [1.0, 0.8, 0.5, 0.7]);
        }
        let (cx, cy) = self.cap;
        let p = (self.clock * 3.0).sin() * 0.5 + 0.5;
        g.radial(cx, cy, 26.0 * ms, 24, [1.0, 0.8, 0.4, 0.35], [1.0, 0.7, 0.3, 0.0]);
        g.ring(cx, cy, (14.0 + p * 6.0) * ms, 1.5 * ms, 24, [1.0, 0.85, 0.5, 0.8 - p * 0.5]);
        g.circle(cx, cy - 1.0 * ms, 7.0 * ms, 16, [0.88, 0.9, 0.92, 1.0]);
        g.circle(cx + 1.5 * ms, cy - 2.0 * ms, 4.0 * ms, 12, [0.3, 0.7, 0.85, 1.0]);
        g.line((cx - 7.0 * ms, cy), (cx + 7.0 * ms, cy), 1.6 * ms, [0.98, 0.62, 0.2, 1.0]);
    }

    fn loc_rect(&self) -> (f32, f32, f32, f32) {
        let h = self.h();
        let (lw, lh) = if self.compact() { (310.0, 170.0) } else { (370.0, 190.0) };
        (10.0, h - lh - 10.0, 10.0 + lw, h - 10.0)
    }

    fn render_map_ui(&mut self) {
        let z = self.zoom;
        // 地方・国の名前（引いて見たとき）
        let labels: [(f32, f32, &str, bool); 10] = [
            (139.6, 36.2, t!("関東地方", "Kanto", "关东地区"), true),
            (142.8, 43.4, t!("北海道（準備中）", "Hokkaido (coming soon)", "北海道（准备中）"), false),
            (140.6, 39.3, t!("東北（準備中）", "Tohoku (coming soon)", "东北（准备中）"), false),
            (137.6, 36.3, t!("中部（準備中）", "Chubu (coming soon)", "中部（准备中）"), false),
            (135.6, 34.7, t!("近畿（準備中）", "Kinki (coming soon)", "近畿（准备中）"), false),
            (132.6, 34.9, t!("中国・四国（準備中）", "Chugoku & Shikoku (coming soon)", "中国·四国（准备中）"), false),
            (130.8, 32.4, t!("九州・沖縄（準備中）", "Kyushu & Okinawa (coming soon)", "九州·冲绳（准备中）"), false),
            (110.0, 33.0, t!("アジア（準備中）", "Asia (coming soon)", "亚洲（准备中）"), false),
            (-100.0, 42.0, t!("北アメリカ（準備中）", "North America (coming soon)", "北美洲（准备中）"), false),
            (134.0, -24.0, t!("オーストラリア（準備中）", "Australia (coming soon)", "澳大利亚（准备中）"), false),
        ];
        let kz = self.kanto_zoom();
        for (lon, lat, s, kanto) in labels {
            let a = if kanto { (2.2 - z / kz).clamp(0.0, 1.0) } else { (1.2 - z / kz * 1.5).clamp(0.0, 1.0) * if lat.abs() > 0.0 && (z > kz * 0.1 || lon > 120.0 && lon < 150.0) { 1.0 } else { 0.0 } };
            let a = if !kanto && (lon < 120.0 || lon > 150.0 || lat < 24.0) { (0.6 - z / kz * 8.0).clamp(0.0, 0.8) } else { a };
            if a <= 0.01 {
                continue;
            }
            let (mx, my) = lonlat_to_map(lon, lat);
            let (sx, sy) = self.map_to_screen(mx, my);
            let col = if kanto { [1.0, 0.98, 0.80, a] } else { [0.88, 0.90, 0.92, a * 0.9] };
            self.gfx.text_shadow(s, sx - 140.0, sy - 12.0, 280.0, 24.0, if kanto { Font::BigCenter } else { Font::NormalCenter }, col);
        }
        // 観察スポットの名前
        let (mx, my) = self.mouse;
        let show_all = z > kz * 0.9;
        let here = self.at_spot();
        // いる場所・マウスの下を先に、重ならないものだけ名前を出す
        let mut order: Vec<usize> = (0..SPOTS.len()).collect();
        order.sort_by_key(|&i| {
            let (px, py) = lonlat_to_map(SPOTS[i].lon, SPOTS[i].lat);
            let (sx, sy) = self.map_to_screen(px, py);
            let hover = (mx - sx).abs() < 16.0 && (my - sy).abs() < 16.0;
            if hover { 0 } else if Some(i) == here { 1 } else { 2 }
        });
        let mut placed: Vec<[f32; 4]> = Vec::new();
        for i in order {
            let s = &SPOTS[i];
            let (px, py) = lonlat_to_map(s.lon, s.lat);
            let (sx, sy) = self.map_to_screen(px, py);
            let hover = (mx - sx).abs() < 16.0 && (my - sy).abs() < 16.0;
            let wd = self.gfx.measure(s.name.get(), Font::Small) + 12.0;
            let rect = [sx + 12.0, sy - 10.0, sx + 12.0 + wd, sy + 8.0];
            let clash = placed.iter().any(|r| rect[0] < r[2] && rect[2] > r[0] && rect[1] < r[3] && rect[3] > r[1]);
            if (show_all && !clash) || hover {
                placed.push(rect);
                let label = s.name.get();
                self.gfx.fill_round(sx + 12.0, sy - 10.0, sx + 12.0 + wd, sy + 8.0, 6.0, [0.03, 0.12, 0.06, 0.78]);
                self.gfx.text(label, sx + 18.0, sy - 9.0, wd, 16.0, Font::Small, if hover { GOLD } else { [0.85, 1.0, 0.80, 1.0] });
            }
            if hover {
                self.tip = Some(format!("{}（{}）\n{}", s.name.get(), s.area.get(), s.habs.iter().map(|h| h.name()).collect::<Vec<_>>().join(lang::sep())));
            }
        }
        self.render_loc_card();
        self.render_radar_card();
        self.render_month_bar();
        // 右下：地図の操作
        let (w, h) = (self.w(), self.h());
        let bw = if self.compact() { 40.0 } else { 44.0 };
        let y = h - bw - 10.0;
        self.add_button(w - bw - 10.0, y, w - 10.0, y + bw, "＋", Act::MapZoom(1), true, false);
        self.add_button(w - bw * 2.0 - 16.0, y, w - bw - 16.0, y + bw, "－", Act::MapZoom(-1), true, false);
        self.add_button(w - bw * 2.0 - 132.0, y, w - bw * 2.0 - 22.0, y + bw, t!("関東を見る", "Show Kanto", "查看关东"), Act::ShowKanto, true, false);
    }

    /// 月の帯（季節を選ぶ）。広い画面では上の真ん中、せまい画面では左下のカードの上
    fn month_rect(&self) -> (f32, f32, f32, f32) {
        let w = self.w();
        if w >= 1000.0 {
            let (x0, x1) = (350.0, w - 320.0);
            let cw = (x1 - x0).min(640.0);
            let cx = (x0 + x1) * 0.5;
            (cx - cw * 0.5, self.top_h() + 8.0, cx + cw * 0.5, self.top_h() + 108.0)
        } else {
            let (_, ly0, _, _) = self.loc_rect();
            (10.0, ly0 - 110.0, w - 10.0, ly0 - 10.0)
        }
    }

    fn render_month_bar(&mut self) {
        let (x0, y0, x1, y1) = self.month_rect();
        self.panel_box(x0, y0, x1, y1);
        let month = self.prog.month;
        let title = tf!("{}　{}・{}", "{}  {} · {}", "{}　{}·{}", month_name(month), season_name(month), time_name(self.prog.time));
        self.gfx.text(&title, x0 + 12.0, y0 + 4.0, 200.0, 20.0, Font::Normal, TEXT);
        let hint = t!("月と時間帯を選ぶ（, . と T）", "Pick a month and time of day (, . and T)", "选择月份和时间段（, . 和 T）");
        self.gfx.text(hint, x0 + 150.0, y0 + 6.0, x1 - x0 - 160.0, 16.0, Font::Small, DIM);
        let bw = 30.0;
        self.add_button(x0 + 6.0, y0 + 26.0, x0 + 6.0 + bw, y0 + 60.0, "◀", Act::StepMonth(-1), true, false);
        self.add_button(x1 - 6.0 - bw, y0 + 26.0, x1 - 6.0, y0 + 60.0, "▶", Act::StepMonth(1), true, false);
        let (cx0, cx1) = (x0 + 10.0 + bw, x1 - 10.0 - bw);
        let cw = (cx1 - cx0) / 12.0;
        let cols = [[0.55, 0.80, 0.45], [0.30, 0.66, 0.40], [0.86, 0.56, 0.22], [0.62, 0.72, 0.86]];
        let (mx, my) = self.mouse;
        for m in 1..=12u8 {
            let a = cx0 + (m - 1) as f32 * cw;
            let b = a + cw - 2.0;
            let (ty0, ty1) = (y0 + 26.0, y0 + 60.0);
            let c = cols[season_of(m)];
            let cur = m == month;
            self.gfx.fill_round(a, ty0, b, ty1, 5.0, if cur { [c[0], c[1], c[2], 1.0] } else { [c[0] * 0.45, c[1] * 0.45, c[2] * 0.45, 0.95] });
            if cur {
                self.gfx.stroke_round(a, ty0, b, ty1, 5.0, 2.0, [1.0, 1.0, 1.0, 0.95]);
            }
            let label = if self.compact() || cw < 44.0 { format!("{}", m) } else { month_name(m) };
            self.gfx.text(&label, a, ty0 + 4.0, b - a, ty1 - ty0 - 4.0, Font::NormalCenter, if cur { [0.05, 0.08, 0.05, 1.0] } else { TEXT });
            // レーダーで探している虫がいる月
            if let Some(e) = self.radar {
                if SPECIES[e].in_season(m) {
                    self.gfx.fill_rect(a + 2.0, ty1 - 4.0, b - 2.0, ty1 - 1.0, GOLD);
                }
            }
            if mx >= a && mx <= b && my >= ty0 && my <= ty1 {
                let n = SPECIES.iter().filter(|s| s.in_season(m)).count();
                self.tip = Some(tf!("{}（{}）\n見られる虫：{}種", "{} ({})\nInsects about: {}", "{}（{}）\n能看到的昆虫：{}种", month_name(m), season_name(m), n));
            }
            self.buttons.push(Button { x0: a, y0: ty0, x1: b, y1: ty1, act: Act::Month(m), label: month_name(m), enabled: true });
        }
        // 時間帯（早朝・昼・夕方・夜）
        let tw = (x1 - x0 - 12.0) / 4.0;
        let tcols = [[0.98, 0.72, 0.56], [0.55, 0.78, 0.98], [0.95, 0.52, 0.28], [0.16, 0.20, 0.42]];
        for t in 0..4u8 {
            let a = x0 + 6.0 + t as f32 * tw;
            let b = a + tw - 4.0;
            let (ty0, ty1) = (y0 + 66.0, y1 - 6.0);
            let c = tcols[t as usize];
            let cur = t == self.prog.time;
            self.gfx.fill_round(a, ty0, b, ty1, 5.0, if cur { [c[0], c[1], c[2], 1.0] } else { [c[0] * 0.4, c[1] * 0.4, c[2] * 0.4, 0.95] });
            if cur {
                self.gfx.stroke_round(a, ty0, b, ty1, 5.0, 2.0, [1.0, 1.0, 1.0, 0.95]);
            }
            let icon = ["🌅", "☀", "🌇", "🌙"][t as usize];
            let label = format!("{} {}", icon, time_name(t));
            let tc = if cur && t != 3 { [0.05, 0.08, 0.05, 1.0] } else { TEXT };
            self.gfx.text(&label, a, ty0 + 3.0, b - a, ty1 - ty0 - 3.0, Font::NormalCenter, tc);
            if let Some(e) = self.radar {
                if SPECIES[e].active_at(t) {
                    self.gfx.fill_rect(a + 2.0, ty1 - 4.0, b - 2.0, ty1 - 1.0, GOLD);
                }
            }
            if self.mouse.0 >= a && self.mouse.0 <= b && self.mouse.1 >= ty0 && self.mouse.1 <= ty1 {
                let n = SPECIES.iter().filter(|s| s.in_season(month) && s.active_at(t)).count();
                self.tip = Some(tf!("{}の{}\n見られる虫：{}種", "{} in {}\nInsects about: {}", "{}的{}\n能看到的昆虫：{}种", month_name(month), time_name(t), n));
            }
            self.buttons.push(Button { x0: a, y0: ty0, x1: b, y1: ty1, act: Act::TimeOfDay(t), label: time_name(t).to_string(), enabled: true });
        }
    }

    /// 地図のレーダーのカード：何を探しているか、どこにいるか
    fn render_radar_card(&mut self) {
        let Some(e) = self.radar else { return };
        let cw = if self.compact() { 280.0 } else { 330.0 };
        let (x0, y0) = (10.0, self.top_h() + 8.0);
        let (x1, y1) = (x0 + cw, y0 + 128.0);
        self.panel_box(x0, y0, x1, y1);
        let pulse = (self.clock * 3.0).sin() * 0.5 + 0.5;
        self.gfx.stroke_circle(x0 + 22.0, y0 + 20.0, 8.0 + pulse * 3.0, 2.0, [1.0, 0.86, 0.35, 0.9]);
        self.gfx.fill_circle(x0 + 22.0, y0 + 20.0, 3.5, [1.0, 0.9, 0.5, 1.0]);
        let logged = if self.prog.stars[e] > 0 { t!("収録済み", "logged", "已收录") } else { t!("未収録", "not logged", "未收录") };
        self.gfx.text(&tf!("レーダー　{}", "Radar  {}", "雷达　{}", logged), x0 + 38.0, y0 + 4.0, cw - 90.0, 16.0, Font::Small, GOLD);
        self.gfx.text(entry_name(e), x0 + 38.0, y0 + 20.0, cw - 90.0, 20.0, Font::Normal, TEXT);
        self.add_button(x1 - 44.0, y0 + 6.0, x1 - 8.0, y0 + 32.0, "×", Act::RadarOff, true, false);
        let month = self.prog.month;
        let time = self.prog.time;
        let here = self.at_spot().filter(|&s| SPOTS[s].habs.iter().any(|&h| lives_here(&SPECIES[e], s, h, month, time)));
        let (line, go) = if SPECIES[e].in_season(month) && !SPECIES[e].active_at(time) {
            (tf!("{}にはいない。{}に見られる", "Not about in the {}. Seen: {}", "{}看不到。{}能看到", time_name(time), times_text(SPECIES[e].times)), Some(t!("その時間へ行く", "Go to that time", "前往那个时间")))
        } else if !SPECIES[e].in_season(month) {
            (tf!("{}にはいない。{}に見られる", "Not around in {}. Seen {}", "{}看不到。{}能看到", month_name(month), months_text(SPECIES[e].months)), Some(t!("その季節へ行く", "Go to its season", "前往那个季节")))
        } else if self.cap_target.is_some() {
            (t!("移動中…　着いたら降りよう（Enter）", "On the way… land when you arrive (Enter)", "移动中…　到达后降落（Enter）").to_string(), None)
        } else if let Some(s) = here {
            let hs: Vec<&str> = SPOTS[s].habs.iter().filter(|&&h| lives_here(&SPECIES[e], s, h, month, time)).map(|h| h.name()).collect();
            (tf!("ここの「{}」にいる！", "Here, in the {}!", "就在这里的{}！", hs.join(lang::sep())), None)
        } else {
            (tf!("光っているスポットにいる（{}か所）", "Found at the glowing spots ({})", "在发光的观察点（{}处）", self.radar_cells.len()), Some(t!("いちばん近い所へ", "Go to the nearest", "前往最近处")))
        };
        self.gfx.text(&line, x0 + 12.0, y0 + 46.0, cw - 24.0, 20.0, Font::Small, [0.8, 1.0, 0.9, 1.0]);
        if let Some(label) = go {
            self.add_button(x0 + 12.0, y1 - 44.0, x0 + 200.0, y1 - 10.0, label, Act::RadarGo, true, true);
        }
        self.add_button(x1 - 104.0, y1 - 44.0, x1 - 10.0, y1 - 10.0, t!("図鑑で見る", "Guide", "看图鉴"), Act::Entry(e), true, false);
    }

    fn render_dive_radar(&mut self) {
        let Some(sc) = &self.scene else { return };
        let (w, h) = (self.w(), self.h());
        let r = if self.compact() { 48.0 } else { 58.0 };
        let shutter = if self.compact() { 38.0 } else { 42.0 };
        let (cx, cy) = (w - r - 22.0, h - shutter * 2.0 - 44.0 - r);
        let range = 14.0;
        self.gfx.fill_circle(cx, cy, r + 4.0, [0.02, 0.08, 0.12, 0.82]);
        self.gfx.stroke_circle(cx, cy, r, 1.5, [0.45, 0.95, 0.8, 0.7]);
        self.gfx.stroke_circle(cx, cy, r * 0.5, 1.0, [0.45, 0.95, 0.8, 0.35]);
        self.gfx.fill_rect(cx - r, cy - 0.5, cx + r, cy + 0.5, [0.45, 0.95, 0.8, 0.25]);
        self.gfx.fill_rect(cx - 0.5, cy - r, cx + 0.5, cy + r, [0.45, 0.95, 0.8, 0.25]);
        // 回る走査線（点で描く）
        let sweep = self.clock * 2.2;
        for k in 0..10 {
            let f = k as f32 / 10.0;
            self.gfx.fill_circle(cx + sweep.cos() * r * f, cy + sweep.sin() * r * f, 1.4, [0.5, 1.0, 0.85, 0.6]);
        }
        let target = self.radar.filter(|&e| e < n_species());
        let mut best: Option<(f32, f32, f32)> = None;
        let mut blips: Vec<(f32, f32, [f32; 4], f32)> = Vec::new();
        for c in &sc.creatures {
            let (dx, dy) = (c.x - sc.cap.x, c.y - sc.cap.y);
            let d = (dx * dx + dy * dy).sqrt();
            if Some(c.sp) == target && best.map_or(true, |b| d < b.0) {
                best = Some((d, c.x, c.y));
            }
            if d > range {
                continue;
            }
            let (bx, by) = (cx + dx / range * r, cy + dy / range * r);
            if Some(c.sp) == target {
                blips.push((bx, by, [1.0, 0.85, 0.3, 1.0], 4.5));
            } else if self.prog.stars[c.sp] == 0 {
                blips.push((bx, by, [0.4, 0.95, 1.0, 0.95], 3.0));
            } else {
                blips.push((bx, by, [0.75, 0.8, 0.8, 0.45], 2.2));
            }
        }
        // 探しているものを最後に（上に）描く
        blips.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap());
        for (bx, by, c, s) in blips {
            self.gfx.fill_circle(bx, by, s, c);
        }
        self.gfx.fill_circle(cx, cy, 3.0, [1.0, 1.0, 1.0, 0.95]);
        let missing = sc.present.iter().filter(|&&i| self.prog.stars[i] == 0).count();
        let label = tf!("レーダー　未収録{}種", "Radar  {} new here", "雷达　未收录{}种", missing);
        self.gfx.text(&label, cx - r - 30.0, cy - r - 24.0, r * 2.0 + 30.0, 18.0, Font::Small, [0.6, 1.0, 0.9, 0.95]);
        // 探しているもの
        let Some(e) = self.radar else { return };
        let v = self.dive_view();
        let top = self.top_h();
        match best {
            None => {
                let s = tf!("{}：ここにはいない", "{}: not here", "{}：这里没有", entry_name(e));
                self.gfx.text(&s, cx - r - 120.0, cy + r + 6.0, r * 2.0 + 120.0, 18.0, Font::Small, DIM);
            }
            Some((d, x, y)) => {
                let s = tf!("{}　{:.0}m", "{}  {:.0} m", "{}　{:.0}米", entry_name(e), d);
                self.gfx.text(&s, cx - r - 120.0, cy + r + 6.0, r * 2.0 + 120.0, 18.0, Font::Small, GOLD);
                // 画面の外なら、端に矢印
                let (sx, sy) = ((x - v.cx) * v.ppm + w * 0.5, (y - v.cy) * v.ppm + h * 0.5);
                let inside = sx > 30.0 && sx < w - 30.0 && sy > top + 30.0 && sy < h - 30.0;
                if inside {
                    self.gfx.stroke_circle(sx, sy, 26.0 + (self.clock * 5.0).sin() * 4.0, 2.5, [1.0, 0.86, 0.3, 0.9]);
                } else {
                    let (mx, my) = (w * 0.5, (top + h) * 0.5);
                    let (ax, ay) = (sx - mx, sy - my);
                    let l = (ax * ax + ay * ay).sqrt().max(1.0);
                    let ex = sx.clamp(40.0, w - 40.0);
                    // 左上の情報のカード・右下のレーダーとシャッターに重ならないように
                    let bottom = if ex < 330.0 || ex > w - 160.0 { h - 175.0 } else { h - 40.0 };
                    let ey = sy.clamp(top + 40.0, bottom);
                    self.gfx.fill_circle(ex, ey, 17.0, [1.0, 0.84, 0.3, 0.95]);
                    // 矢印の向きを、中心から外へ向かう点で示す
                    for k in 0..4 {
                        let f = 6.0 + k as f32 * 4.0;
                        self.gfx.fill_circle(ex + ax / l * f, ey + ay / l * f, 3.2 - k as f32 * 0.5, [0.25, 0.12, 0.0, 1.0]);
                    }
                }
            }
        }
    }

    fn render_loc_card(&mut self) {
        let (x0, y0, x1, y1) = self.loc_rect();
        self.panel_box(x0, y0, x1, y1);
        let tw = x1 - x0 - 24.0;
        let Some(s) = self.at_spot() else {
            let (title, body) = if self.cap_target.is_some() {
                (t!("移動中…", "On the way…", "移动中…"), t!("観察スポットに着いたら、降りる場所を選べます。", "When you reach the spot, you can choose where to land.", "到达观察点后，可以选择降落地点。"))
            } else {
                (t!("関東地方", "Kanto", "关东地区"), t!("地図の光る点が観察スポット。押すとカプセルが飛んでいくよ。関東の外は、いま準備中です。", "The glowing dots are observation spots. Press one to fly there. Areas outside Kanto are coming soon.", "地图上发光的点是观察点。点击后观测舱就会飞过去。关东以外的地区正在准备中。"))
            };
            self.gfx.text(title, x0 + 12.0, y0 + 8.0, tw, 24.0, Font::Big, TEXT);
            let hh = self.gfx.measure_h(body, Font::Normal, tw);
            self.gfx.text(body, x0 + 12.0, y0 + 40.0, tw, hh, Font::Normal, DIM);
            let bh = 34.0;
            let by = y1 - bh - 10.0;
            let half = (x1 - x0 - 30.0) / 2.0;
            self.add_button(x0 + 10.0, by, x0 + 10.0 + half, by + bh, t!("◀ 前のスポット", "◀ Prev spot", "◀ 上一个"), Act::StepSpot(-1), true, false);
            self.add_button(x0 + 20.0 + half, by, x1 - 10.0, by + bh, t!("次のスポット ▶", "Next spot ▶", "下一个 ▶"), Act::StepSpot(1), true, false);
            return;
        };
        let spot = &SPOTS[s];
        self.gfx.text(spot.name.get(), x0 + 12.0, y0 + 6.0, tw, 24.0, Font::Big, TEXT);
        self.gfx.text(spot.area.get(), x0 + 12.0, y0 + 32.0, tw, 18.0, Font::Small, ACCENT);
        let desc = spot.desc.get();
        let n = spot.habs.len();
        let bh = if self.compact() { 32.0 } else { 34.0 };
        let rows = (n + 1) / 2;
        let by0 = y1 - 10.0 - rows as f32 * (bh + 6.0) + 6.0;
        let dh = (by0 - y0 - 56.0).max(18.0);
        self.gfx.text(desc, x0 + 12.0, y0 + 52.0, tw, dh, Font::Small, DIM);
        let half = (x1 - x0 - 30.0) / 2.0;
        for (k, &hb) in spot.habs.iter().enumerate() {
            let bx = x0 + 10.0 + (k % 2) as f32 * (half + 10.0);
            let by = by0 + (k / 2) as f32 * (bh + 6.0);
            let hot = self.pending_hab == Some(hb) || (self.pending_hab.is_none() && k == 0);
            let label = if hot { format!("{} (Enter)", dive_label(hb)) } else { dive_label(hb) };
            self.add_button(bx, by, bx + half, by + bh, &label, Act::Dive(hb), true, hot);
        }
    }

    // ------------------------------------------------------------ 上の帯・宿題のカード

    fn render_top(&mut self) {
        let w = self.w();
        let th = self.top_h();
        self.gfx.fill_rect(0.0, 0.0, w, th, [0.03, 0.10, 0.06, 0.88]);
        self.gfx.fill_rect(0.0, th - 1.5, w, th, [0.55, 0.85, 0.50, 0.4]);
        let (lon, lat) = map_to_lonlat(self.cap.0, self.cap.1);
        let (small, big) = if let Some(sc) = &self.scene {
            (SPOTS[sc.spot].name.get().to_string(), sc.hab.name().to_string())
        } else if let Some(s) = self.at_spot() {
            (SPOTS[s].area.get().to_string(), SPOTS[s].name.get().to_string())
        } else {
            let ns = if lat >= 0.0 { t!("北緯", "N", "北纬") } else { t!("南緯", "S", "南纬") };
            let ew = if lon >= 0.0 { t!("東経", "E", "东经") } else { t!("西経", "W", "西经") };
            (format!("{}{:.1}° {}{:.1}°", ns, lat.abs(), ew, lon.abs()), t!("関東地方", "Kanto", "关东地区").to_string())
        };
        let small = format!("{}（{}）・{}・{}", month_name(self.prog.month), season_name(self.prog.month), time_name(self.prog.time), small);
        self.gfx.text(&small, 120.0, 4.0, 300.0, 18.0, Font::Small, DIM);
        self.gfx.text(&big, 120.0, 18.0, 260.0, 24.0, Font::Big, TEXT);
        // 右のボタン
        let bh = th - 12.0;
        let mut x = w - 8.0;
        let items: Vec<(&str, Act, f32)> = vec![
            (t!("タイトル", "Title", "标题"), Act::ToTitle, 76.0),
            ("⛶", Act::Fullscreen, 38.0),
            (lang::lang().native(), Act::Lang, 74.0),
            (t!("？ 操作", "? Help", "？ 操作"), Act::Help, 72.0),
        ];
        for (label, act, bw) in items {
            self.add_button(x - bw, 6.0, x, 6.0 + bh, label, act, true, false);
            x -= bw + 6.0;
        }
        let fresh = self.prog.fresh.iter().filter(|&&f| f).count();
        let gl = if fresh > 0 { tf!("図鑑 {}/{} ●", "Guide {}/{} ●", "图鉴 {}/{} ●", self.prog.found(), n_entries()) } else { tf!("図鑑 {}/{}", "Guide {}/{}", "图鉴 {}/{}", self.prog.found(), n_entries()) };
        let gw = if self.compact() { 110.0 } else { 130.0 };
        self.add_button(x - gw, 6.0, x, 6.0 + bh, &gl, Act::Guide, true, fresh > 0);
        x -= gw + 6.0;
        self.add_button(x - 84.0, 6.0, x, 6.0 + bh, t!("宿題 F1", "Tasks F1", "作业 F1"), Act::Missions, true, false);
    }

    fn mission_rect(&self) -> Option<(f32, f32, f32, f32)> {
        if self.screen == Screen::Title || self.panel != Panel::None {
            return None;
        }
        let w = self.w();
        let cw = if self.compact() { 260.0 } else { 300.0 };
        let y0 = self.top_h() + 8.0;
        if !self.mission_open || self.prog.all_clear() {
            return Some((w - cw - 10.0, y0, w - 10.0, y0 + 34.0));
        }
        let n = MISSIONS[self.prog.mission].targets.len() as f32;
        let rh = if self.compact() { 26.0 } else { 28.0 };
        Some((w - cw - 10.0, y0, w - 10.0, y0 + 40.0 + n * rh + 8.0))
    }

    fn render_mission_card(&mut self) {
        let Some((x0, y0, x1, y1)) = self.mission_rect() else { return };
        self.panel_box(x0, y0, x1, y1);
        if self.prog.all_clear() {
            self.gfx.text(t!("宿題：すべて合格！", "All tasks passed!", "作业：全部合格！"), x0 + 12.0, y0 + 7.0, x1 - x0 - 24.0, 20.0, Font::Normal, GOLD);
            return;
        }
        let mi = self.prog.mission;
        let m = &MISSIONS[mi];
        let targets = m.target_list();
        let done_n = targets.iter().filter(|&&(t, n)| self.prog.stars[t.entry()] >= n).count();
        let passed = self.prog.is_done(mi);
        let head = if passed {
            tf!("✔ 宿題{} {}", "✔ Task {} {}", "✔ 作业{} {}", mi + 1, m.title.get())
        } else {
            tf!("宿題{} {}（{}/{}）", "Task {} {} ({}/{})", "作业{} {}（{}/{}）", mi + 1, m.title.get(), done_n, targets.len())
        };
        self.gfx.text(&head, x0 + 12.0, y0 + 8.0, x1 - x0 - 112.0, 20.0, Font::Normal, if passed { GOOD } else { ACCENT });
        let open = self.mission_open;
        // まだ合格していない、前・次の宿題へ
        self.add_button(x1 - 96.0, y0 + 5.0, x1 - 68.0, y0 + 29.0, "◀", Act::CycleMission(-1), true, false);
        self.add_button(x1 - 66.0, y0 + 5.0, x1 - 38.0, y0 + 29.0, "▶", Act::CycleMission(1), true, false);
        self.add_button(x1 - 34.0, y0 + 5.0, x1 - 8.0, y0 + 29.0, if open { "－" } else { "＋" }, Act::MissionCard, true, false);
        if !open {
            return;
        }
        let rh = if self.compact() { 26.0 } else { 28.0 };
        let mut y = y0 + 38.0;
        let present: Vec<usize> = self.scene.as_ref().map(|s| s.present.clone()).unwrap_or_default();
        for &(tg, need) in &targets {
            let have = self.prog.stars[tg.entry()];
            let ok = have >= need;
            let mark = if ok { "✔" } else { "○" };
            let here = matches!(tg, Target::Sp(i) if present.contains(&i)) && !ok;
            let col = if ok { GOOD } else if here { GOLD } else { TEXT };
            let _ = need;
            let need_s = String::new();
            let label = format!("{} {}{}", mark, tg.name(), need_s);
            self.gfx.text(&label, x0 + 12.0, y + 3.0, x1 - x0 - 90.0, 20.0, Font::Normal, col);
            if here {
                self.gfx.text(t!("ここにいる！", "Here!", "就在这里！"), x1 - 120.0, y + 4.0, 70.0, 18.0, Font::Small, GOLD);
            }
            if !ok {
                self.add_button(x1 - 52.0, y, x1 - 10.0, y + rh - 4.0, t!("行く", "Go", "前往"), Act::Goto(tg), true, false);
            }
            y += rh;
        }
    }

    // ------------------------------------------------------------ 観察の画面

    fn render_dive_ui(&mut self) {
        let (w, h) = (self.w(), self.h());
        let Some(sc) = &self.scene else { return };
        let lens = self.lens;
        let depth_s = sc.depth_text(sc.cap.y);
        let hab = sc.hab;
        let present = sc.present.clone();
        let have = present.iter().filter(|&&i| self.prog.stars[i] > 0).count();
        // 宿題の対象が画面の外にいれば、端に印を出す
        let v = self.dive_view();
        let mut marks: Vec<(f32, f32)> = Vec::new();
        if !self.prog.all_clear() {
            let m = &MISSIONS[self.prog.mission];
            for &(tg, need) in &m.target_list() {
                {
                    let Target::Sp(i) = tg;
                    if self.prog.stars[i] >= need {
                        continue;
                    }
                    let mut best: Option<(f32, f32, f32)> = None;
                    for c in &sc.creatures {
                        if c.sp != i {
                            continue;
                        }
                        let d = (c.x - sc.cap.x).powi(2) + (c.y - sc.cap.y).powi(2);
                        if best.map_or(true, |b| d < b.0) {
                            best = Some((d, c.x, c.y));
                        }
                    }
                    if let Some((_, x, y)) = best {
                        marks.push(((x - v.cx) * v.ppm + w * 0.5, (y - v.cy) * v.ppm + h * 0.5));
                    }
                }
            }
        }
        // 情報のカード
        let cw = if self.compact() { 250.0 } else { 290.0 };
        let ch = if self.compact() { 138.0 } else { 132.0 };
        let (x0, y1) = (10.0, h - 10.0);
        let y0 = y1 - ch;
        self.panel_box(x0, y0, x0 + cw, y1);
        self.gfx.text(&format!("{}　{}", hab.name(), depth_s), x0 + 12.0, y0 + 8.0, cw - 24.0, 20.0, Font::Normal, TEXT);
        if present.is_empty() {
            self.gfx.text(t!("この季節・時間帯、ここに虫はいない", "No insects here at this season and time", "这个季节和时间这里没有昆虫"), x0 + 12.0, y0 + 30.0, cw - 24.0, 18.0, Font::Small, GOLD);
        } else {
        self.gfx.text(&tf!("ここの虫：{}/{}種 収録済み", "Insects here: {}/{} logged", "此处昆虫：已收录 {}/{} 种", have, present.len()), x0 + 12.0, y0 + 30.0, cw - 24.0, 18.0, Font::Small, DIM);
        }
        let bh = 34.0;
        let by = y1 - bh - 10.0;
        let half = (cw - 30.0) / 3.0;
        let up = t!("地図へ Esc", "To map Esc", "回地图 Esc");
        self.add_button(x0 + 10.0, by, x0 + 10.0 + half, by + bh, up, Act::Surface, true, false);
        let light_s = if self.prog.light { t!("ライト ON", "Light ON", "灯 开") } else { t!("ライト L", "Light L", "灯 L") };
        let lt = self.prog.light;
        self.add_button(x0 + 15.0 + half, by, x0 + 15.0 + half * 2.0, by + bh, light_s, Act::Light, true, lt);
        self.add_button(x0 + 20.0 + half * 2.0, by, x0 + 20.0 + half * 3.0, by + bh, t!("加速 Shift", "Boost Shift", "加速 Shift"), Act::Boost, true, false);
        let by2 = by - bh - 6.0;
        self.gfx.text(&tf!("ズーム ×{:.1}", "Zoom ×{:.1}", "变焦 ×{:.1}", lens), x0 + 12.0, by2 + 8.0, 120.0, 18.0, Font::Normal, TEXT);
        self.add_button(x0 + cw - 92.0, by2, x0 + cw - 54.0, by2 + bh, "－", Act::Lens(-1), lens > 1.01, false);
        self.add_button(x0 + cw - 48.0, by2, x0 + cw - 10.0, by2 + bh, "＋", Act::Lens(1), lens < 7.99, false);
        // シャッター
        let r = if self.compact() { 38.0 } else { 42.0 };
        let (sx, sy) = (w - r - 22.0, h - r - 22.0);
        let hover = (self.mouse.0 - sx).powi(2) + (self.mouse.1 - sy).powi(2) < r * r;
        self.gfx.fill_circle(sx, sy, r + 6.0, [0.02, 0.08, 0.12, 0.8]);
        self.gfx.fill_circle(sx, sy, r, if hover { [1.0, 0.8, 0.45, 1.0] } else { [0.95, 0.66, 0.28, 1.0] });
        self.gfx.stroke_circle(sx, sy, r - 7.0, 2.0, [1.0, 1.0, 1.0, 0.85]);
        self.gfx.text(t!("撮影", "Shoot", "拍摄"), sx - r, sy - 12.0, r * 2.0, 24.0, Font::BigCenter, [0.1, 0.06, 0.02, 1.0]);
        self.buttons.push(Button { x0: sx - r, y0: sy - r, x1: sx + r, y1: sy + r, act: Act::Shoot, label: t!("撮影", "Photo", "拍摄").to_string(), enabled: true });
        // ファインダー
        if lens >= 1.8 {
            let m = 40.0;
            let l = 46.0;
            let (fx0, fy0, fx1, fy1) = (m, self.top_h() + m * 0.5, w - m, h - m);
            let c = [1.0, 1.0, 1.0, 0.85];
            for (x, y, dx, dy) in [(fx0, fy0, 1.0, 1.0), (fx1, fy0, -1.0, 1.0), (fx0, fy1, 1.0, -1.0), (fx1, fy1, -1.0, -1.0)] {
                self.gfx.fill_rect(x.min(x + dx * l), y - 1.5, x.max(x + dx * l), y + 1.5, c);
                self.gfx.fill_rect(x - 1.5, y.min(y + dy * l), x + 1.5, y.max(y + dy * l), c);
            }
            let (cx, cy) = (w * 0.5, h * 0.5);
            self.gfx.fill_rect(cx - 12.0, cy - 0.8, cx + 12.0, cy + 0.8, [1.0, 1.0, 1.0, 0.6]);
            self.gfx.fill_rect(cx - 0.8, cy - 12.0, cx + 0.8, cy + 12.0, [1.0, 1.0, 1.0, 0.6]);
            self.gfx.fill_circle(fx0 + 22.0, fy0 + 22.0, 6.0, [1.0, 0.3, 0.25, 0.5 + 0.5 * (self.clock * 3.0).sin().abs()]);
        }
        self.render_dive_radar();
        // 宿題の対象の印
        let top = self.top_h();
        for (mx, my) in marks {
            let inside = mx > 30.0 && mx < w - 30.0 && my > top + 30.0 && my < h - 30.0;
            if inside {
                self.gfx.stroke_circle(mx, my, 22.0 + (self.clock * 4.0).sin() * 3.0, 2.0, [1.0, 0.85, 0.35, 0.7]);
            } else {
                let ex = mx.clamp(26.0, w - 26.0);
                let ey = my.clamp(top + 26.0, h - 26.0);
                self.gfx.fill_circle(ex, ey, 14.0, [1.0, 0.82, 0.3, 0.9]);
                self.gfx.text("!", ex - 10.0, ey - 11.0, 20.0, 22.0, Font::BigCenter, [0.2, 0.1, 0.0, 1.0]);
            }
        }
    }

    // ------------------------------------------------------------ タイトル

    fn render_title(&mut self) {
        let (w, h) = (self.w(), self.h());
        self.gfx.fill_rect(0.0, 0.0, w, h, [0.0, 0.04, 0.02, 0.35]);
        let cw = if self.compact() { 420.0 } else { 520.0 };
        let x0 = if self.compact() { (w - cw) * 0.5 } else { w * 0.06 };
        let y0 = h * 0.16;
        self.gfx.fill_round(x0, y0, x0 + cw, y0 + 150.0, 18.0, [0.03, 0.10, 0.06, 0.78]);
        self.gfx.stroke_round(x0, y0, x0 + cw, y0 + 150.0, 18.0, 1.5, EDGE);
        self.gfx.text(t!("昆虫図鑑", "YYInsects", "昆虫图鉴"), x0, y0 + 14.0, cw, 56.0, Font::Title, [0.90, 1.0, 0.75, 1.0]);
        self.gfx.text(t!("ミクロ観察カプセルで虫をさがせ", "A Micro-Capsule Insect Field Guide", "乘微观观测舱寻找昆虫"), x0, y0 + 72.0, cw, 26.0, Font::BigCenter, TEXT);
        self.gfx.text(t!("第1巻　関東地方の昆虫", "Volume 1: Insects of Kanto", "第1卷　关东地区的昆虫"), x0, y0 + 104.0, cw, 24.0, Font::NormalCenter, ACCENT);
        let bw = if self.compact() { 300.0 } else { 340.0 };
        let bh = if self.compact() { 40.0 } else { 44.0 };
        let bx = x0 + (cw - bw) * 0.5;
        let mut by = y0 + 172.0;
        if self.have_save {
            self.add_button(bx, by, bx + bw, by + bh, t!("つづきから", "Continue", "继续"), Act::Continue, true, true);
            by += bh + 10.0;
        }
        let new_label = if self.confirm_new > 0.0 { t!("もう一度押すと、はじめから（記録は上書き）", "Press again to restart (overwrites save)", "再按一次重新开始（覆盖存档）") } else { t!("はじめから", "New journey", "从头开始") };
        let hot = !self.have_save;
        self.add_button(bx, by, bx + bw, by + bh, new_label, Act::NewGame, true, hot);
        by += bh + 10.0;
        let half = (bw - 10.0) / 2.0;
        self.add_button(bx, by, bx + half, by + 36.0, lang::lang().native(), Act::Lang, true, false);
        self.add_button(bx + half + 10.0, by, bx + bw, by + 36.0, t!("全画面", "Fullscreen", "全屏"), Act::Fullscreen, true, false);
        by += 46.0;
        self.add_button(bx, by, bx + half, by + 36.0, t!("記録ファイルを読み込む", "Load a save file", "读取存档文件"), Act::Import, true, false);
        self.add_button(bx + half + 10.0, by, bx + bw, by + 36.0, t!("操作説明", "Help", "操作说明"), Act::Help, true, false);
        let foot = t!(
            "夏休みの自由研究は「昆虫図鑑」づくり。虫と同じ大きさに縮むカプセルで、関東の公園や森をめぐろう。",
            "Your summer project: an insect field guide. Shrink down to insect size in the capsule and explore the parks and forests of Kanto.",
            "暑假的自由研究是制作“昆虫图鉴”。乘坐能缩小到昆虫大小的观测舱，探访关东的公园和森林吧。"
        );
        self.gfx.text_shadow(foot, 20.0, h - 34.0, w - 40.0, 24.0, Font::NormalCenter, [0.95, 0.98, 0.92, 0.95]);
    }

    // ------------------------------------------------------------ 図鑑

    fn guide_layout(&self) -> (f32, f32, f32, f32, usize, f32, f32) {
        let (w, h) = (self.w(), self.h());
        let x0 = 20.0;
        let (_, rows, _) = group_tabs(w - 40.0);
        let y0 = self.top_h() + 56.0 + rows as f32 * 40.0;
        let x1 = w - 20.0;
        let y1 = h - 16.0;
        let cell_w = if self.compact() { 112.0 } else { 132.0 };
        let cols = (((x1 - x0) + 10.0) / (cell_w + 10.0)).floor().max(2.0) as usize;
        let cw = ((x1 - x0) - (cols - 1) as f32 * 10.0) / cols as f32;
        let ch = cw * 0.86;
        (x0, y0, x1, y1, cols, cw, ch)
    }

    /// いま図鑑で開いているなかまの項目
    fn guide_entries(&self) -> Vec<usize> {
        (0..n_entries()).filter(|&e| entry_group(e) == GROUPS[self.guide_group]).collect()
    }

    fn render_guide_gl(&mut self) {
        let (w, h) = (self.w(), self.h());
        self.gfx.rect(0.0, 0.0, w, h, [0.02, 0.07, 0.11, 0.97]);
        let (x0, y0, _x1, y1, cols, cw, ch) = self.guide_layout();
        let list = self.guide_entries();
        let rows = (list.len() + cols - 1) / cols;
        self.guide_max = ((rows as f32) * (ch + 10.0) - (y1 - y0)).max(0.0);
        self.guide_scroll = self.guide_scroll.clamp(0.0, self.guide_max);
        for (k, &e) in list.iter().enumerate() {
            let (r, c) = (k / cols, k % cols);
            let cx0 = x0 + c as f32 * (cw + 10.0);
            let cy0 = y0 + r as f32 * (ch + 10.0) - self.guide_scroll;
            if cy0 + ch < y0 - 4.0 || cy0 > y1 {
                continue;
            }
            let found = self.prog.stars[e] > 0;
            let hover = self.mouse.0 >= cx0 && self.mouse.0 <= cx0 + cw && self.mouse.1 >= cy0 && self.mouse.1 <= cy0 + ch && !self.touch;
            let bg = if found { [0.10, 0.26, 0.34, 1.0] } else { [0.06, 0.12, 0.17, 1.0] };
            let bg = if hover { mix4(bg, [0.3, 0.5, 0.55, 1.0], 0.3) } else { bg };
            self.gfx.rect(cx0, cy0, cx0 + cw, cy0 + ch, bg);
            self.gfx.rect_outline(cx0, cy0, cx0 + cw, cy0 + ch, 1.0, if found { [0.45, 0.85, 0.80, 0.7] } else { [0.3, 0.4, 0.45, 0.6] });
            let a = if found { 1.0 } else { 0.16 };
            self.draw_entry_art(e, cx0 + 6.0, cy0 + 6.0, cx0 + cw - 6.0, cy0 + ch - 28.0, a);
            self.gfx.flush();
        }
        // 上下の余白を塗って、スクロールした項目を隠す
        self.gfx.rect(0.0, 0.0, w, y0 - 2.0, [0.02, 0.07, 0.11, 1.0]);
        self.gfx.rect(0.0, y1 + 2.0, w, h, [0.02, 0.07, 0.11, 1.0]);
    }

    /// 図鑑の絵を枠の中に描く
    fn draw_entry_art(&mut self, e: usize, x0: f32, y0: f32, x1: f32, y1: f32, a: f32) {
        if self.artwork.draw(&mut self.gfx, entry_slug(e), [x0, y0, x1, y1], a, Fit::Contain) {
            return;
        }
        let (bw, bh) = (x1 - x0, y1 - y0);
        let (ew, eh) = art::extent(e);
        let len = (bw * 0.86 / ew).min(bh * 0.86 / eh);
        let (x, y) = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);
        art::draw(&mut self.gfx, e, x, y, len, 1.0, 0.0, self.clock, a, 0.6, 7);
    }

    fn render_guide_ui(&mut self) {
        let w = self.w();
        let (x0, y0, _x1, y1, cols, cw, ch) = self.guide_layout();
        let title = tf!("関東の昆虫図鑑　{}/{}", "Kanto insect guide  {}/{}", "关东昆虫图鉴　{}/{}", self.prog.found(), n_entries());
        self.gfx.text(&title, 24.0, self.top_h() + 12.0, w - 160.0, 30.0, Font::Big, TEXT);
        self.add_button(w - 120.0, self.top_h() + 10.0, w - 20.0, self.top_h() + 44.0, t!("閉じる", "Close", "关闭"), Act::Close, true, false);
        let ty = self.top_h() + 52.0;
        let (tab_cols, _, tw) = group_tabs(w - 40.0);
        for (i, g) in GROUPS.iter().enumerate() {
            let ty = ty + (i / tab_cols) as f32 * 40.0;
            let bx = 20.0 + (i % tab_cols) as f32 * tw;
            let cur = i == self.guide_group;
            let all = SPECIES.iter().filter(|s| s.group == *g).count();
            let got = (0..n_entries()).filter(|&e| SPECIES[e].group == *g && self.prog.stars[e] > 0).count();
            self.add_button(bx, ty, bx + tw - 6.0, ty + 34.0, &format!("{} {}/{}", g.name(), got, all), Act::GuideTab(i), true, cur);
            if cur {
                self.gfx.fill_rect(bx + 6.0, ty + 30.0, bx + tw - 12.0, ty + 33.0, [1.0, 0.9, 0.7, 1.0]);
            }
        }
        let list = self.guide_entries();
        for (k, &e) in list.iter().enumerate() {
            let (r, c) = (k / cols, k % cols);
            let cx0 = x0 + c as f32 * (cw + 10.0);
            let cy0 = y0 + r as f32 * (ch + 10.0) - self.guide_scroll;
            if cy0 + ch < y0 || cy0 > y1 - 10.0 {
                continue;
            }
            let st = self.prog.stars[e];
            let name = if st > 0 { entry_name(e).to_string() } else { "？？？".to_string() };
            let vis_y0 = cy0.max(y0);
            let vis_y1 = (cy0 + ch).min(y1);
            if cy0 + ch - 26.0 >= y0 && cy0 + ch <= y1 + 1.0 {
                self.gfx.text(&name, cx0 + 4.0, cy0 + ch - 22.0, cw - 8.0, 16.0, Font::NormalCenter, if st > 0 { TEXT } else { DIM });
            }
            if self.prog.fresh[e] && cy0 >= y0 {
                self.gfx.fill_round(cx0 + 4.0, cy0 + 4.0, cx0 + 46.0, cy0 + 20.0, 6.0, ACCENT);
                self.gfx.text("NEW", cx0 + 4.0, cy0 + 4.0, 42.0, 16.0, Font::NormalCenter, [0.1, 0.05, 0.0, 1.0]);
            }
            if vis_y1 > vis_y0 + 4.0 {
                self.buttons.push(Button { x0: cx0, y0: vis_y0, x1: cx0 + cw, y1: vis_y1, act: Act::Entry(e), label: entry_name(e).to_string(), enabled: true });
            }
        }
    }

    fn entry_layout(&self) -> (f32, f32, f32, f32, f32) {
        let (w, h) = (self.w(), self.h());
        let x0 = 20.0;
        let y0 = self.top_h() + 56.0;
        let x1 = w - 20.0;
        let y1 = h - 16.0;
        let split = x0 + (x1 - x0) * 0.46;
        (x0, y0, x1, y1, split)
    }

    fn render_entry_gl(&mut self, e: usize) {
        let (w, h) = (self.w(), self.h());
        self.gfx.rect(0.0, 0.0, w, h, [0.03, 0.08, 0.05, 0.97]);
        let (x0, y0, _x1, y1, split) = self.entry_layout();
        let (ax0, ay0, ax1, ay1) = (x0, y0, split - 10.0, y1 - 50.0);
        // 背景（すみかの色）
        let sp = &SPECIES[e];
        if sp.beh == Beh::Swim || sp.beh == Beh::Skate {
            self.gfx.rect_vgrad(ax0, ay0, ax1, ay1, [0.36, 0.62, 0.56, 1.0], [0.10, 0.24, 0.20, 1.0]);
        } else {
            self.gfx.rect_vgrad(ax0, ay0, ax1, ay1, [0.50, 0.72, 0.90, 1.0], [0.82, 0.90, 0.80, 1.0]);
            let gy = ay1 - (ay1 - ay0) * 0.16;
            self.gfx.rect_vgrad(ax0, gy, ax1, ay1, [0.46, 0.60, 0.30, 1.0], [0.30, 0.36, 0.20, 1.0]);
        }
        let found = self.prog.stars[e] > 0;
        let pad = 20.0;
        self.draw_entry_art(e, ax0 + pad, ay0 + pad, ax1 - pad, ay1 - pad, if found { 1.0 } else { 0.14 });
        self.gfx.rect_outline(ax0, ay0, ax1, ay1, 1.5, [0.55, 0.85, 0.50, 0.6]);
    }

    fn render_entry_ui(&mut self, e: usize) {
        let w = self.w();
        let (_x0, y0, x1, y1, split) = self.entry_layout();
        let st = self.prog.stars[e];
        let found = st > 0;
        let top = self.top_h();
        if found && crate::artwork::find(entry_slug(e)).is_none() {
            self.gfx.text(t!("イラスト（大きさは実物とちがいます）", "Illustration (not to scale)", "插图（大小与实物不同）"), 30.0, y1 - 43.0, split - 50.0, 20.0, Font::NormalCenter, DIM);
        }
        self.gfx.text(&tf!("図鑑 No.{:02}　{}", "Guide No.{:02}  {}", "图鉴 No.{:02}　{}", e + 1, entry_group(e).name()), 24.0, top + 14.0, 420.0, 24.0, Font::Big, DIM);
        self.add_button(w - 120.0, top + 10.0, w - 20.0, top + 44.0, t!("一覧へ", "Back", "返回"), Act::Close, true, false);
        let list: Vec<usize> = (0..n_entries()).filter(|&x| entry_group(x) == entry_group(e)).collect();
        let k = list.iter().position(|&x| x == e).unwrap_or(0);
        let prev = list[(k + list.len() - 1) % list.len()];
        let next = list[(k + 1) % list.len()];
        self.add_button(w - 230.0, top + 10.0, w - 180.0, top + 44.0, "◀", Act::Entry(prev), true, false);
        self.add_button(w - 174.0, top + 10.0, w - 124.0, top + 44.0, "▶", Act::Entry(next), true, false);
        let (tx, tw) = (split + 14.0, x1 - split - 14.0);
        let mut y = y0;
        let name = if found { entry_name(e) } else { "？？？" };
        let name_h = self.gfx.measure_h(name, Font::TitleMid, tw).max(34.0);
        self.gfx.text(name, tx, y, tw, name_h, Font::TitleMid, TEXT);
        y += name_h + 8.0;
        let sp = &SPECIES[e];
        if found {
            let title_h = 26.0;
            let body_y = y + title_h + 8.0;
            let body_h = (y1 - 98.0 - body_y).max(28.0);
            let key = (e, lang::lang(), tw.to_bits(), body_h.to_bits());
            if self.entry_book.as_ref().map(|b| b.key) != Some(key) {
                let mut sections = crate::guide::sections(e, lang::lang());
                let metadata = format!("{} / {}・{}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n\n",
                    sp.sci, sp.group.order(), sp.family.get(),
                    t!("季節", "Season", "季节"), format!("{}　{}: {}", months_text(sp.months), t!("時間帯", "Time", "时间段"), times_text(sp.times)),
                    t!("すみか", "Habitat", "栖息地"), hab_text(sp),
                    t!("会える場所", "Where", "地点"), place_text(e),
                    t!("大きさ", "Size", "大小"), size_text(sp.len_cm));
                sections[0].body.insert_str(0, &metadata);
                self.entry_book = Some(EntryBook { key, pages: reading_pages(&mut self.gfx, &sections, tw, body_h) });
                self.entry_page = 0;
            }
            let book = self.entry_book.as_ref().unwrap();
            self.entry_page = self.entry_page.min(book.pages.len().saturating_sub(1));
            let count = book.pages.len();
            let (title, body) = &book.pages[self.entry_page];
            self.gfx.text(title, tx, y, tw, title_h, Font::Big, ACCENT);
            self.gfx.text(body, tx, body_y, tw, body_h, Font::Normal, TEXT);
            let by = y1 - 90.0;
            let bw = (tw * 0.32).min(150.0);
            self.add_button(tx, by, tx + bw, by + 34.0, t!("↑ 前のページ", "↑ Previous", "↑ 上一页"), Act::EntryPage(-1), self.entry_page > 0, false);
            self.add_button(x1 - bw, by, x1, by + 34.0, t!("次のページ ↓", "Next ↓", "下一页 ↓"), Act::EntryPage(1), self.entry_page + 1 < count, false);
            self.gfx.text(&format!("{} / {}", self.entry_page + 1, count), tx + bw + 4.0, by + 6.0, tw - 2.0 * bw - 8.0, 24.0, Font::NormalCenter, DIM);
            self.add_button(tx, y1 - 44.0, (tx + 220.0).min(x1), y1 - 6.0, t!("◎ レーダーで探す", "◎ Find with radar", "◎ 用雷达寻找"), Act::Radar(e), true, false);
        } else {
            for (label, value) in [
                (t!("季節", "Season", "季节"), months_text(sp.months)),
                (t!("時間帯", "Time", "时间段"), times_text(sp.times)),
                (t!("すみか", "Habitat", "栖息地"), hab_text(sp)),
                (t!("会える場所", "Where", "地点"), place_text(e)),
            ] {
                let text = format!("{}: {}", label, value);
                let hh = self.gfx.measure_h(&text, Font::Normal, tw);
                self.gfx.text(&text, tx, y, tw, hh, Font::Normal, TEXT);
                y += hh + 6.0;
            }
            let hint = t!("まだ撮影していない。上の場所で探してみよう。", "Not photographed yet. Look for it at the places above.", "还没有拍到。去上面的地点找找吧。");
            let hh = self.gfx.measure_h(hint, Font::Normal, tw);
            self.gfx.text(hint, tx, y + 8.0, tw, hh, Font::Normal, DIM);
            self.add_button(tx, y1 - 90.0, (tx + 220.0).min(x1), y1 - 52.0, t!("探しに行く", "Go look for it", "去寻找"), Act::Goto(Target::Sp(e)), true, true);
            self.add_button(tx, y1 - 44.0, (tx + 220.0).min(x1), y1 - 6.0, t!("◎ レーダーで探す", "◎ Find with radar", "◎ 用雷达寻找"), Act::Radar(e), true, false);
        }
    }

    // ------------------------------------------------------------ 宿題・操作・先生のことば

    fn render_missions(&mut self) {
        let (w, h) = (self.w(), self.h());
        let pw = (w - 40.0).min(820.0);
        let x0 = (w - pw) * 0.5;
        let y0 = self.top_h() + 16.0;
        let y1 = h - 16.0;
        self.panel_box(x0, y0, x0 + pw, y1);
        self.gfx.text(t!("夏休みの宿題：関東の昆虫図鑑づくり", "Summer assignment: a Kanto insect guide", "暑假作业：制作关东昆虫图鉴"), x0 + 20.0, y0 + 14.0, pw - 160.0, 26.0, Font::Big, TEXT);
        self.add_button(x0 + pw - 120.0, y0 + 12.0, x0 + pw - 16.0, y0 + 44.0, t!("閉じる", "Close", "关闭"), Act::Close, true, false);
        let mut y = y0 + 60.0;
        let cur = self.prog.mission;
        let total = self.prog.done_count();
        self.gfx.text(&tf!("合格 {}/{}　好きな順番で取り組めます", "Passed {}/{}  Do them in any order", "合格 {}/{}　可按任意顺序完成", total, MISSIONS.len()), x0 + 20.0, y0 + 40.0, pw - 160.0, 16.0, Font::Small, DIM);
        if self.prog.all_clear() {
            self.gfx.text(t!("すべての宿題に合格！ ほかの地方は準備中。待っててね。", "All tasks passed! Other regions are coming soon.", "全部作业合格！其他地区正在准备中，敬请期待。"), x0 + 20.0, y, pw - 40.0, 22.0, Font::Big, GOLD);
            y += 30.0;
        }
        {
            let m = &MISSIONS[cur];
            let passed = self.prog.is_done(cur);
            let head = if passed {
                tf!("✔ 選んだ宿題　{}. {}（合格）", "✔ Chosen task  {}. {} (passed)", "✔ 选择的作业　{}. {}（合格）", cur + 1, m.title.get())
            } else {
                tf!("選んだ宿題　{}. {}", "Chosen task  {}. {}", "选择的作业　{}. {}", cur + 1, m.title.get())
            };
            self.gfx.text(&head, x0 + 20.0, y, pw - 40.0, 22.0, Font::Big, if passed { GOOD } else { ACCENT });
            y += 28.0;
            let b = format!("{}{}{}", t!("トキ先生", "Ms. Toki", "时老师"), t!("：", ": ", "："), m.brief.get());
            let hh = self.gfx.measure_h(&b, Font::Normal, pw - 40.0);
            self.gfx.text(&b, x0 + 20.0, y, pw - 40.0, hh, Font::Normal, TEXT);
            y += hh + 8.0;
            for (tg, need) in m.target_list() {
                let ok = self.prog.stars[tg.entry()] >= need;
                let s = format!("{} {}", if ok { "✔" } else { "○" }, tg.name());
                self.gfx.text(&s, x0 + 32.0, y + 3.0, pw - 200.0, 20.0, Font::Normal, if ok { GOOD } else { TEXT });
                if !ok {
                    self.add_button(x0 + pw - 140.0, y, x0 + pw - 24.0, y + 24.0, t!("行く", "Go", "前往"), Act::Goto(tg), true, false);
                }
                y += 27.0;
            }
            y += 10.0;
        }
        // 宿題の一覧（押すと選べる）
        let rh = 32.0;
        let colw = (pw - 40.0) * 0.5;
        for i in 0..MISSIONS.len() {
            let (cx, cy) = (x0 + 20.0 + (i % 2) as f32 * colw, y + (i / 2) as f32 * rh);
            if cy + rh - 4.0 > y1 - 8.0 {
                continue;
            }
            let mark = if self.prog.is_done(i) { "✔" } else if i == cur { "▶" } else { "○" };
            let label = format!("{} {}. {}", mark, i + 1, MISSIONS[i].title.get());
            self.add_button(cx, cy, cx + colw - 8.0, cy + rh - 4.0, &label, Act::SelectMission(i), true, i == cur);
        }
    }

    fn render_help(&mut self) {
        let (w, h) = (self.w(), self.h());
        let pw = (w - 40.0).min(760.0);
        let x0 = (w - pw) * 0.5;
        let y0 = self.top_h() + 16.0;
        let y1 = h - 16.0;
        self.panel_box(x0, y0, x0 + pw, y1);
        let title_w = pw - if platform::CAN_RELOAD { 410.0 } else { 160.0 };
        self.gfx.text(t!("カプセルの操作", "Capsule controls", "观测舱操作"), x0 + 20.0, y0 + 14.0, title_w, 26.0, Font::Big, TEXT);
        if platform::CAN_RELOAD {
            let label = self.reload_label();
            self.add_button(x0 + pw - 380.0, y0 + 12.0, x0 + pw - 130.0, y0 + 44.0, label, Act::Reload, true, self.confirm_reload > 0.0);
        }
        self.add_button(x0 + pw - 120.0, y0 + 12.0, x0 + pw - 16.0, y0 + 44.0, t!("閉じる", "Close", "关闭"), Act::Close, true, false);
        let lines: Vec<(&str, &str)> = if self.touch {
            vec![
                (t!("地図", "Map", "地图"), t!("光る観察スポットをタップするとカプセルが飛ぶ・なぞって地図を動かす・2本指で拡大縮小", "Tap a glowing spot to fly there, drag to pan, pinch to zoom", "点击发光的观察点飞过去·拖动移动地图·双指缩放")),
                (t!("季節", "Season", "季节"), t!("上の月の帯で季節、その下で早朝・昼・夕方・夜を選ぶ。虫ごとに見られる月と時間帯がちがう", "Pick the season on the month bar and the time of day below it. Each insect has its own months and times", "在上方月份条选择季节，下面选择清晨、白天、傍晚、夜晚。每种昆虫出现的月份和时间不同")),
                (t!("降りる", "Land", "降落"), t!("スポットに着いたら、左下の「〜へ降りる」で林や草はら、池などへ", "At a spot, use the Land buttons at the bottom left to visit woods, meadows or ponds", "到达观察点后，用左下的降落按钮前往树林、草地或池塘")),
                (t!("観察", "Observe", "观察"), t!("触れた方へカプセルが進む。＋－でズーム、撮影ボタンで撮る", "The capsule moves where you touch. ＋－ to zoom, shutter to shoot", "观测舱朝触碰方向前进。＋－变焦，按快门拍摄")),
                (t!("関東の外", "Elsewhere", "关东以外"), t!("日本のほかの地方や外国は、いま準備中です", "Other regions of Japan and overseas are coming soon", "日本其他地区和国外正在准备中")),
            ]
        } else {
            vec![
                (t!("地図", "Map", "地图"), t!("光る観察スポットをクリックして飛ぶ／[ と ] で前・次のスポット／ドラッグや矢印キーで地図を動かす／ホイールで拡大縮小／K で関東の全体", "Click a glowing spot to fly there / [ and ] for previous/next spot / drag or arrow keys to pan / wheel to zoom / K to show Kanto", "点击发光的观察点飞过去／[ 和 ] 切换观察点／拖动或方向键移动地图／滚轮缩放／K 查看关东全图")),
                (t!("季節", "Season", "季节"), t!("上の月の帯をクリック、または , と . で前・次の月へ。その下で早朝・昼・夕方・夜を選ぶ（T で切り替え）。虫ごとに見られる月と時間帯がちがう。夜はライト（L）を使おう", "Click the month bar, or use , and . for the previous/next month. Below it, pick early morning, daytime, evening or night (T to cycle). Each insect has its own months and times. Use the light (L) at night", "点击上方的月份条，或用 , 和 . 切换月份。下面选择清晨、白天、傍晚、夜晚（T 切换）。每种昆虫出现的月份和时间不同。夜里用灯（L）")),
                (t!("降りる", "Land", "降落"), t!("Enter：スポットの林・草はら・池などに降りる（左下のボタンで場所を選べる）", "Enter: land at the spot (choose woods, meadow, pond… with the buttons at the bottom left)", "Enter：降落到观察点（用左下的按钮选择树林、草地、池塘等）")),
                (t!("観察", "Observe", "观察"), t!("WASD・矢印・マウスを押した方へ進む／Shift で加速／ホイール・Q/E でズーム／L でライト", "WASD, arrows or hold the mouse to move / Shift to boost / wheel or Q,E to zoom / L for light", "WASD·方向键·按住鼠标前进／Shift加速／滚轮·Q/E变焦／L开灯")),
                (t!("撮影", "Photo", "拍摄"), t!("Space（または右クリック）：ある程度大きく・真ん中に・明るく写すと図鑑に収録できる", "Space (or right-click): a reasonably big, centred and bright shot goes into the guide", "Space（或右键）：拍得够大、居中、明亮，就能收录进图鉴")),
                (t!("画面", "Screens", "画面"), t!("F1 宿題　F2 図鑑　F3 この説明　Esc 地図へ・閉じる　Ctrl＋＋/－ 文字の大きさ　F11 全画面", "F1 tasks  F2 guide  F3 help  Esc map/close  Ctrl +/- text size  F11 fullscreen", "F1 作业　F2 图鉴　F3 说明　Esc 返回地图·关闭　Ctrl＋＋/－ 字号　F11 全屏")),
            ]
        };
        let mut y = y0 + 58.0;
        for (k, v) in lines {
            self.gfx.text(k, x0 + 20.0, y, 90.0, 20.0, Font::Big, ACCENT);
            let hh = self.gfx.measure_h(v, Font::Normal, pw - 140.0).max(22.0);
            self.gfx.text(v, x0 + 118.0, y + 2.0, pw - 140.0, hh, Font::Normal, TEXT);
            y += hh + 12.0;
        }
        y += 6.0;
        let tips = t!(
            "ヒント：宿題のカードの「行く」で、その虫に会えるスポットへカプセルが向かう。観察中、宿題の虫は ! の印で示される。逃げる虫には、ゆっくり近づこう。",
            "Tip: 'Go' on the task card sends the capsule to a spot where that insect lives. While observing, task insects are marked with '!'. Approach shy insects slowly.",
            "提示：点击作业卡上的“前往”，观测舱会前往能遇到该昆虫的观察点。观察时，作业中的昆虫会用“!”标出。对胆小的昆虫要慢慢靠近。"
        );
        let hh = self.gfx.measure_h(tips, Font::Normal, pw - 40.0);
        self.gfx.text(tips, x0 + 20.0, y, pw - 40.0, hh, Font::Normal, [0.80, 0.98, 0.80, 1.0]);
        y += hh + 16.0;
        self.gfx.text(platform::storage_note(), x0 + 20.0, y, pw - 40.0, 40.0, Font::Small, DIM);
        y += 44.0;
        if y + 36.0 < y1 {
            let bw = ((pw - 50.0) / 2.0).min(240.0);
            self.add_button(x0 + 20.0, y, x0 + 20.0 + bw, y + 34.0, t!("記録をファイルに書き出す", "Export save to a file", "导出存档文件"), Act::Export, self.have_save, false);
            self.add_button(x0 + 30.0 + bw, y, x0 + 30.0 + bw * 2.0, y + 34.0, t!("記録ファイルを読み込む", "Load a save file", "读取存档文件"), Act::Import, true, false);
            y += 44.0;
        }
        if y + 40.0 < y1 {
            self.gfx.text(t!("地図：Natural Earth、国土地理院「地球地図日本」を加工して作成", "Map data: Natural Earth; GSI Global Map Japan (modified)", "地图：根据Natural Earth和日本国土地理院“地球地图日本”加工制作"), x0 + 20.0, y, pw - 40.0, 20.0, Font::Small, DIM);
        }
    }

    fn reload_label(&self) -> &'static str {
        if self.confirm_reload > 0.0 {
            t!("再読み込みする（記録は残る）", "Reload now (keeps saves)", "再按一次重新加载（存档保留）")
        } else {
            t!("キャッシュを消して再読み込み", "Clear cache and reload", "清除缓存并重新加载")
        }
    }

    fn teacher(&mut self, x: f32, y: f32, r: f32) {
        if self.artwork.draw_teacher(&self.gfx, [x - r, y - r, x + r, y + r]) {
            self.gfx.stroke_round(x - r, y - r, x + r, y + r, 10.0, 1.5, EDGE);
            return;
        }
        // トキ先生（未来の先生）
        self.gfx.fill_circle(x, y, r + 4.0, [0.45, 0.85, 0.80, 0.6]);
        self.gfx.fill_circle(x, y, r, [0.10, 0.22, 0.32, 1.0]);
        self.gfx.fill_circle(x, y + r * 0.12, r * 0.62, [0.98, 0.86, 0.74, 1.0]);
        self.gfx.fill_round(x - r * 0.7, y - r * 0.62, x + r * 0.7, y - r * 0.12, r * 0.3, [0.36, 0.30, 0.56, 1.0]);
        self.gfx.fill_circle(x - r * 0.22, y + r * 0.1, r * 0.08, [0.1, 0.1, 0.15, 1.0]);
        self.gfx.fill_circle(x + r * 0.22, y + r * 0.1, r * 0.08, [0.1, 0.1, 0.15, 1.0]);
        self.gfx.stroke_round(x - r * 0.42, y - r * 0.02, x - r * 0.04, y + r * 0.22, 4.0, 2.0, [0.3, 0.9, 1.0, 0.9]);
        self.gfx.stroke_round(x + r * 0.04, y - r * 0.02, x + r * 0.42, y + r * 0.22, 4.0, 2.0, [0.3, 0.9, 1.0, 0.9]);
        self.gfx.fill_round(x - r * 0.18, y + r * 0.38, x + r * 0.18, y + r * 0.46, 3.0, [0.8, 0.4, 0.4, 1.0]);
    }

    fn dialog(&mut self, speaker: &str, text: &str, foot: &str) {
        self.dialog_voice = format!("{speaker}。{text}。{foot}");
        let (w, h) = (self.w(), self.h());
        let pw = (w - 40.0).min(820.0);
        let x0 = (w - pw) * 0.5;
        let ph = if self.compact() { 200.0 } else { 220.0 };
        let y0 = h - ph - 24.0;
        self.gfx.fill_rect(0.0, 0.0, w, h, [0.0, 0.0, 0.0, 0.35]);
        self.panel_box(x0, y0, x0 + pw, y0 + ph);
        let r = if self.compact() { 40.0 } else { 48.0 };
        self.teacher(x0 + 24.0 + r, y0 + 24.0 + r, r);
        let tx = x0 + 48.0 + r * 2.0;
        let tw = pw - (tx - x0) - 20.0;
        self.gfx.text(speaker, tx, y0 + 16.0, tw, 22.0, Font::Big, ACCENT);
        let hh = self.gfx.measure_h(text, Font::Normal, tw);
        self.gfx.text(text, tx, y0 + 46.0, tw, hh, Font::Normal, TEXT);
        self.gfx.text(foot, x0, y0 + ph - 28.0, pw - 20.0, 20.0, Font::Small, DIM);
        let fw = self.gfx.measure(foot, Font::Small);
        let _ = fw;
    }

    fn render_intro(&mut self, n: u8) {
        let name = t!("トキ先生", "Ms. Toki", "时老师");
        let pages: [L; 4] = [
            L(
                "こんにちは！ 理科クラブのトキです。この夏の自由研究は「昆虫図鑑」づくり。虫と同じ大きさまで縮むミクロ観察カプセルに乗って、虫を撮影して図鑑に収録してもらいます。",
                "Hello! I'm Ms. Toki from the science club. This summer's project is to build an insect field guide. Ride the micro observation capsule, which shrinks to insect size, photograph insects and add them to your guide.",
                "你好！我是科学俱乐部的时老师。这个夏天的自由研究是制作“昆虫图鉴”。乘坐能缩小到昆虫大小的微观观测舱，拍摄昆虫并收录进图鉴。",
            ),
            L(
                "最初の行き先は関東地方。高尾山や明治神宮、井の頭公園、筑波山、霞ヶ浦など、虫の観察にぴったりの場所がたくさんあります。ほかの地方や外国は、いま準備中です。",
                "Your first destination is the Kanto region: Mt. Takao, Meiji Jingu, Inokashira Park, Mt. Tsukuba, Lake Kasumigaura and many more great places for insects. Other regions and countries are coming soon.",
                "第一站是关东地区。高尾山、明治神宫、井之头公园、筑波山、霞浦等，有许多适合观察昆虫的地方。其他地区和国外正在准备中。",
            ),
            L(
                "地図で光っている観察スポットを押すと、カプセルが飛んでいきます。着いたら「雑木林へ降りる」などのボタンで、虫のすむ場所に降りましょう。虫には季節があります。地図の上の月の帯で春・夏・秋・冬を、その下で早朝・昼・夕方・夜を選べます。カブトムシやホタルは夜の虫ですよ。",
                "Press a glowing observation spot on the map and the capsule flies there. When you arrive, use a button such as 'Land in the woodland' to visit the insects' home. Insects have seasons and times: use the month bar at the top of the map for spring to winter, and the row below it for early morning, daytime, evening or night. Rhinoceros beetles and fireflies come out at night.",
                "点击地图上发光的观察点，观测舱就会飞过去。到达后，用“降落到杂木林”等按钮，降落到昆虫生活的地方。昆虫有季节和时间之分。用地图上方的月份条切换春夏秋冬，下面一排可以选择清晨、白天、傍晚、夜晚。独角仙和萤火虫是夜里的昆虫哦。",
            ),
            L(
                "虫は、大きく・真ん中に・明るく撮れば図鑑に収録されます。右上の宿題のカードの「行く」を押せば、カプセルが目的の虫のいる場所へ向かいます。では、いってらっしゃい！",
                "A big, centred, bright shot goes straight into your guide. Press 'Go' on the task card at the top right and the capsule heads to where the insect lives. Off you go!",
                "昆虫拍得够大、居中、明亮，就会收录进图鉴。点击右上角作业卡上的“前往”，观测舱就会驶向目标昆虫所在的地方。出发吧！",
            ),
        ];
        let text = pages[(n as usize).min(3)].get();
        let foot = tf!("（{}/4）クリック・Enter で次へ", "({}/4) Click or press Enter", "（{}/4）点击或按 Enter 继续", n + 1);
        self.dialog(name, text, &foot);
    }

    /// 関東の外を選んだとき
    fn render_soon(&mut self) {
        let name = t!("トキ先生", "Ms. Toki", "时老师");
        let text = tf!(
            "{}の昆虫は、いま準備中です。もう少し待っててね！ いまは関東地方の観察スポットをめぐって、図鑑をうめていきましょう。",
            "The insects of {} are still being prepared. Please wait a little longer! For now, explore the observation spots in Kanto and fill your guide.",
            "{}的昆虫还在准备中。请再稍等一下！现在先去关东地区的观察点，把图鉴填满吧。",
            self.soon_name
        );
        self.dialog(name, &text, t!("クリック・Enter で閉じる", "Click or press Enter", "点击或按 Enter 关闭"));
    }

    fn render_done(&mut self, m: usize) {
        let name = t!("トキ先生", "Ms. Toki", "时老师");
        let mm = &MISSIONS[m];
        let mut text = format!("{}\n\n{}", tf!("宿題{}「{}」合格！", "Task {} '{}' passed!", "作业{}“{}”合格！", m + 1, mm.title.get()), mm.done.get());
        if self.prog.all_clear() {
            text.push_str(&tf!("\n\nこれで関東の{}の宿題すべて合格です！ ほかの地方は準備中、待っててね。", "\n\nThat completes all {} Kanto tasks! Other regions are coming soon.", "\n\n关东的{}个作业全部合格了！其他地区正在准备中，敬请期待。", MISSIONS.len()));
        } else if !self.prog.is_done(self.prog.mission) {
            text.push_str(&tf!("\n\n次の宿題：{}（宿題の一覧から好きなものを選べます）", "\n\nNext task: {} (pick any task from the task list)", "\n\n下一个作业：{}（可在作业一览中任选）", MISSIONS[self.prog.mission].title.get()));
        }
        self.dialog(name, &text, t!("クリック・Enter で閉じる", "Click or press Enter", "点击或按 Enter 关闭"));
    }

    // ------------------------------------------------------------ 写真・知らせ・説明

    fn render_photo_gl(&mut self) {
        let Some(p)=&self.photo else { return; };
        let Some(&(entry,stars,fresh,_))=p.lines.get(p.index) else { return; };
        let (age,w,h)=(p.age,self.w(),self.h());
        let reveal=crate::reveal::draw_stage(&mut self.gfx,w,h,age,stars,fresh);
        let [x0,y0,x1,y1]=reveal.bounds;
        if reveal.front {
            let pad=(x1-x0)*0.045;
            // 画像が無いものは手描きの絵（draw_entry_art の中で画像を先に試す）
            self.draw_entry_art(entry,x0+pad,y0+48.0,x1-pad,y1-132.0,reveal.alpha);
            let c=crate::reveal::accent(stars);
            self.gfx.rect(x0+pad,y1-122.0,x1-pad,y1-121.0,[c[0],c[1],c[2],0.45]);
        }
    }

    fn render_photo(&mut self) {
        let Some(p) = &self.photo else { return };
        if let Some(&(entry,stars,fresh,better))=p.lines.get(p.index) {
            let (age,index,count)=(p.age,p.index,p.lines.len());
            let (w,h)=(self.w(),self.h());
            let reveal=crate::reveal::layout(w,h,age);
            if !reveal.front || age<0.60 { return; }
            let [x0,y0,x1,y1]=reveal.bounds;
            let cw=x1-x0;
            let c=crate::reveal::accent(stars);
            let _ = better;
            let head=if fresh { t!("新発見！", "NEW DISCOVERY!", "新发现！") }
                else { t!("撮影成功！", "CAPTURED!", "拍摄成功！") };
            self.gfx.text(head,x0-100.0,y0-44.0,cw+200.0,32.0,Font::BigCenter,c);
            self.gfx.text(&tf!("図鑑 No.{:02}","Guide No.{:02}","图鉴 No.{:02}",entry+1),
                x0+14.0,y0+15.0,cw-90.0,20.0,Font::Normal,c);
            self.gfx.text(&format!("{} / {}",index+1,count),x1-68.0,y0+15.0,54.0,20.0,Font::NormalCenter,DIM);
            self.gfx.text(entry_name(entry),x0+10.0,y1-111.0,cw-20.0,27.0,Font::BigCenter,TEXT);
            if entry<n_species() {
                self.gfx.text(SPECIES[entry].sci,x0+10.0,y1-83.0,cw-20.0,18.0,Font::NormalCenter,DIM);
            }
            let cx=(x0+x1)*0.5;
            if age>0.82 {
                let el=age-0.82;
                let pop=(el*14.0).sin()*(-el*7.0).exp()*8.0;
                let stamp=if fresh { t!("図鑑に収録", "Added to the guide", "收录图鉴") } else { t!("収録済み", "Already logged", "已收录") };
                self.gfx.text(stamp,x0+10.0,y1-52.0-pop,cw-20.0,32.0,Font::BigCenter,if fresh {GOLD} else {GOOD});
            }
            let label=if index+1<count {
                t!("次のカード ▶", "Next card ▶", "下一张 ▶")
            } else { t!("図鑑に収録！", "Added to guide!", "已收录图鉴！") };
            if age>1.2 {
                self.add_button(cx-110.0,y1+16.0,cx+110.0,y1+53.0,label,Act::NextPhoto,true,true);
                self.gfx.text(t!("タップ・Enterで進む　Escですべて閉じる", "Tap / Enter to continue · Esc to close all", "点击或Enter继续 · Esc全部关闭"),
                    cx-250.0,y1+59.0,500.0,18.0,Font::Small,DIM);
            }
            return;
        }
        let lines = p.lines.clone();
        let advice = p.advice;
        let (w, h) = (self.w(), self.h());
        let pw = if self.compact() { 300.0 } else { 340.0 };
        let n = lines.len().max(1) as f32;
        let ph = 64.0 + n * 30.0 + if advice.is_some() { 48.0 } else { 0.0 };
        let x0 = (w - pw) * 0.5;
        let y0 = (h - ph) * 0.42;
        // ポラロイド風の白い枠
        self.gfx.fill_round(x0 - 6.0, y0 - 6.0, x0 + pw + 6.0, y0 + ph + 6.0, 8.0, [0.96, 0.95, 0.90, 0.97]);
        self.gfx.fill_round(x0 + 6.0, y0 + 6.0, x0 + pw - 6.0, y0 + ph - 6.0, 6.0, [0.06, 0.14, 0.20, 1.0]);
        let head = if lines.iter().any(|l| l.2) { t!("撮影成功！ 図鑑に収録", "Got it! Added to the guide", "拍摄成功！已收录图鉴") } else if !lines.is_empty() { t!("撮影した！", "Snap!", "拍到了！") } else { t!("うまく撮れなかった…", "Not a good shot…", "没拍好……") };
        self.gfx.text(head, x0 + 16.0, y0 + 16.0, pw - 32.0, 26.0, Font::Big, TEXT);
        let mut y = y0 + 52.0;
        for (e, st, new, better) in lines {
            let _ = (st, better);
            self.gfx.text(entry_name(e), x0 + 18.0, y, pw - 110.0, 22.0, Font::Normal, TEXT);
            if new {
                self.gfx.fill_round(x0 + pw - 72.0, y + 1.0, x0 + pw - 18.0, y + 19.0, 6.0, ACCENT);
                self.gfx.text("NEW", x0 + pw - 72.0, y + 1.0, 54.0, 18.0, Font::NormalCenter, [0.1, 0.05, 0.0, 1.0]);
            } else {
                self.gfx.text(t!("収録済み", "Logged", "已收录"), x0 + pw - 90.0, y + 2.0, 76.0, 18.0, Font::Small, GOOD);
            }
            y += 30.0;
        }
        if let Some(a) = advice {
            self.gfx.text(a, x0 + 18.0, y + 2.0, pw - 36.0, 44.0, Font::Small, [1.0, 0.86, 0.6, 1.0]);
        }
    }

    fn render_notices(&mut self) {
        let w = self.w();
        let mut y = self.top_h() + 10.0;
        // 地図では、上の月の帯に重ねない
        if self.screen == Screen::Map && self.panel == Panel::None {
            let (_, my0, _, my1) = self.month_rect();
            if my0 < self.h() * 0.5 {
                y = my1 + 8.0;
            }
        }
        let notes: Vec<(String, f32)> = self.notices.clone();
        for (s, left) in notes {
            let a = left.min(1.0);
            let tw = self.gfx.measure(&s, Font::Normal).min(w * 0.6) + 28.0;
            let x0 = (w - tw) * 0.5;
            self.gfx.fill_round(x0, y, x0 + tw, y + 30.0, 10.0, [0.02, 0.08, 0.12, 0.85 * a]);
            self.gfx.text(&s, x0 + 14.0, y + 6.0, tw - 28.0, 20.0, Font::Normal, [1.0, 0.95, 0.85, a]);
            y += 36.0;
        }
    }

    fn render_tip(&mut self) {
        if self.touch || self.panel != Panel::None {
            return;
        }
        let Some(s) = self.tip.clone() else { return };
        let (mx, my) = self.mouse;
        let mut wd: f32 = 0.0;
        for l in s.lines() {
            wd = wd.max(self.gfx.measure(l, Font::Normal));
        }
        let wd = wd + 20.0;
        let hh = s.lines().count() as f32 * 19.0 + 12.0;
        let x0 = (mx + 16.0).min(self.w() - wd - 6.0);
        let y0 = if my - hh - 10.0 > self.top_h() { my - hh - 10.0 } else { my + 20.0 };
        self.gfx.fill_round(x0, y0, x0 + wd, y0 + hh, 8.0, BG2);
        self.gfx.stroke_round(x0, y0, x0 + wd, y0 + hh, 8.0, 1.0, EDGE);
        self.gfx.text(&s, x0 + 10.0, y0 + 6.0, wd - 16.0, hh, Font::Normal, TEXT);
    }
}

// These simulations use the headless renderer; native Direct3D needs a real HWND.
#[cfg(all(test, any(not(windows), feature = "headless")))]
mod tests {
    use super::*;

    fn game() -> Game {
        Game::new(Gfx::new(1600, 900))
    }

    fn started() -> Game {
        let mut g = game();
        g.do_act(Act::NewGame);
        g.prog.intro_done = true;
        g.panel = Panel::None;
        g
    }

    /// 対象の虫に近づいて、大きく撮れるように合わせる
    fn frame_subject(g: &mut Game, sp: usize) {
        let sc = g.scene.as_mut().unwrap();
        let cr = sc.creatures.iter_mut().find(|c| c.sp == sp).unwrap_or_else(|| panic!("missing {}", SPECIES[sp].slug));
        let b = cr.bounds();
        let center = ((b[0] + b[2]) * 0.5, (b[1] + b[3]) * 0.5);
        g.dcam = center;
        sc.cap.x = center.0;
        sc.cap.y = center.1;
        g.prog.light = true;
        g.lens = ((g.w() / (b[2] - b[0]) * 0.8).min(g.h() / (b[3] - b[1]) * 0.8) / (g.h() / 4.0)).clamp(1.0, 8.0);
    }

    #[test]
    fn reading_pages_preserve_every_character_and_fit_the_available_height() {
        let mut gfx = Gfx::new(1600, 900);
        for e in 0..n_entries() {
            for section in crate::guide::sections(e, lang::Lang::Ja) {
                let original = section.body.clone();
                for (width, height) in [(230.0, 100.0), (480.0, 320.0)] {
                    let pages = reading_pages(&mut gfx, std::slice::from_ref(&section), width, height);
                    assert_eq!(pages.iter().map(|(_, body)| body.as_str()).collect::<String>(), original);
                    for (title, body) in pages {
                        assert_eq!(title, section.title);
                        assert!(gfx.measure_h(&body, Font::Normal, width) <= height, "{}", entry_slug(e));
                    }
                }
            }
        }
    }

    #[test]
    fn entry_pages_navigate_and_keep_unphotographed_content_hidden() {
        let mut g = game();
        g.prog.stars[0] = 1;
        g.do_act(Act::Entry(0));
        g.render_entry_ui(0);
        let count = g.entry_book.as_ref().unwrap().pages.len();
        assert!(count > 2);
        let first_voice = g.voice_text();
        g.key_down(VK_DOWN);
        assert_eq!(g.entry_page, 1);
        assert_ne!(g.voice_text(), first_voice);
        g.do_act(Act::EntryPage(1000));
        assert_eq!(g.entry_page, count - 1);
        g.key_down(VK_RIGHT);
        assert_eq!(g.panel, Panel::Entry(1));
        g.render_entry_ui(1);
        assert!(g.entry_book.is_none());
        assert!(!g.voice_text().contains(SPECIES[1].desc.get()));
    }

    #[test]
    fn clicking_outside_kanto_says_coming_soon() {
        let mut g = started();
        g.render();
        // 大阪
        g.zoom = 2.0;
        g.cam = lonlat_to_map(135.5, 34.7);
        g.follow = false;
        g.clamp_cam();
        let (sx, sy) = g.map_to_screen(g.cam.0, g.cam.1);
        g.map_click(sx, sy);
        assert_eq!(g.panel, Panel::Soon);
        g.render();
        assert!(g.dialog_voice.contains("近畿") || g.dialog_voice.contains("Kinki") || g.dialog_voice.contains("近畿"));
        g.key_down(VK_RETURN);
        assert_eq!(g.panel, Panel::None);
        // 関東の観察スポットを押すと飛んでいく
        g.show_kanto();
        let s = &SPOTS[S_TAKAO];
        let (mx, my) = lonlat_to_map(s.lon, s.lat);
        let (sx, sy) = g.map_to_screen(mx, my);
        g.map_click(sx, sy);
        assert_eq!(g.panel, Panel::None);
        for _ in 0..300 {
            g.update(0.1);
        }
        assert_eq!(g.at_spot(), Some(S_TAKAO));
    }

    #[test]
    fn new_game_land_and_shoot() {
        let mut g = game();
        g.do_act(Act::NewGame);
        assert_eq!(g.screen, Screen::Map);
        for _ in 0..4 {
            g.do_act(Act::IntroNext);
        }
        assert_eq!(g.panel, Panel::None);
        assert_eq!(g.at_spot(), Some(S_SHOWA));
        g.key_down(VK_RETURN);
        assert_eq!(g.screen, Screen::Dive);
        for _ in 0..30 {
            g.frame(&[], 1.0 / 60.0, true);
        }
        g.do_act(Act::Shoot);
        assert!(g.photo.is_some());
        g.close_photo();
        g.do_act(Act::Surface);
        assert_eq!(g.screen, Screen::Map);
    }

    #[test]
    fn every_mission_can_be_visited_photographed_and_completed() {
        let mut g = started();
        for (mi, m) in MISSIONS.iter().enumerate() {
            g.prog.mission = mi;
            for (target, _) in m.target_list() {
                g.close_photo();
                g.goto(target);
                for _ in 0..400 {
                    g.update(0.1);
                }
                assert!(g.cap_target.is_none(), "{}", target.name());
                let s = g.at_spot().expect("arrived at a spot");
                let h = g.pending_hab.expect("knows where to land");
                assert!(SPOTS[s].habs.contains(&h));
                g.enter_dive(h);
                assert_eq!(g.screen, Screen::Dive);
                frame_subject(&mut g, target.entry());
                g.shoot();
                assert_eq!(g.prog.stars[target.entry()], 1, "could not photograph {} for {}", target.name(), m.title.0);
                g.close_photo();
                g.do_act(Act::Surface);
            }
            assert!(g.prog.is_done(mi), "{}", m.title.0);
        }
        let saved = Progress::from_bytes(&g.prog.to_bytes()).unwrap();
        assert_eq!(saved.done_count(), MISSIONS.len());
    }

    #[test]
    fn every_insect_can_be_photographed() {
        let mut g = started();
        for sp in 0..n_species() {
            g.close_photo();
            g.goto(Target::Sp(sp));
            for _ in 0..400 {
                g.update(0.1);
            }
            let h = g.pending_hab.unwrap();
            g.enter_dive(h);
            frame_subject(&mut g, sp);
            g.shoot();
            assert_eq!(g.prog.stars[sp], 1, "could not photograph {}", SPECIES[sp].slug);
            g.close_photo();
            g.do_act(Act::Surface);
        }
    }

    #[test]
    fn seasons_change_the_insects_and_are_saved() {
        let mut g = started();
        let kabuto = species_index("japanese-rhinoceros-beetle");
        g.prog.month = 12;
        g.goto(Target::Sp(kabuto));
        assert!(SPECIES[kabuto].in_season(g.prog.month), "goto moves to the insect's season");
        for _ in 0..400 {
            g.update(0.1);
        }
        let h = g.pending_hab.unwrap();
        g.enter_dive(h);
        assert!(g.scene.as_ref().unwrap().present.contains(&kabuto));
        g.do_act(Act::Month(1));
        assert_eq!(g.screen, Screen::Map, "changing the month returns to the map");
        g.enter_dive(h);
        let sc = g.scene.as_ref().unwrap();
        assert!(!sc.present.contains(&kabuto));
        assert!(sc.present.iter().all(|&i| SPECIES[i].in_season(1)));
        g.do_act(Act::StepMonth(-1));
        assert_eq!(g.prog.month, 12);
        let saved = Progress::from_bytes(&g.prog.to_bytes()).unwrap();
        assert_eq!(saved.month, 12);
        g.do_act(Act::Surface);
        g.render();
        assert!(g.buttons.iter().filter(|b| matches!(b.act, Act::Month(_))).count() == 12);
    }

    #[test]
    fn time_of_day_changes_the_insects_and_is_saved() {
        let mut g = started();
        let kabuto = species_index("japanese-rhinoceros-beetle");
        g.prog.time = 1;
        g.goto(Target::Sp(kabuto));
        assert!(SPECIES[kabuto].active_at(g.prog.time), "goto moves to an active time");
        for _ in 0..400 {
            g.update(0.1);
        }
        let h = g.pending_hab.unwrap();
        g.enter_dive(h);
        assert!(g.scene.as_ref().unwrap().present.contains(&kabuto));
        g.do_act(Act::TimeOfDay(1));
        assert_eq!(g.screen, Screen::Map);
        g.enter_dive(h);
        let sc = g.scene.as_ref().unwrap();
        assert!(!sc.present.contains(&kabuto));
        assert!(sc.present.iter().all(|&i| SPECIES[i].active_at(1)));
        // 夜は暗く、ライトで明るくなる
        g.do_act(Act::TimeOfDay(3));
        g.enter_dive(h);
        let sc = g.scene.as_ref().unwrap();
        assert!(sc.ambient_at(sc.cap.y) < 0.3);
        assert!(sc.light_at(sc.cap.x + 0.5, sc.cap.y, true) > 0.8);
        let saved = Progress::from_bytes(&g.prog.to_bytes()).unwrap();
        assert_eq!(saved.time, 3);
        g.do_act(Act::Surface);
        g.render();
        assert_eq!(g.buttons.iter().filter(|b| matches!(b.act, Act::TimeOfDay(_))).count(), 4);
    }

    #[test]
    fn all_screens_render() {
        let mut g = game();
        g.frame(&[], 0.016, true);
        g.do_act(Act::NewGame);
        for p in [Panel::Intro(0), Panel::Guide, Panel::Entry(3), Panel::Entry(n_entries() - 1), Panel::Missions, Panel::Help, Panel::Done(0), Panel::Soon] {
            g.panel = p;
            g.frame(&[], 0.016, true);
        }
        g.panel = Panel::None;
        g.prog.intro_done = true;
        g.radar = Some(5);
        g.frame(&[], 0.016, true);
        for (s, spot) in SPOTS.iter().enumerate() {
            g.prog.month = [7, 1, 4, 10][s % 4];
            g.prog.time = (s % 4) as u8;
            g.cap = lonlat_to_map(spot.lon, spot.lat);
            g.cap_target = None;
            for &h in spot.habs {
                g.do_act(Act::Dive(h));
                assert_eq!(g.screen, Screen::Dive, "{} {:?}", s, h);
                g.lens = 3.0;
                g.frame(&[], 0.016, true);
                g.do_act(Act::Shoot);
                g.frame(&[], 0.016, true);
                g.close_photo();
                g.do_act(Act::Surface);
            }
        }
    }

    #[test]
    fn guide_cards_accept_swipes_and_taps_at_mobile_scales() {
        for (width, height) in [(844, 390), (1280, 720), (2532, 1170)] {
            let mut g = Game::new(Gfx::new(width, height));
            g.panel = Panel::Guide;
            g.touch = true;
            g.render();
            let ui = g.gfx.ui;
            let card = g.buttons.iter().find(|b| matches!(b.act, Act::Entry(_))).unwrap();
            let (x, y) = ((card.x0 + card.x1) * 0.5, (card.y0 + card.y1) * 0.5);
            let entry = card.act;
            assert_eq!(g.hit_kind(x * ui, y * ui), 3, "card at {width}x{height}");
            g.handle(Ev::Down(0, x, y));
            if let Act::Entry(e) = entry {
                assert_eq!(g.panel, Panel::Entry(e));
            }
        }
    }

    #[test]
    fn group_tabs_fit_and_remain_clickable() {
        for width in [480, 800, 1280, 1600] {
            let mut g = Game::new(Gfx::new(width, 900));
            g.panel = Panel::Guide;
            g.render();
            let tabs: Vec<_> = g.buttons.iter().filter(|b| matches!(b.act, Act::GuideTab(_))).collect();
            assert_eq!(tabs.len(), GROUPS.len());
            let (_, grid_y, _, _, _, _, _) = g.guide_layout();
            for b in tabs {
                assert!(b.x0 >= 0.0 && b.x1 <= g.w() && b.y1 < grid_y);
            }
        }
    }
}
