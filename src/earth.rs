//! 地図（日本を中心にしたメルカトル図法）と、関東地方の観察スポット
//!
//! 地図の座標は経度1度あたり `DEG` の単位。左端が西経40度（中心が東経140度）、上下は緯度±80度まで。
//! 地図の絵は tools/make_map.py が作った assets/map/*.png（Natural Earth・国土地理院「地球地図日本」）。

use crate::lang::L;
use crate::rng::fbm;

pub const DEG: f32 = 10.0;
/// 地図の左端の経度
pub const LON_LEFT: f32 = -40.0;
pub const LAT_MAX: f32 = 80.0;
/// 1ラジアンあたりの地図の単位
const RAD: f32 = DEG * 180.0 / std::f32::consts::PI;
pub const MAP_W: f32 = 360.0 * DEG;
/// 2 * merc(80°) * RAD
pub const MAP_H: f32 = 2791.73;

fn merc(lat: f32) -> f32 {
    let lat = lat.clamp(-LAT_MAX, LAT_MAX).to_radians();
    (std::f32::consts::FRAC_PI_4 + lat * 0.5).tan().ln()
}

fn merc_top() -> f32 {
    merc(LAT_MAX)
}

pub fn lonlat_to_map(lon: f32, lat: f32) -> (f32, f32) {
    ((lon - LON_LEFT).rem_euclid(360.0) * DEG, (merc_top() - merc(lat)) * RAD)
}

pub fn map_to_lonlat(x: f32, y: f32) -> (f32, f32) {
    let lon = x / DEG + LON_LEFT;
    let lon = (lon + 180.0).rem_euclid(360.0) - 180.0;
    let lat = (2.0 * (merc_top() - y / RAD).exp().atan() - std::f32::consts::FRAC_PI_2).to_degrees();
    (lon, lat)
}

/// 二点間の距離（km）
pub fn km_dist(a: (f32, f32), b: (f32, f32)) -> f32 {
    let mlat = ((a.1 + b.1) * 0.5).to_radians().cos().max(0.05);
    let mut dlon = (a.0 - b.0).abs() % 360.0;
    if dlon > 180.0 {
        dlon = 360.0 - dlon;
    }
    let dx = dlon * mlat;
    let dy = a.1 - b.1;
    (dx * dx + dy * dy).sqrt() * 111.2
}

// ---------------------------------------------------------------- 環境

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Habitat {
    /// 雑木林・社寺の森
    Forest,
    /// 山の森
    Mountain,
    /// 草地・野原・湿原
    Meadow,
    /// 公園・花壇・町の木
    Park,
    /// 池・沼・湖
    Pond,
    /// 川・渓流
    River,
    /// 田んぼ
    Paddy,
    /// 海辺
    Shore,
}

pub const FOREST: u16 = 1;
pub const MOUNTAIN: u16 = 2;
pub const MEADOW: u16 = 4;
pub const PARK: u16 = 8;
pub const POND: u16 = 16;
pub const RIVER: u16 = 32;
pub const PADDY: u16 = 64;
pub const SHORE: u16 = 128;

pub const HABITATS: [Habitat; 8] = [Habitat::Forest, Habitat::Mountain, Habitat::Meadow, Habitat::Park, Habitat::Pond, Habitat::River, Habitat::Paddy, Habitat::Shore];

impl Habitat {
    pub fn bit(self) -> u16 {
        1 << (self as u16)
    }
    pub fn name(self) -> &'static str {
        match self {
            Habitat::Forest => t!("雑木林", "Woodland", "杂木林"),
            Habitat::Mountain => t!("山の森", "Mountain forest", "山林"),
            Habitat::Meadow => t!("草はら", "Meadow", "草地"),
            Habitat::Park => t!("公園・花だん", "Park & flowerbeds", "公园·花坛"),
            Habitat::Pond => t!("池・沼", "Pond", "池塘"),
            Habitat::River => t!("川べり", "Riverside", "河边"),
            Habitat::Paddy => t!("田んぼ", "Rice paddy", "稻田"),
            Habitat::Shore => t!("海辺", "Seashore", "海边"),
        }
    }
    /// 水のある場面（水面より下に潜れる）
    pub fn is_water(self) -> bool {
        matches!(self, Habitat::Pond | Habitat::River | Habitat::Paddy)
    }
    /// 高い木が立つ場面
    pub fn has_trees(self) -> bool {
        matches!(self, Habitat::Forest | Habitat::Mountain | Habitat::Park)
    }
}

// ---------------------------------------------------------------- 観察スポット

pub struct SpotDef {
    pub name: L,
    /// 都県・市町村
    pub area: L,
    pub lon: f32,
    pub lat: f32,
    /// 降りられる場面
    pub habs: &'static [Habitat],
    pub desc: L,
}

use Habitat::*;

#[allow(dead_code)]
#[allow(dead_code)]
pub const S_TAKAO: usize = 0;
#[allow(dead_code)]
pub const S_MEIJI: usize = 1;
#[allow(dead_code)]
pub const S_INOKASHIRA: usize = 2;
#[allow(dead_code)]
pub const S_UENO: usize = 3;
#[allow(dead_code)]
pub const S_SHOWA: usize = 4;
#[allow(dead_code)]
pub const S_TAMAGAWA: usize = 5;
#[allow(dead_code)]
pub const S_SAYAMA: usize = 6;
#[allow(dead_code)]
pub const S_KAMAKURA: usize = 7;
#[allow(dead_code)]
pub const S_TSUKUBA: usize = 8;
#[allow(dead_code)]
pub const S_KASUMIGAURA: usize = 9;
#[allow(dead_code)]
pub const S_NIKKO: usize = 10;
#[allow(dead_code)]
pub const S_WATARASE: usize = 11;
#[allow(dead_code)]
pub const S_RANZAN: usize = 12;
#[allow(dead_code)]
pub const S_NARITA: usize = 13;
#[allow(dead_code)]
pub const S_KASAI: usize = 14;
#[allow(dead_code)]
pub const S_NAGATORO: usize = 15;
#[allow(dead_code)]
pub const S_YORO: usize = 16;

pub static SPOTS: &[SpotDef] = &[
    SpotDef {
        name: L("高尾山", "Mt. Takao", "高尾山"),
        area: L("東京都八王子市", "Hachioji, Tokyo", "东京都八王子市"),
        lon: 139.244,
        lat: 35.625,
        habs: &[Mountain, Forest, River],
        desc: L("都心から近いのに、昆虫の種類がとても多い山。南と北の森の生きものが出会う。", "A mountain close to central Tokyo with a remarkable variety of insects, where southern and northern forests meet.", "离东京市中心很近，却拥有非常丰富昆虫种类的山。南北森林的生物在这里相遇。"),
    },
    SpotDef {
        name: L("明治神宮", "Meiji Jingu", "明治神宫"),
        area: L("東京都渋谷区", "Shibuya, Tokyo", "东京都涩谷区"),
        lon: 139.699,
        lat: 35.676,
        habs: &[Forest, Park],
        desc: L("100年ほど前に人が植えた木が、大きな森に育った神社。都会の真ん中の虫の楽園。", "A shrine whose trees, planted about a century ago, have grown into a great forest: an insect haven in the middle of the city.", "约100年前人工种下的树木长成大森林的神社，是都市中心的昆虫乐园。"),
    },
    SpotDef {
        name: L("井の頭恩賜公園", "Inokashira Park", "井之头恩赐公园"),
        area: L("東京都武蔵野市・三鷹市", "Musashino & Mitaka, Tokyo", "东京都武藏野市·三鹰市"),
        lon: 139.574,
        lat: 35.700,
        habs: &[Park, Pond, Forest],
        desc: L("湧き水がつくる池と雑木林のある公園。トンボや水の虫も見られる。", "A park with a spring-fed pond and woodland, home to dragonflies and water insects.", "拥有泉水形成的池塘和杂木林的公园，可以看到蜻蜓和水生昆虫。"),
    },
    SpotDef {
        name: L("上野恩賜公園・不忍池", "Ueno Park & Shinobazu Pond", "上野恩赐公园·不忍池"),
        area: L("東京都台東区", "Taito, Tokyo", "东京都台东区"),
        lon: 139.771,
        lat: 35.712,
        habs: &[Park, Pond],
        desc: L("ハスの池と大きな木の並ぶ公園。夏はセミの声でいっぱいになる。", "A park of lotus ponds and tall trees, filled with cicada song in summer.", "有荷花池和高大树木的公园，夏天到处是蝉鸣。"),
    },
    SpotDef {
        name: L("国営昭和記念公園", "Showa Kinen Park", "国营昭和纪念公园"),
        area: L("東京都立川市・昭島市", "Tachikawa & Akishima, Tokyo", "东京都立川市·昭岛市"),
        lon: 139.398,
        lat: 35.705,
        habs: &[Meadow, Park, Forest],
        desc: L("広い草はらと花畑、雑木林がある大きな公園。バッタやチョウを探しやすい。", "A huge park of open meadows, flower fields and woodland: easy hunting for grasshoppers and butterflies.", "有广阔草地、花田和杂木林的大公园，很容易找到蝗虫和蝴蝶。"),
    },
    SpotDef {
        name: L("多摩川の河原", "Tama River banks", "多摩川河滩"),
        area: L("東京都世田谷区・神奈川県川崎市", "Setagaya, Tokyo & Kawasaki, Kanagawa", "东京都世田谷区·神奈川县川崎市"),
        lon: 139.627,
        lat: 35.610,
        habs: &[River, Meadow],
        desc: L("東京と神奈川の境を流れる川。河原の草はらはバッタの宝庫。", "The river between Tokyo and Kanagawa. Its grassy banks are full of grasshoppers.", "流经东京和神奈川交界的河流。河滩草地是蝗虫的宝库。"),
    },
    SpotDef {
        name: L("狭山丘陵（トトロの森）", "Sayama Hills", "狭山丘陵（龙猫森林）"),
        area: L("埼玉県所沢市・東京都東村山市", "Tokorozawa, Saitama & Higashimurayama, Tokyo", "埼玉县所泽市·东京都东村山市"),
        lon: 139.42,
        lat: 35.77,
        habs: &[Forest, Paddy],
        desc: L("雑木林と谷あいの田んぼが残る里山。昔ながらの虫がたくさんすむ。", "Satoyama countryside with woodland and valley rice paddies, where traditional countryside insects still thrive.", "保留着杂木林和山谷稻田的里山，栖息着许多传统昆虫。"),
    },
    SpotDef {
        name: L("鶴岡八幡宮（鎌倉）", "Tsurugaoka Hachimangu, Kamakura", "鹤冈八幡宫（镰仓）"),
        area: L("神奈川県鎌倉市", "Kamakura, Kanagawa", "神奈川县镰仓市"),
        lon: 139.556,
        lat: 35.326,
        habs: &[Forest, Park, Pond],
        desc: L("古都鎌倉の神社。境内の大木と源平池に、暖かい地方の虫も来る。", "A shrine in the old capital of Kamakura. Its great trees and ponds attract insects from warmer regions too.", "古都镰仓的神社。境内的大树和源平池也吸引着来自温暖地区的昆虫。"),
    },
    SpotDef {
        name: L("筑波山", "Mt. Tsukuba", "筑波山"),
        area: L("茨城県つくば市", "Tsukuba, Ibaraki", "茨城县筑波市"),
        lon: 140.103,
        lat: 36.225,
        habs: &[Mountain, Forest],
        desc: L("関東平野にぽつんとそびえる山。ふもとから山頂まで森が続く。", "A lone peak rising from the Kanto Plain, forested from foot to summit.", "孤零零耸立在关东平原上的山，从山脚到山顶都是森林。"),
    },
    SpotDef {
        name: L("霞ヶ浦", "Lake Kasumigaura", "霞浦"),
        area: L("茨城県土浦市", "Tsuchiura, Ibaraki", "茨城县土浦市"),
        lon: 140.21,
        lat: 36.07,
        habs: &[Pond, Paddy, Meadow],
        desc: L("日本で2番目に大きな湖。まわりにはハスの畑と田んぼが広がる。", "Japan's second-largest lake, surrounded by lotus fields and rice paddies.", "日本第二大湖，周围是广阔的莲田和稻田。"),
    },
    SpotDef {
        name: L("日光・戦場ヶ原", "Senjogahara, Nikko", "日光·战场原"),
        area: L("栃木県日光市", "Nikko, Tochigi", "栃木县日光市"),
        lon: 139.43,
        lat: 36.78,
        habs: &[Mountain, Meadow, River],
        desc: L("標高1400mの湿原。夏でも涼しく、山の虫や北の虫に会える。", "A marsh 1,400 m up. Cool even in summer, it is home to mountain and northern insects.", "海拔1400米的湿原，夏天也很凉爽，能遇到山地和北方的昆虫。"),
    },
    SpotDef {
        name: L("渡良瀬遊水地", "Watarase Retarding Basin", "渡良濑游水地"),
        area: L("栃木県栃木市・群馬県・埼玉県・茨城県", "Tochigi, Gunma, Saitama & Ibaraki", "栃木县栃木市·群马县·埼玉县·茨城县"),
        lon: 139.68,
        lat: 36.23,
        habs: &[Meadow, Pond],
        desc: L("4つの県にまたがる、本州最大のヨシ原。湿地のトンボがたくさんいる。", "The largest reed bed on Honshu, spanning four prefectures and full of wetland dragonflies.", "横跨四县的本州最大芦苇荡，栖息着许多湿地蜻蜓。"),
    },
    SpotDef {
        name: L("嵐山渓谷（オオムラサキの森）", "Ranzan Valley", "岚山溪谷（大紫蛱蝶森林）"),
        area: L("埼玉県嵐山町", "Ranzan, Saitama", "埼玉县岚山町"),
        lon: 139.32,
        lat: 36.05,
        habs: &[Forest, River],
        desc: L("国蝶オオムラサキを守る雑木林と、清流の渓谷。", "Woodland protected for the national butterfly, the Great Purple Emperor, beside a clear mountain stream.", "保护国蝶大紫蛱蝶的杂木林与清流溪谷。"),
    },
    SpotDef {
        name: L("成田山新勝寺", "Naritasan Shinshoji", "成田山新胜寺"),
        area: L("千葉県成田市", "Narita, Chiba", "千叶县成田市"),
        lon: 140.318,
        lat: 35.786,
        habs: &[Park, Forest, Paddy],
        desc: L("大きなお寺の公園と森。まわりの台地には田んぼと雑木林が残る。", "The park and forest of a great temple, with rice paddies and woods on the surrounding plateau.", "大寺院的公园与森林，周围台地上保留着稻田和杂木林。"),
    },
    SpotDef {
        name: L("葛西臨海公園", "Kasai Rinkai Park", "葛西临海公园"),
        area: L("東京都江戸川区", "Edogawa, Tokyo", "东京都江户川区"),
        lon: 139.861,
        lat: 35.643,
        habs: &[Shore, Park, Meadow],
        desc: L("東京湾に面した公園。砂浜と草はらで、海辺の虫が見られる。", "A park on Tokyo Bay, where beaches and meadows host seaside insects.", "面朝东京湾的公园，在沙滩和草地上可以看到海边的昆虫。"),
    },
    SpotDef {
        name: L("長瀞（秩父）", "Nagatoro, Chichibu", "长瀞（秩父）"),
        area: L("埼玉県長瀞町", "Nagatoro, Saitama", "埼玉县长瀞町"),
        lon: 139.11,
        lat: 36.10,
        habs: &[River, Mountain],
        desc: L("荒川の上流の岩だたみと清流。川の虫と山の虫がそろう。", "Rocky terraces and clear water on the upper Arakawa, with both river and mountain insects.", "荒川上游的岩石台阶和清流，河流和山地的昆虫都有。"),
    },
    SpotDef {
        name: L("養老渓谷（房総）", "Yoro Valley, Boso", "养老溪谷（房总）"),
        area: L("千葉県大多喜町・市原市", "Otaki & Ichihara, Chiba", "千叶县大多喜町·市原市"),
        lon: 140.15,
        lat: 35.25,
        habs: &[Forest, River],
        desc: L("房総半島の暖かい森と渓谷。ホタルの名所として知られる。", "Warm forests and gorges of the Boso Peninsula, famous for fireflies.", "房总半岛温暖的森林与溪谷，以萤火虫闻名。"),
    },
];

// ---------------------------------------------------------------- 地域

/// 都道府県の名前（1=北海道 … 47=沖縄）
const PREFS: [&str; 47] = [
    "北海道", "青森県", "岩手県", "宮城県", "秋田県", "山形県", "福島県", "茨城県", "栃木県", "群馬県", "埼玉県", "千葉県", "東京都", "神奈川県",
    "新潟県", "富山県", "石川県", "福井県", "山梨県", "長野県", "岐阜県", "静岡県", "愛知県", "三重県", "滋賀県", "京都府", "大阪府", "兵庫県",
    "奈良県", "和歌山県", "鳥取県", "島根県", "岡山県", "広島県", "山口県", "徳島県", "香川県", "愛媛県", "高知県", "福岡県", "佐賀県", "長崎県",
    "熊本県", "大分県", "宮崎県", "鹿児島県", "沖縄県",
];

/// 地図の上の場所の分類
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Place {
    Sea,
    /// 関東地方（都道府県の番号）
    Kanto(u8),
    /// 日本のほかの地方（都道府県の番号）
    Japan(u8),
    /// 海外
    Abroad,
}

pub fn pref_name(code: u8) -> &'static str {
    let ja = PREFS.get(code.wrapping_sub(1) as usize).copied().unwrap_or("");
    match crate::lang::lang() {
        crate::lang::Lang::Ja | crate::lang::Lang::Zh => ja,
        crate::lang::Lang::En => PREFS_EN.get(code.wrapping_sub(1) as usize).copied().unwrap_or(""),
    }
}

const PREFS_EN: [&str; 47] = [
    "Hokkaido", "Aomori", "Iwate", "Miyagi", "Akita", "Yamagata", "Fukushima", "Ibaraki", "Tochigi", "Gunma", "Saitama", "Chiba", "Tokyo", "Kanagawa",
    "Niigata", "Toyama", "Ishikawa", "Fukui", "Yamanashi", "Nagano", "Gifu", "Shizuoka", "Aichi", "Mie", "Shiga", "Kyoto", "Osaka", "Hyogo",
    "Nara", "Wakayama", "Tottori", "Shimane", "Okayama", "Hiroshima", "Yamaguchi", "Tokushima", "Kagawa", "Ehime", "Kochi", "Fukuoka", "Saga", "Nagasaki",
    "Kumamoto", "Oita", "Miyazaki", "Kagoshima", "Okinawa",
];

/// 地方の名前（都道府県の番号から）
pub fn district_name(code: u8) -> &'static str {
    match code {
        1 => t!("北海道", "Hokkaido", "北海道"),
        2..=7 => t!("東北地方", "Tohoku", "东北地区"),
        8..=14 => t!("関東地方", "Kanto", "关东地区"),
        15..=23 => t!("中部地方", "Chubu", "中部地区"),
        24..=30 => t!("近畿地方", "Kinki", "近畿地区"),
        31..=35 => t!("中国地方", "Chugoku", "中国地区"),
        36..=39 => t!("四国地方", "Shikoku", "四国地区"),
        40..=46 => t!("九州地方", "Kyushu", "九州地区"),
        _ => t!("沖縄", "Okinawa", "冲绳"),
    }
}

/// 海外のおおよその地域の名前
pub fn abroad_name(lon: f32, lat: f32) -> &'static str {
    if lat < -60.0 {
        return t!("南極", "Antarctica", "南极洲");
    }
    if (-170.0..-30.0).contains(&lon) {
        return if lat > 13.0 { t!("北アメリカ", "North America", "北美洲") } else { t!("南アメリカ", "South America", "南美洲") };
    }
    if lon >= 110.0 && lat < -10.0 || lon >= 150.0 && lat < 0.0 {
        return t!("オーストラリア・オセアニア", "Australia & Oceania", "澳大利亚·大洋洲");
    }
    if lon >= 124.0 && lon < 131.0 && lat > 33.0 && lat < 43.0 {
        return t!("朝鮮半島", "Korean Peninsula", "朝鲜半岛");
    }
    if lat > 50.0 && lon > 40.0 || lon > 130.0 && lat > 42.0 {
        return t!("シベリア・ロシア", "Siberia & Russia", "西伯利亚·俄罗斯");
    }
    if lon >= 95.0 && lat < 23.0 {
        return t!("東南アジア", "Southeast Asia", "东南亚");
    }
    if lon >= 68.0 && lon < 95.0 && lat < 30.0 {
        return t!("インド・南アジア", "India & South Asia", "印度·南亚");
    }
    if lon >= 73.0 && lat >= 18.0 {
        return t!("中国・モンゴル", "China & Mongolia", "中国·蒙古");
    }
    if lon < 60.0 && lon >= -20.0 && lat < 36.0 {
        return t!("アフリカ・中東", "Africa & the Middle East", "非洲·中东");
    }
    t!("ヨーロッパ・中央アジア", "Europe & Central Asia", "欧洲·中亚")
}

// ---------------------------------------------------------------- 地図の絵

/// 地図の上の四角に置く絵（world・japan・kanto）
pub struct Layer {
    pub w: u32,
    pub h: u32,
    /// 地図の座標の四角
    pub rect: [f32; 4],
    /// 1画素の値（world・japan は L、kanto は環境と都道府県）
    data: Vec<u8>,
    chans: usize,
}

impl Layer {
    fn load(bytes: &[u8], lon0: f32, lat0: f32, lon1: f32, lat1: f32) -> Layer {
        let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Png).expect("map png");
        let (x0, y1) = lonlat_to_map(lon0, lat0);
        let (mut x1, y0) = lonlat_to_map(lon1, lat1);
        if x1 <= x0 {
            x1 += MAP_W;
        }
        let (w, h) = (img.width(), img.height());
        let (data, chans) = match img {
            image::DynamicImage::ImageLuma8(g) => (g.into_raw(), 1),
            other => (other.to_rgb8().into_raw(), 3),
        };
        Layer { w, h, rect: [x0, y0, x1, y1], data, chans }
    }

    /// 地図の座標の画素の値（四角の外なら None）
    fn at(&self, x: f32, y: f32) -> Option<&[u8]> {
        let [x0, y0, x1, y1] = self.rect;
        if x < x0 || x >= x1 || y < y0 || y >= y1 {
            return None;
        }
        let i = ((x - x0) / (x1 - x0) * self.w as f32) as usize;
        let j = ((y - y0) / (y1 - y0) * self.h as f32) as usize;
        let k = (j.min(self.h as usize - 1) * self.w as usize + i.min(self.w as usize - 1)) * self.chans;
        Some(&self.data[k..k + self.chans])
    }

    fn get(&self, i: usize, j: usize) -> &[u8] {
        let k = (j * self.w as usize + i) * self.chans;
        &self.data[k..k + self.chans]
    }
}

pub struct Earth {
    pub world: Layer,
    pub japan: Layer,
    pub kanto: Layer,
}

pub fn rgba(c: [f32; 3], a: f32) -> u32 {
    let r = (c[0].clamp(0.0, 1.0) * 255.0) as u32;
    let g = (c[1].clamp(0.0, 1.0) * 255.0) as u32;
    let b = (c[2].clamp(0.0, 1.0) * 255.0) as u32;
    let a = (a.clamp(0.0, 1.0) * 255.0) as u32;
    r | (g << 8) | (b << 16) | (a << 24)
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

fn is_kanto(code: u8) -> bool {
    (8..=14).contains(&code)
}

impl Earth {
    pub fn new() -> Earth {
        Earth {
            world: Layer::load(include_bytes!("../assets/map/world.png"), LON_LEFT, -LAT_MAX, LON_LEFT + 360.0, LAT_MAX),
            japan: Layer::load(include_bytes!("../assets/map/japan.png"), 122.0, 24.0, 150.0, 46.0),
            kanto: Layer::load(include_bytes!("../assets/map/kanto.png"), 138.35, 34.85, 141.0, 37.2),
        }
    }

    /// 地図の上の場所が何か
    pub fn place(&self, lon: f32, lat: f32) -> Place {
        let (x, y) = lonlat_to_map(lon, lat);
        if let Some(v) = self.kanto.at(x, y) {
            let code = v[1];
            if code == 0 {
                return if v[0] == 6 { Place::Kanto(0) } else { Place::Sea };
            }
            return if is_kanto(code) { Place::Kanto(code) } else if code <= 47 { Place::Japan(code) } else { Place::Abroad };
        }
        if let Some(v) = self.japan.at(x, y) {
            return match v[0] {
                0 => Place::Sea,
                // 伊豆諸島・小笠原は関東（東京都）でも、まだ観察できない
                c @ 1..=47 => Place::Japan(c),
                70 => Place::Japan(0),
                _ => Place::Abroad,
            };
        }
        match self.world.at(x, y) {
            Some(v) if v[0] > 127 => Place::Abroad,
            _ => Place::Sea,
        }
    }

    /// 世界の陸（日本のまわりの四角の中は日本の絵に任せる）
    pub fn world_pixels(&self) -> Vec<u32> {
        let l = &self.world;
        let mut px = vec![0u32; (l.w * l.h) as usize];
        let [jx0, jy0, jx1, jy1] = self.japan.rect;
        for j in 0..l.h as usize {
            for i in 0..l.w as usize {
                let x = l.rect[0] + (i as f32 + 0.5) / l.w as f32 * (l.rect[2] - l.rect[0]);
                let y = l.rect[1] + (j as f32 + 0.5) / l.h as f32 * (l.rect[3] - l.rect[1]);
                if x > jx0 + 2.0 && x < jx1 - 2.0 && y > jy0 + 2.0 && y < jy1 - 2.0 {
                    continue;
                }
                if l.get(i, j)[0] > 127 {
                    let n = fbm(i as f32 * 0.03, j as f32 * 0.03, 11, 4);
                    let (_, lat) = map_to_lonlat(x, y);
                    px[j * l.w as usize + i] = rgba(land_col(lat, n, true), 1.0);
                }
            }
        }
        px
    }

    /// 日本とまわりの国（関東の四角の中は関東の絵に任せる）
    pub fn japan_pixels(&self) -> Vec<u32> {
        let l = &self.japan;
        let (w, h) = (l.w as usize, l.h as usize);
        let mut px = vec![0u32; w * h];
        let [kx0, ky0, kx1, ky1] = self.kanto.rect;
        for j in 0..h {
            for i in 0..w {
                let x = l.rect[0] + (i as f32 + 0.5) / w as f32 * (l.rect[2] - l.rect[0]);
                let y = l.rect[1] + (j as f32 + 0.5) / h as f32 * (l.rect[3] - l.rect[1]);
                if x > kx0 + 0.3 && x < kx1 - 0.3 && y > ky0 + 0.3 && y < ky1 - 0.3 {
                    continue;
                }
                let v = l.get(i, j)[0];
                if v == 0 {
                    continue;
                }
                let (_, lat) = map_to_lonlat(x, y);
                let n = fbm(i as f32 * 0.04, j as f32 * 0.04, 13, 4);
                let c = match v {
                    70 => [0.30, 0.52, 0.74],
                    1..=47 => {
                        // 都道府県の境
                        let edge = [(1i32, 0i32), (0, 1), (-1, 0), (0, -1)].iter().any(|&(dx, dy)| {
                            let (a, b) = (i as i32 + dx, j as i32 + dy);
                            a >= 0 && b >= 0 && (a as usize) < w && (b as usize) < h && { let u = l.get(a as usize, b as usize)[0]; u != v && u != 0 && u != 70 }
                        });
                        let base = land_col(lat, n, false);
                        let base = if is_kanto(v) { lerp3(base, [0.55, 0.78, 0.40], 0.35) } else { base };
                        if edge { lerp3(base, [0.98, 0.95, 0.85], 0.55) } else { base }
                    }
                    _ => land_col(lat, n, true),
                };
                px[j * w + i] = rgba(c, 1.0);
            }
        }
        px
    }

    /// 関東：環境ごとの色（関東の外は少し暗く）
    pub fn kanto_pixels(&self) -> Vec<u32> {
        let l = &self.kanto;
        let (w, h) = (l.w as usize, l.h as usize);
        let mut px = vec![0u32; w * h];
        for j in 0..h {
            for i in 0..w {
                let v = l.get(i, j);
                let (zone, code) = (v[0], v[1]);
                if zone == 0 && code == 0 {
                    continue;
                }
                let n = fbm(i as f32 * 0.05, j as f32 * 0.05, 17, 4);
                let n2 = fbm(i as f32 * 0.012, j as f32 * 0.012, 19, 4);
                let mut c = match zone {
                    1 => lerp3([0.70, 0.72, 0.42], [0.62, 0.68, 0.36], n),
                    2 => lerp3([0.50, 0.74, 0.36], [0.58, 0.80, 0.40], n),
                    3 => lerp3([0.24, 0.50, 0.24], [0.32, 0.56, 0.28], n),
                    4 => {
                        let shade = 0.75 + 0.5 * n2;
                        let g = lerp3([0.14, 0.36, 0.18], [0.24, 0.44, 0.24], n);
                        [g[0] * shade, g[1] * shade, g[2] * shade]
                    }
                    5 => lerp3([0.70, 0.70, 0.70], [0.80, 0.78, 0.74], n),
                    6 => [0.30, 0.54, 0.78],
                    7 => [0.36, 0.62, 0.84],
                    8 => lerp3([0.88, 0.82, 0.62], [0.94, 0.88, 0.70], n),
                    _ => [0.6, 0.7, 0.5],
                };
                if code != 0 {
                    let edge = [(1i32, 0i32), (0, 1), (-1, 0), (0, -1)].iter().any(|&(dx, dy)| {
                        let (a, b) = (i as i32 + dx, j as i32 + dy);
                        a >= 0 && b >= 0 && (a as usize) < w && (b as usize) < h && { let u = l.get(a as usize, b as usize)[1]; u != code && u != 0 }
                    });
                    if !is_kanto(code) {
                        // 関東の外は日本の地図と同じ色にして、つなぎ目を見せない
                        let y = l.rect[1] + (j as f32 + 0.5) / h as f32 * (l.rect[3] - l.rect[1]);
                        let (_, lat) = map_to_lonlat(0.0, y);
                        c = if zone == 6 || zone == 7 { [0.30, 0.52, 0.74] } else { land_col(lat, fbm(i as f32 * 0.004, j as f32 * 0.004, 13, 4), code > 47) };
                    }
                    if edge {
                        c = lerp3(c, [1.0, 0.97, 0.88], 0.7);
                    }
                }
                px[j * w + i] = rgba(c, 1.0);
            }
        }
        px
    }
}

/// 陸の色（緯度で、北は針葉樹・南は濃い緑・乾いた所は茶色っぽく）
fn land_col(lat: f32, n: f32, foreign: bool) -> [f32; 3] {
    let a = lat.abs();
    let ramp = |x: f32, a0: f32, a1: f32| ((x - a0) / (a1 - a0)).clamp(0.0, 1.0);
    let green = lerp3([0.34, 0.54, 0.30], [0.44, 0.60, 0.34], n);
    // 亜熱帯の乾いた帯（15〜35度）、北の森（50度より北）、極地（66度より北）をなめらかにつなぐ
    let dry = ramp(a, 12.0, 20.0) * (1.0 - ramp(a, 31.0, 39.0));
    let c = lerp3(green, lerp3([0.62, 0.58, 0.40], [0.50, 0.56, 0.34], n), dry * 0.8);
    let c = lerp3(c, lerp3([0.30, 0.42, 0.30], [0.40, 0.48, 0.34], n), ramp(a, 46.0, 54.0));
    let c = lerp3(c, lerp3([0.78, 0.80, 0.78], [0.88, 0.90, 0.90], n), ramp(a, 62.0, 70.0));
    if foreign {
        lerp3(c, [0.50, 0.50, 0.46], 0.35)
    } else {
        c
    }
}

/// 海の色（地図の背景）
pub fn ocean_texture() -> (u32, u32, Vec<u32>) {
    let (w, h) = (720u32, 560u32);
    let mut px = vec![0u32; (w * h) as usize];
    for j in 0..h {
        for i in 0..w {
            let n = fbm(i as f32 * 0.02, j as f32 * 0.02, 5, 5);
            let n2 = fbm(i as f32 * 0.08, j as f32 * 0.08, 9, 3);
            let (_, lat) = map_to_lonlat(0.0, (j as f32 + 0.5) / h as f32 * MAP_H);
            let warm = (1.0 - (lat.abs() / 70.0)).clamp(0.0, 1.0);
            let c = [0.08 + 0.03 * n + 0.02 * warm, 0.24 + 0.06 * n + 0.05 * warm + 0.02 * n2, 0.44 + 0.08 * n + 0.05 * warm];
            px[(j * w + i) as usize] = rgba(c, 1.0);
        }
    }
    (w, h, px)
}

/// 雲（白、透明度で形を作る）。横につなげて流す
pub fn cloud_texture() -> (u32, u32, Vec<u32>) {
    let (w, h) = (512u32, 256u32);
    let mut px = vec![0u32; (w * h) as usize];
    for j in 0..h {
        for i in 0..w {
            let a = i as f32 / w as f32 * std::f32::consts::TAU;
            let (sx, cx) = a.sin_cos();
            let y = j as f32 * 0.035;
            let n = fbm(cx * 2.6 + 10.0, y + sx * 2.6, 77, 5) * 0.65 + fbm(sx * 5.0 + 3.0, y * 2.0 + cx * 5.0, 91, 4) * 0.35;
            let v = ((n - 0.55) * 2.6).clamp(0.0, 0.8);
            px[(j * w + i) as usize] = rgba([1.0, 1.0, 1.0], v);
        }
    }
    (w, h, px)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_roundtrip_and_size() {
        assert!((2.0 * merc(LAT_MAX) * RAD - MAP_H).abs() < 0.1);
        for &(lon, lat) in &[(139.7, 35.7), (-73.0, 40.7), (2.3, 48.9), (151.2, -33.9), (-40.5, 0.0)] {
            let (x, y) = lonlat_to_map(lon, lat);
            assert!((0.0..=MAP_W).contains(&x) && (0.0..=MAP_H).contains(&y));
            let (lo, la) = map_to_lonlat(x, y);
            assert!((lo - lon).abs() < 0.01 && (la - lat).abs() < 0.01, "{lon},{lat} -> {lo},{la}");
        }
        // 東京は地図のほぼ真ん中
        let (x, _) = lonlat_to_map(139.7, 35.7);
        assert!((x - MAP_W * 0.5).abs() < 10.0);
    }

    #[test]
    fn places_are_classified() {
        let e = Earth::new();
        for (i, s) in SPOTS.iter().enumerate() {
            assert!(matches!(e.place(s.lon, s.lat), Place::Kanto(_)), "spot {} {:?}", i, e.place(s.lon, s.lat));
        }
        assert_eq!(e.place(141.35, 43.06), Place::Japan(1)); // 札幌
        assert_eq!(e.place(135.50, 34.69), Place::Japan(27)); // 大阪
        assert_eq!(e.place(-100.0, 40.0), Place::Abroad);
        assert_eq!(e.place(160.0, 30.0), Place::Sea);
        assert_eq!(e.place(139.8, 35.4), Place::Sea); // 東京湾
    }
}
