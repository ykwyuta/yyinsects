//! 関東地方の昆虫（図鑑の項目）
//!
//! 大きさ（体長・チョウは前ばねの長さ）・季節・すみかは、ゲーム用のおおよそのもの。

use crate::earth::*;
use crate::lang::L;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Beh {
    /// ひらひら飛ぶ（チョウ・ガ）
    Flutter,
    /// まっすぐ速く飛ぶ・空中で止まる（トンボ・ハチ）
    Fly,
    /// 木の幹や草にとまる（近づくと飛んで逃げる）
    Perch,
    /// 地面を歩く
    Crawl,
    /// 地面を歩き、ときどき跳ぶ（バッタ）
    Hop,
    /// 水の中を泳ぐ
    Swim,
    /// 水面をすべる
    Skate,
}

impl Beh {
    /// 地面に足をつけているか（足の位置を基準に置く）
    pub fn grounded(self) -> bool {
        matches!(self, Beh::Crawl | Beh::Hop)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Group {
    Butterfly,
    Dragonfly,
    Beetle,
    Bug,
    Grasshopper,
    Bee,
}

pub const GROUPS: [Group; 6] = [Group::Butterfly, Group::Dragonfly, Group::Beetle, Group::Bug, Group::Grasshopper, Group::Bee];

impl Group {
    pub fn name(self) -> &'static str {
        match self {
            Group::Butterfly => t!("チョウ・ガ", "Butterflies & moths", "蝴蝶·蛾"),
            Group::Dragonfly => t!("トンボ", "Dragonflies", "蜻蜓"),
            Group::Beetle => t!("甲虫", "Beetles", "甲虫"),
            Group::Bug => t!("セミ・カメムシ", "Cicadas & bugs", "蝉·蝽"),
            Group::Grasshopper => t!("バッタ・カマキリ", "Grasshoppers & mantises", "蝗虫·螳螂"),
            Group::Bee => t!("ハチ・アリなど", "Bees, ants & others", "蜂·蚂蚁等"),
        }
    }
    pub fn order(self) -> &'static str {
        match self {
            Group::Butterfly => t!("チョウ目（鱗翅目）", "Lepidoptera", "鳞翅目"),
            Group::Dragonfly => t!("トンボ目", "Odonata", "蜻蜓目"),
            Group::Beetle => t!("コウチュウ目（甲虫目）", "Coleoptera", "鞘翅目"),
            Group::Bug => t!("カメムシ目（半翅目）", "Hemiptera", "半翅目"),
            Group::Grasshopper => t!("バッタ目・カマキリ目・ナナフシ目", "Orthoptera, Mantodea & Phasmida", "直翅目·螳螂目·竹节虫目"),
            Group::Bee => t!("ハチ目・ハサミムシ目", "Hymenoptera & Dermaptera", "膜翅目·革翅目"),
        }
    }
}

/// 絵の形（art.rs が描く）
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Art {
    /// チョウ 0=シロチョウ 1=アゲハ（尾つき） 2=タテハ 3=シジミ 4=マダラ 5=ヤママユガ（尾の長いガ）
    Butterfly(u8),
    /// トンボ 0=細いトンボ 1=ヤンマ 2=イトトンボ 3=カワトンボ 4=チョウトンボ
    Dragonfly(u8),
    /// コガネムシ 0=カナブン 1=カブトムシ 2=センチコガネ
    Scarab(u8),
    /// クワガタ 0=小さなあご 1=ノコギリ 2=ミヤマ
    Stag(u8),
    Ladybug,
    Firefly,
    Longhorn,
    TigerBeetle,
    Jewel,
    DivingBeetle,
    /// セミ 0=透明なはね 1=茶色いはね 2=まだらのはね
    Cicada(u8),
    /// 水のカメムシ 0=ミズカマキリ 1=タイコウチ 2=コオイムシ
    WaterBug(u8),
    Strider,
    StinkBug,
    /// バッタ 0=トノサマバッタ 1=とがった頭 2=キリギリス 3=コオロギ
    Hopper(u8),
    /// カマキリ 0=ほっそり 1=はばひろ
    Mantis(u8),
    Stick,
    /// ハチ 0=ミツバチ 1=スズメバチ 2=クマバチ
    Bee(u8),
    Ant,
    Earwig,
}

impl Art {
    /// 木の幹にとまる虫（とまるとき頭を上にする）
    pub fn trunk(self) -> bool {
        matches!(self, Art::Cicada(_) | Art::Scarab(0) | Art::Scarab(1) | Art::Stag(_) | Art::Jewel | Art::Longhorn | Art::Stick)
    }
    /// はねを動かして飛ぶ絵か
    pub fn flies(self) -> bool {
        matches!(self, Art::Butterfly(_) | Art::Dragonfly(_) | Art::Bee(_))
    }
}

pub struct Species {
    /// 画像の名前（assets/artwork/<slug>.jpg・assets/units/<slug>.png があれば使う）
    pub slug: &'static str,
    pub name: L,
    pub sci: &'static str,
    pub group: Group,
    /// 科の名前
    pub family: L,
    /// 体長（cm。チョウ・ガは前ばねの長さ）
    pub len_cm: f32,
    /// すむ環境（earth の FOREST など）
    pub hab: u16,
    /// 会えるスポット（空なら、すむ環境のあるすべてのスポット）
    pub spots: &'static [usize],
    pub beh: Beh,
    pub art: Art,
    pub col: [f32; 3],
    pub col2: [f32; 3],
    /// 1場面あたりの数の目安
    pub rarity: f32,
    /// この距離より近づくと逃げる
    pub shy: f32,
    /// 動く速さ
    pub speed: f32,
    /// 成虫が見られる月（はじめ, おわり）。はじめ > おわり なら年をまたぐ
    pub months: (u8, u8),
    /// 活動する時間帯（DAWN・DAY・DUSK・NIGHT の組み合わせ）
    pub times: u8,
    pub desc: L,
    pub fact: L,
}

/// 活動しない時間帯に休んでいる場所
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Rest {
    /// 葉の裏や草のかげで、はねを閉じて休む（チョウ・テントウムシ・ホタル）
    UnderLeaf,
    /// 草の茎や小枝の先にとまって休む（トンボ・バッタ・カマキリ）
    Roost,
    /// 木の幹や枝にとまったまま（セミ・カミキリ）
    Trunk,
    /// 木の根元の落ち葉や土の中にもぐる（カブトムシ・クワガタ）
    Litter,
    /// 草の根元や石のかげにひそむ（コオロギ）
    Hideout,
    /// 巣の中・巣の入り口（ハチ・アリ）
    Nest,
    /// 岸の草や浮き葉のかげ（アメンボ）
    WaterEdge,
}

impl Rest {
    pub fn text(self) -> &'static str {
        match self {
            Rest::UnderLeaf => t!("葉の裏や草のかげで、はねを閉じてじっと休んでいる", "resting with wings closed under leaves or in the grass", "合着翅膀在叶子背面或草丛阴影里静静休息"),
            Rest::Roost => t!("草の茎や小枝の先にとまって、じっと休んでいる", "resting still on grass stems or twig tips", "停在草茎或小树枝尖上静静休息"),
            Rest::Trunk => t!("木の幹や枝にとまったまま、じっとしている", "staying still on tree trunks and branches", "停在树干或树枝上一动不动"),
            Rest::Litter => t!("木の根元の落ち葉や土の中にもぐっている", "burrowed into fallen leaves or soil at the foot of trees", "钻进树根旁的落叶或土里"),
            Rest::Hideout => t!("草の根元や石のかげにひそんでいる", "hiding at the base of plants or behind stones", "躲在草根或石头后面"),
            Rest::Nest => t!("巣の中や、巣の入り口で休んでいる", "resting in or at the entrance of the nest", "在巢里或巢口休息"),
            Rest::WaterEdge => t!("岸の草や浮き葉のかげで休んでいる", "resting among bank plants and floating leaves", "在岸边的草或浮叶阴影里休息"),
        }
    }
}

/// 成虫の季節でないときの姿（冬越しの姿）
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stage {
    /// たまご
    Egg,
    /// たまごのかたまり（カマキリの卵のう）
    EggCase,
    /// イモムシ・ケムシ（チョウ・ガの幼虫）
    Caterpillar,
    /// 丸まった白い幼虫（コガネムシ・クワガタ）
    Grub,
    /// まっすぐな白い幼虫（カミキリムシ・タマムシ）
    Borer,
    /// ヤゴ（トンボの幼虫）
    Yago,
    /// 土の中のセミの幼虫
    CicadaNymph,
    /// ホタルの幼虫（水の中）
    FireflyLarva,
    /// 小さなカメムシの幼虫
    BugNymph,
    /// チョウのさなぎ
    Chrysalis,
    /// ガのまゆ・土の中のさなぎ
    Cocoon,
    /// 成虫のまま冬眠
    Adult,
}

impl Stage {
    pub fn name(self) -> &'static str {
        match self {
            Stage::Egg => t!("たまご", "eggs", "卵"),
            Stage::EggCase => t!("たまごのかたまり（卵のう）", "egg case", "卵鞘"),
            Stage::Caterpillar | Stage::Grub | Stage::Borer | Stage::CicadaNymph | Stage::FireflyLarva | Stage::BugNymph => t!("幼虫", "larva", "幼虫"),
            Stage::Yago => t!("幼虫（ヤゴ）", "nymph", "稚虫（水虿）"),
            Stage::Chrysalis | Stage::Cocoon => t!("さなぎ", "pupa", "蛹"),
            Stage::Adult => t!("成虫", "adult", "成虫"),
        }
    }
    /// 成虫の絵の大きさに対する、絵の大きさ
    pub fn scale(self) -> f32 {
        match self {
            Stage::Egg => 0.5,
            Stage::EggCase | Stage::Chrysalis | Stage::Cocoon => 0.6,
            Stage::BugNymph => 0.7,
            Stage::Adult => 1.0,
            _ => 0.85,
        }
    }
}

/// 冬越しの場所
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Site {
    /// 土の中
    Soil,
    /// 落ち葉の下
    Litter,
    /// 朽ち木・枯れ木の中
    DeadWood,
    /// 草の茎・枝
    Stem,
    /// 木の幹・樹皮のすきま
    Bark,
    /// 水の底
    WaterBottom,
    /// 巣の中
    Nest,
    /// 石の下
    UnderStone,
    /// 水辺の岸（落ち葉や草の根元）
    Bank,
}

impl Site {
    pub fn text(self) -> &'static str {
        match self {
            Site::Soil => t!("土の中", "in the soil", "土里"),
            Site::Litter => t!("落ち葉の下", "under fallen leaves", "落叶下"),
            Site::DeadWood => t!("朽ち木・枯れ木の中", "inside rotting or dead wood", "朽木·枯木里"),
            Site::Stem => t!("草の茎や枝", "on grass stems and twigs", "草茎和树枝上"),
            Site::Bark => t!("木の幹や樹皮のすきま", "on tree trunks and in bark crevices", "树干和树皮缝隙里"),
            Site::WaterBottom => t!("水の底", "at the bottom of the water", "水底"),
            Site::Nest => t!("巣の中", "inside the nest", "巢里"),
            Site::UnderStone => t!("石の下", "under stones", "石头下"),
            Site::Bank => t!("水辺の岸の落ち葉や草の根元", "under leaves and plants on the bank", "岸边的落叶和草根下"),
        }
    }
    /// その場面にこの場所があるか
    pub fn fits(self, hab: Habitat) -> bool {
        match self {
            Site::WaterBottom => hab.is_water(),
            Site::Soil | Site::Litter | Site::DeadWood | Site::UnderStone => !hab.is_water(),
            Site::Bark => hab.has_trees(),
            Site::Stem | Site::Nest | Site::Bank => true,
        }
    }
}

/// 時間帯 0=早朝 1=昼 2=夕方 3=夜（bit で組み合わせる）
pub const DAWN: u8 = 1;
pub const DAY: u8 = 2;
pub const DUSK: u8 = 4;
pub const NIGHT: u8 = 8;
pub const ALL_DAY: u8 = 15;

pub fn time_name(time: u8) -> &'static str {
    [t!("早朝", "Early morning", "清晨"), t!("昼", "Daytime", "白天"), t!("夕方", "Evening", "傍晚"), t!("夜", "Night", "夜晚")][time as usize % 4]
}

/// 時間帯の組み合わせの文（「夕方・夜」）
pub fn times_text(bits: u8) -> String {
    if bits == ALL_DAY {
        return t!("一日じゅう", "All day and night", "全天").to_string();
    }
    (0..4u8).filter(|t| bits & (1 << t) != 0).map(time_name).collect::<Vec<_>>().join(crate::lang::sep())
}

/// 季節 0=春（3〜5月） 1=夏（6〜8月） 2=秋（9〜11月） 3=冬（12〜2月）
pub fn season_of(month: u8) -> usize {
    match month {
        3..=5 => 0,
        6..=8 => 1,
        9..=11 => 2,
        _ => 3,
    }
}

pub fn season_name(month: u8) -> &'static str {
    [t!("春", "Spring", "春"), t!("夏", "Summer", "夏"), t!("秋", "Autumn", "秋"), t!("冬", "Winter", "冬")][season_of(month)]
}

pub fn month_name(month: u8) -> String {
    const EN: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    match crate::lang::lang() {
        crate::lang::Lang::En => EN[(month as usize + 11) % 12].to_string(),
        _ => format!("{}月", month),
    }
}

/// 月の範囲の文（「7〜9月」）
pub fn months_text(m: (u8, u8)) -> String {
    if m == (1, 12) {
        return t!("一年じゅう（冬は集まってじっとしている）", "All year (gathered and still in winter)", "全年（冬天聚在一起不动）").to_string();
    }
    if m.0 == m.1 {
        return month_name(m.0);
    }
    match crate::lang::lang() {
        crate::lang::Lang::En => format!("{}–{}", month_name(m.0), month_name(m.1)),
        _ => format!("{}〜{}月", m.0, m.1),
    }
}

impl Species {
    /// その月に成虫が見られるか
    pub fn in_season(&self, month: u8) -> bool {
        let (a, b) = self.months;
        if a <= b {
            (a..=b).contains(&month)
        } else {
            month >= a || month <= b
        }
    }

    /// 活動しない時間帯に休んでいる場所
    pub fn rest(&self) -> Rest {
        if self.slug == "winter-moth" {
            return Rest::Trunk;
        }
        match self.art {
            Art::Butterfly(_) | Art::Ladybug | Art::Firefly | Art::StinkBug => Rest::UnderLeaf,
            Art::Dragonfly(_) | Art::Mantis(_) | Art::Hopper(0) | Art::Hopper(1) | Art::Hopper(2) => Rest::Roost,
            Art::Longhorn | Art::Jewel | Art::Cicada(_) | Art::Stick => Rest::Trunk,
            Art::Scarab(_) | Art::Stag(_) | Art::TigerBeetle => Rest::Litter,
            Art::Hopper(_) | Art::Earwig | Art::DivingBeetle | Art::WaterBug(_) => Rest::Hideout,
            Art::Bee(_) | Art::Ant => Rest::Nest,
            Art::Strider => Rest::WaterEdge,
        }
    }

    /// 成虫の季節でないときの姿と場所。None なら関東では冬を越せない（毎年南から飛んでくる）
    pub fn winter(&self) -> Option<(Stage, Site)> {
        use Site::*;
        use Stage::*;
        Some(match self.slug {
            "cabbage-white" | "asian-swallowtail" | "old-world-swallowtail" | "spangle" => (Chrysalis, Bark),
            "common-bluebottle" => (Chrysalis, Stem),
            "eastern-pale-clouded-yellow" | "pale-grass-blue" | "small-copper" | "indian-fritillary" => (Caterpillar, Litter),
            "great-purple-emperor" | "hestina-persimilis" => (Caterpillar, Litter),
            "red-ring-skirt" => (Caterpillar, Bark),
            "blue-admiral" => (Adult, Litter),
            "chestnut-tiger" => (Caterpillar, Stem),
            "japanese-luna-moth" => (Cocoon, Litter),
            "winter-moth" => (Cocoon, Soil),
            "autumn-darter" => (Egg, WaterBottom),
            "wandering-glider" => return None,
            "japanese-rhinoceros-beetle" | "kanabun" | "miyama-stag-beetle" => (Grub, Soil),
            "saw-stag-beetle" => (Grub, DeadWood),
            "dorcus-rectus" => (Adult, DeadWood),
            "genji-firefly" | "heike-firefly" => (FireflyLarva, WaterBottom),
            "chinese-diving-beetle" | "water-stick-insect" | "water-scorpion" | "giant-water-bug-appasus" => (Adult, WaterBottom),
            "rosalia-batesi" | "white-spotted-longicorn" | "japanese-jewel-beetle" => (Borer, DeadWood),
            "tiger-beetle" | "dor-beetle" => (Adult, Soil),
            "pond-skater" => (Adult, Bank),
            "striped-shield-bug" => (BugNymph, Litter),
            "migratory-locust" | "oriental-longheaded-grasshopper" | "smaller-longheaded-grasshopper" | "emma-field-cricket" | "japanese-katydid" | "bell-cricket" => (Egg, Soil),
            "chinese-mantis" | "giant-asian-mantis" => (EggCase, Stem),
            "small-mantis" => (EggCase, UnderStone),
            "stick-insect" => (Egg, Litter),
            "western-honey-bee" | "japanese-honey-bee" | "carpenter-bee" | "japanese-carpenter-ant" => (Adult, Nest),
            "asian-giant-hornet" | "yellow-hornet" => (Adult, DeadWood),
            "maritime-earwig" => (Adult, UnderStone),
            _ => match self.art {
                Art::Cicada(_) => (CicadaNymph, Soil),
                Art::Dragonfly(_) => (Yago, WaterBottom),
                Art::Ladybug => (Adult, Litter),
                _ => (Adult, Litter),
            },
        })
    }

    /// その時間帯（0〜3）に活動しているか
    pub fn active_at(&self, time: u8) -> bool {
        self.times & (1 << (time % 4)) != 0
    }

    /// 画面に描く長さ。小さな虫ほど大きめに描いて見つけやすくする
    pub fn disp_len(&self) -> f32 {
        (0.30 * (self.len_cm / 5.0).powf(0.45)).max(0.12)
    }
}

#[allow(clippy::too_many_arguments)]
const fn sp(
    slug: &'static str, name: L, sci: &'static str, group: Group, family: L, len_cm: f32, hab: u16, spots: &'static [usize],
    beh: Beh, art: Art, col: [f32; 3], col2: [f32; 3], rarity: f32, shy: f32, speed: f32, months: (u8, u8), times: u8, desc: L, fact: L,
) -> Species {
    Species { slug, name, sci, group, family, len_cm, hab, spots, beh, art, col, col2, rarity, shy, speed, months, times, desc, fact }
}

use Beh::*;
use Group::{Bee as GBee, Beetle as GBeetle, Bug as GBug, Butterfly as GButterfly, Dragonfly as GDragonfly, Grasshopper as GHopper};

const PIERIDAE: L = L("シロチョウ科", "Pieridae", "粉蝶科");
const PAPILIONIDAE: L = L("アゲハチョウ科", "Papilionidae", "凤蝶科");
const NYMPHALIDAE: L = L("タテハチョウ科", "Nymphalidae", "蛱蝶科");
const LYCAENIDAE: L = L("シジミチョウ科", "Lycaenidae", "灰蝶科");
const LIBELLULIDAE: L = L("トンボ科", "Libellulidae", "蜻科");
const AESHNIDAE: L = L("ヤンマ科", "Aeshnidae", "蜓科");
const SCARABAEIDAE: L = L("コガネムシ科", "Scarabaeidae", "金龟科");
const LUCANIDAE: L = L("クワガタムシ科", "Lucanidae", "锹甲科");
const CICADIDAE: L = L("セミ科", "Cicadidae", "蝉科");
const ACRIDIDAE: L = L("バッタ科", "Acrididae", "蝗科");
const MANTIDAE: L = L("カマキリ科", "Mantidae", "螳科");
const APIDAE: L = L("ミツバチ科", "Apidae", "蜜蜂科");
const VESPIDAE: L = L("スズメバチ科", "Vespidae", "胡蜂科");


pub static SPECIES: &[Species] = &[
    // ---------------------------------------------------------------- チョウ・ガ
    sp("cabbage-white", L("モンシロチョウ", "Small white", "菜粉蝶"), "Pieris rapae", GButterfly, PIERIDAE, 2.8, MEADOW | PARK | PADDY, &[],
        Flutter, Art::Butterfly(0), [0.97, 0.97, 0.93], [0.20, 0.20, 0.22], 4.0, 1.2, 0.5, (3, 11), DAY,
        L("白いはねに黒い点のあるチョウ。キャベツなどアブラナの仲間の畑や、花だんのまわりを飛ぶ。", "A white butterfly with black spots, fluttering over cabbage fields and flowerbeds.", "白色翅膀上带黑点的蝴蝶，在卷心菜等十字花科田地和花坛周围飞舞。"),
        L("幼虫は「青虫」とよばれ、キャベツの葉を食べて育つ。", "Its caterpillar, the familiar green 'cabbage worm', grows up eating cabbage leaves.", "幼虫被称为“青虫”，以卷心菜叶为食长大。")),
    sp("eastern-pale-clouded-yellow", L("モンキチョウ", "Eastern pale clouded yellow", "斑缘豆粉蝶"), "Colias erate", GButterfly, PIERIDAE, 2.6, MEADOW | PADDY, &[],
        Flutter, Art::Butterfly(0), [0.96, 0.90, 0.42], [0.30, 0.26, 0.16], 3.0, 1.2, 0.55, (3, 11), DAY,
        L("黄色いはねのふちが黒いチョウ。シロツメクサの咲く草はらや河原を低く飛ぶ。", "A yellow butterfly with dark wing edges that flies low over clover meadows and riverbanks.", "黄色翅膀镶黑边的蝴蝶，在开着白车轴草的草地和河滩上低飞。"),
        L("メスには白っぽいものもいて、モンシロチョウとまちがえやすい。", "Some females are whitish and easily mistaken for small whites.", "有些雌蝶偏白色，容易被误认为菜粉蝶。")),
    sp("asian-swallowtail", L("ナミアゲハ", "Asian swallowtail", "柑橘凤蝶"), "Papilio xuthus", GButterfly, PAPILIONIDAE, 4.5, PARK | MEADOW | FOREST, &[],
        Flutter, Art::Butterfly(1), [0.96, 0.90, 0.60], [0.10, 0.10, 0.10], 2.5, 1.5, 0.6, (4, 10), DAWN | DAY,
        L("黄色と黒のしま模様の大きなチョウ。町の庭や公園でもよく見られる、いちばん身近なアゲハ。", "A large yellow-and-black striped butterfly, the most familiar swallowtail in town gardens and parks.", "黄黑条纹的大型蝴蝶，是城市庭院和公园里最常见的凤蝶。"),
        L("幼虫はミカンやサンショウの葉を食べる。おどろくと頭から臭い角を出す。", "Its caterpillars eat citrus and Japanese pepper leaves, and push out a smelly orange horn when startled.", "幼虫吃柑橘和花椒的叶子，受惊时会从头部伸出有臭味的角。")),
    sp("old-world-swallowtail", L("キアゲハ", "Old World swallowtail", "金凤蝶"), "Papilio machaon", GButterfly, PAPILIONIDAE, 4.8, MEADOW | MOUNTAIN, &[],
        Flutter, Art::Butterfly(1), [0.98, 0.84, 0.36], [0.10, 0.10, 0.10], 2.0, 1.6, 0.65, (4, 10), DAY,
        L("ナミアゲハに似ているが、はねのつけ根が黒くぬりつぶされている。山頂や草原を元気に飛ぶ。", "Like the Asian swallowtail but with solid black wing bases. It patrols hilltops and grasslands.", "与柑橘凤蝶相似，但翅基部是全黑的。在山顶和草原上活跃飞行。"),
        L("オスは山のてっぺんに集まってメスを待つ「山頂占有」をする。", "Males gather on summits to wait for females, a habit called 'hilltopping'.", "雄蝶会聚集在山顶等待雌蝶，这叫“占山顶”。")),
    sp("spangle", L("クロアゲハ", "Spangle", "蓝凤蝶"), "Papilio protenor", GButterfly, PAPILIONIDAE, 6.0, FOREST | PARK, &[],
        Flutter, Art::Butterfly(1), [0.12, 0.12, 0.14], [0.80, 0.25, 0.25], 1.6, 1.6, 0.6, (4, 9), DAWN | DAY | DUSK,
        L("黒くて大きなアゲハ。後ろばねに赤い斑点がある。木かげの多い林のふちや庭を、ゆったり飛ぶ。", "A big black swallowtail with red spots on the hindwings, gliding along shady woodland edges and gardens.", "黑色大型凤蝶，后翅有红色斑点。在树荫多的林缘和庭院中悠然飞舞。"),
        L("チョウには、決まった道すじを通って飛ぶ「チョウ道」がある。", "Swallowtails follow regular flight routes called 'butterfly roads'.", "蝴蝶有固定的飞行路线，叫“蝶道”。")),
    sp("common-bluebottle", L("アオスジアゲハ", "Common bluebottle", "青凤蝶"), "Graphium sarpedon", GButterfly, PAPILIONIDAE, 4.2, PARK | FOREST, &[],
        Flutter, Art::Butterfly(2), [0.10, 0.12, 0.14], [0.35, 0.80, 0.90], 2.0, 1.8, 0.9, (5, 10), DAY,
        L("黒いはねに、すき通った青緑の帯が走るアゲハ。とても速く飛び、クスノキの多い公園でよく見る。", "A swallowtail with a translucent blue-green band across black wings. Fast-flying, it favours parks full of camphor trees.", "黑色翅膀上有一条半透明蓝绿色带的凤蝶。飞得很快，常见于樟树多的公园。"),
        L("幼虫はクスノキの葉を食べる。街路樹のおかげで都会でもふえた。", "Its caterpillars eat camphor leaves, so street trees have helped it thrive in cities.", "幼虫吃樟树叶，多亏行道树，它在城市里也增多了。")),
    sp("great-purple-emperor", L("オオムラサキ", "Great purple emperor", "大紫蛱蝶"), "Sasakia charonda", GButterfly, NYMPHALIDAE, 5.3, FOREST, &[S_RANZAN, S_SAYAMA, S_TAKAO],
        Flutter, Art::Butterfly(2), [0.42, 0.26, 0.62], [0.95, 0.92, 0.80], 0.8, 2.2, 0.8, (6, 8), DAWN | DAY,
        L("日本の国蝶。オスのはねは光の角度で青むらさきにかがやく。クヌギの樹液に集まり、力強く飛ぶ。", "Japan's national butterfly. The male's wings flash violet in the light. It feeds at oak sap and flies powerfully.", "日本的国蝶。雄蝶翅膀随光线角度闪耀蓝紫色。聚集在麻栎树液处，飞行有力。"),
        L("幼虫はエノキの葉で育ち、冬は落ち葉の下ですごす。", "Its caterpillars feed on hackberry leaves and spend the winter under fallen leaves.", "幼虫以朴树叶为食，冬天在落叶下度过。")),
    sp("hestina-persimilis", L("ゴマダラチョウ", "Hestina persimilis", "黑脉蛱蝶"), "Hestina persimilis japonica", GButterfly, NYMPHALIDAE, 4.3, FOREST, &[],
        Flutter, Art::Butterfly(2), [0.94, 0.94, 0.90], [0.12, 0.12, 0.12], 1.2, 1.8, 0.6, (5, 9), DAY,
        L("白と黒のまだら模様のチョウ。雑木林のクヌギの樹液に、カブトムシといっしょに来る。", "A black-and-white mottled butterfly that visits oak sap alongside rhinoceros beetles.", "黑白斑驳的蝴蝶，和独角仙一起来到麻栎的树液处。"),
        L("オオムラサキと同じくエノキで育つ、近い仲間。", "A close relative of the purple emperor that also grows up on hackberry.", "和大紫蛱蝶一样以朴树为食，是近亲。")),
    sp("red-ring-skirt", L("アカボシゴマダラ", "Red ring skirt", "红星斑蛱蝶"), "Hestina assimilis", GButterfly, NYMPHALIDAE, 4.5, PARK | FOREST, &[],
        Flutter, Art::Butterfly(2), [0.94, 0.94, 0.92], [0.80, 0.18, 0.16], 1.3, 1.8, 0.6, (5, 10), DAY,
        L("白黒のはねに赤い点が並ぶチョウ。関東では1990年代から急にふえ、公園でもよく見る。", "A black-and-white butterfly with a row of red spots. It suddenly spread through Kanto from the 1990s.", "黑白翅膀上排列着红点的蝴蝶。1990年代起在关东急剧增多。"),
        L("もとは日本の本州にいなかったチョウで、人が放したものがふえたと考えられている。", "It was not native to Honshu; released individuals are thought to have founded the population.", "原本不分布于本州，被认为是人为放生后繁殖起来的。")),
    sp("blue-admiral", L("ルリタテハ", "Blue admiral", "琉璃蛱蝶"), "Kaniska canace", GButterfly, NYMPHALIDAE, 3.2, FOREST | PARK, &[],
        Flutter, Art::Butterfly(2), [0.10, 0.10, 0.14], [0.35, 0.65, 0.95], 1.4, 1.8, 0.8, (3, 11), DAY,
        L("黒いはねに、るり色（青）の帯があるチョウ。はねの裏は木の皮そっくりの地味な色。", "A black butterfly with a vivid blue band; the underside looks just like tree bark.", "黑色翅膀上有琉璃蓝色带的蝴蝶，翅膀反面是与树皮一模一样的朴素颜色。"),
        L("成虫のまま冬をこし、早春の暖かい日に飛び出す。", "It overwinters as an adult and flies out on warm days in early spring.", "以成虫越冬，早春暖和的日子就会飞出来。")),
    sp("pale-grass-blue", L("ヤマトシジミ", "Pale grass blue", "酢浆灰蝶"), "Zizeeria maha", GButterfly, LYCAENIDAE, 1.3, PARK | MEADOW, &[],
        Flutter, Art::Butterfly(3), [0.62, 0.72, 0.92], [0.85, 0.85, 0.85], 4.0, 1.0, 0.4, (3, 11), DAY | DUSK,
        L("はねの表が水色の小さなチョウ。道ばたや公園のカタバミのまわりにいつもいる。", "A tiny butterfly with pale blue uppersides, always found around wood sorrel on roadsides and in parks.", "翅膀正面呈浅蓝色的小蝴蝶，总是在路边和公园的酢浆草周围。"),
        L("幼虫はカタバミだけを食べる。町でいちばん見つけやすいシジミチョウ。", "Its caterpillars eat only wood sorrel. It's the easiest blue to find in town.", "幼虫只吃酢浆草，是城市里最容易找到的灰蝶。")),
    sp("small-copper", L("ベニシジミ", "Small copper", "红灰蝶"), "Lycaena phlaeas", GButterfly, LYCAENIDAE, 1.5, MEADOW | PADDY, &[],
        Flutter, Art::Butterfly(3), [0.95, 0.50, 0.15], [0.25, 0.20, 0.18], 3.0, 1.0, 0.45, (3, 11), DAY,
        L("オレンジ色と黒の小さなチョウ。春の田んぼのあぜや草はらで花にとまる。", "A small orange-and-black butterfly that visits flowers on paddy ridges and in meadows.", "橙黑色的小蝴蝶，春天停在稻田田埂和草地的花上。"),
        L("幼虫はスイバやギシギシを食べる。", "Its caterpillars eat sorrel and dock.", "幼虫吃酸模和羊蹄。")),
    sp("chestnut-tiger", L("アサギマダラ", "Chestnut tiger", "大绢斑蝶"), "Parantica sita", GButterfly, NYMPHALIDAE, 5.2, MOUNTAIN, &[S_TAKAO, S_NIKKO, S_TSUKUBA],
        Flutter, Art::Butterfly(4), [0.70, 0.85, 0.92], [0.40, 0.22, 0.14], 1.2, 1.4, 0.5, (5, 10), DAWN | DAY,
        L("すき通った浅葱色（うすい青緑）のはねのチョウ。夏は涼しい山にいて、ふわふわと飛ぶ。", "A butterfly with translucent pale blue-green wings that drifts through cool mountain forests in summer.", "有半透明浅葱色（浅蓝绿）翅膀的蝴蝶，夏天在凉爽的山中轻飘飘地飞。"),
        L("秋には海をこえて1000km以上も南へ旅する「渡りをするチョウ」。", "In autumn it migrates over 1,000 km south, even across the sea.", "秋天会越过大海向南迁徙1000公里以上，是“会迁徙的蝴蝶”。")),
    sp("indian-fritillary", L("ツマグロヒョウモン", "Indian fritillary", "斐豹蛱蝶"), "Argynnis hyperbius", GButterfly, NYMPHALIDAE, 3.8, PARK | MEADOW, &[],
        Flutter, Art::Butterfly(2), [0.95, 0.60, 0.22], [0.12, 0.12, 0.14], 2.0, 1.4, 0.55, (4, 11), DAY,
        L("オレンジ色にヒョウ柄のチョウ。メスははねの先が黒い。花だんのパンジーで育つ。", "An orange, leopard-spotted butterfly; females have black wingtips. It breeds on garden pansies.", "橙色豹纹的蝴蝶，雌蝶翅尖是黑色的。在花坛的三色堇上长大。"),
        L("もとは西日本のチョウだったが、温暖化と花だんのスミレで関東にもふえた。", "Once a western Japanese species, it has spread into Kanto with warming and garden violas.", "原本是西日本的蝴蝶，因气候变暖和花坛的堇菜在关东也增多了。")),
    sp("japanese-luna-moth", L("オオミズアオ", "Japanese luna moth", "绿尾大蚕蛾"), "Actias aliena", GButterfly, L("ヤママユガ科", "Saturniidae", "大蚕蛾科"), 5.5, FOREST | MOUNTAIN, &[],
        Flutter, Art::Butterfly(5), [0.72, 0.92, 0.80], [0.60, 0.40, 0.30], 0.9, 1.2, 0.35, (5, 8), NIGHT,
        L("うす緑色の大きなガ。後ろばねに長い尾がある。夜の森を静かに飛ぶ。", "A large pale-green moth with long hindwing tails that flies quietly through forests at night.", "淡绿色的大蛾子，后翅有长尾突，在夜晚的森林中静静飞翔。"),
        L("成虫には口がなく、何も食べずに1週間ほどで一生を終える。", "Adults have no mouth; they eat nothing and live only about a week.", "成虫没有口器，什么也不吃，大约一周就结束一生。")),
    sp("winter-moth", L("シロオビフユシャク", "Japanese winter moth", "白带冬尺蛾"), "Alsophila japonensis", GButterfly, L("シャクガ科", "Geometridae", "尺蛾科"), 1.4, FOREST | PARK, &[],
        Flutter, Art::Butterfly(0), [0.62, 0.58, 0.50], [0.86, 0.83, 0.74], 2.5, 1.0, 0.35, (12, 2), DUSK | NIGHT,
        L("真冬に現れる小さなガ。寒い日でも、オスは雑木林や公園の木のまわりをふわふわと飛ぶ。", "A small moth that appears in midwinter. Even on cold days, males flutter around trees in woods and parks.", "在隆冬出现的小蛾子。即使天气寒冷，雄蛾也会在杂木林和公园的树木周围轻飘飘地飞。"),
        L("メスははねが退化していて飛べない。冬に活動するのは、鳥などの天敵が少ないからと考えられている。", "Females have tiny wings and cannot fly. Being active in winter may help it avoid birds and other enemies.", "雌蛾翅膀退化，不能飞。人们认为它在冬天活动，是因为鸟类等天敌较少。")),
    // ---------------------------------------------------------------- トンボ
    sp("common-skimmer", L("シオカラトンボ", "White-tailed skimmer", "白尾灰蜻"), "Orthetrum albistylum", GDragonfly, LIBELLULIDAE, 5.3, POND | PADDY | PARK | RIVER, &[],
        Fly, Art::Dragonfly(0), [0.62, 0.72, 0.82], [0.12, 0.12, 0.14], 3.0, 1.5, 1.2, (4, 10), DAWN | DAY,
        L("オスは体が白っぽい水色、メスは黄色っぽい（ムギワラトンボ）。池や田んぼのまわりでいちばん身近なトンボ。", "Males are powdery blue-white and females yellowish. The most familiar dragonfly around ponds and paddies.", "雄性身体呈灰白蓝色，雌性偏黄色（叫“麦秆蜻蜓”）。是池塘和稻田周围最常见的蜻蜓。"),
        L("オスの白い粉は、大人になると体から出てくる。", "The male's white powder appears as it matures.", "雄性的白粉是成熟后从身体分泌出来的。")),
    sp("autumn-darter", L("アキアカネ", "Autumn darter", "秋赤蜻"), "Sympetrum frequens", GDragonfly, LIBELLULIDAE, 4.0, PADDY | MEADOW | MOUNTAIN, &[],
        Fly, Art::Dragonfly(0), [0.86, 0.32, 0.18], [0.30, 0.22, 0.16], 3.0, 1.3, 1.0, (6, 11), DAY | DUSK,
        L("秋に赤くなる「赤とんぼ」の代表。田んぼで生まれ、暑い夏は高い山ですごす。", "The classic 'red dragonfly' of autumn. Born in rice paddies, it spends the hot summer high in the mountains.", "秋天变红的“红蜻蜓”代表。在稻田出生，炎热的夏天在高山度过。"),
        L("秋になると山から平地の田んぼへ大群でもどってくる。", "In autumn great swarms return from the mountains to lowland paddies.", "到了秋天会成群结队从山上回到平原的稻田。")),
    sp("lesser-emperor", L("ギンヤンマ", "Lesser emperor", "碧伟蜓"), "Anax parthenope", GDragonfly, AESHNIDAE, 7.0, POND | PADDY, &[],
        Fly, Art::Dragonfly(1), [0.40, 0.70, 0.38], [0.30, 0.60, 0.85], 1.5, 2.0, 1.8, (4, 10), DAY | DUSK,
        L("緑の胸と、オスは水色の腰が目立つ大きなトンボ。池の上を行ったり来たりしてなわばりを見張る。", "A large dragonfly with a green thorax; males have a sky-blue waist. It patrols ponds, guarding its territory.", "胸部绿色、雄性腰部天蓝色的大型蜻蜓。在池塘上来回巡逻守卫领地。"),
        L("昔の子どもたちのトンボとりで、いちばんの人気者だった。", "It was the top prize of children's dragonfly hunts in the old days.", "过去孩子们捉蜻蜓时最受欢迎的就是它。")),
    sp("golden-ringed-dragonfly", L("オニヤンマ", "Jumbo dragonfly", "无霸勾蜓"), "Anotogaster sieboldii", GDragonfly, L("オニヤンマ科", "Cordulegastridae", "大蜓科"), 10.0, RIVER | MOUNTAIN | FOREST, &[S_TAKAO, S_RANZAN, S_NAGATORO, S_YORO, S_TSUKUBA, S_SAYAMA],
        Fly, Art::Dragonfly(1), [0.12, 0.12, 0.10], [0.95, 0.82, 0.20], 1.0, 2.2, 1.6, (6, 9), DAWN | DAY,
        L("日本でいちばん大きなトンボ。黒と黄色のしま模様で、緑の大きな目をもつ。山道の小川ぞいを往復する。", "Japan's largest dragonfly, striped black and yellow with big green eyes, patrolling streams along mountain paths.", "日本最大的蜻蜓，黑黄条纹，有绿色大眼睛。沿着山路小溪来回飞行。"),
        L("幼虫（ヤゴ）は川底の砂にもぐって、3〜4年かけて大人になる。", "Its larva hides in streambed sand and takes three or four years to grow up.", "幼虫（水虿）钻进河底沙中，要花3到4年才能长大。")),
    sp("black-winged-damselfly", L("ハグロトンボ", "Black-winged damselfly", "黑色蟌"), "Atrocalopteryx atrata", GDragonfly, L("カワトンボ科", "Calopterygidae", "色蟌科"), 6.0, RIVER, &[],
        Flutter, Art::Dragonfly(3), [0.20, 0.50, 0.40], [0.08, 0.08, 0.08], 2.5, 1.2, 0.5, (5, 10), DAWN | DAY,
        L("真っ黒なはねをひらひらさせて、チョウのように飛ぶトンボ。オスの体は金属のような緑色。", "A damselfly that flutters its jet-black wings like a butterfly. Males have a metallic green body.", "扇动漆黑翅膀、像蝴蝶一样飞的豆娘。雄性身体呈金属绿色。"),
        L("流れのゆるい、水草の多い小川が好き。とまるとはねを閉じたり開いたりする。", "It prefers slow, weedy streams, and opens and closes its wings while perched.", "喜欢水流缓慢、水草多的小溪，停下时会开合翅膀。")),
    sp("butterfly-dragonfly", L("チョウトンボ", "Butterfly dragonfly", "黑丽翅蜻"), "Rhyothemis fuliginosa", GDragonfly, LIBELLULIDAE, 3.5, POND, &[S_WATARASE, S_KASUMIGAURA, S_UENO, S_INOKASHIRA],
        Flutter, Art::Dragonfly(4), [0.12, 0.12, 0.20], [0.30, 0.20, 0.70], 1.3, 1.4, 0.5, (6, 9), DAY,
        L("はばの広いはねが青むらさきに光るトンボ。チョウのようにひらひらと池の上を舞う。", "A dragonfly with broad, violet-shining wings that flutters over ponds like a butterfly.", "宽阔翅膀闪着蓝紫光的蜻蜓，像蝴蝶一样在池塘上空飞舞。"),
        L("水草がしげる、きれいな池や沼でしか見られない。", "It is found only at clean ponds and marshes with plenty of water plants.", "只出现在水草茂盛的清洁池塘和沼泽。")),
    sp("wandering-glider", L("ウスバキトンボ", "Wandering glider", "黄蜻"), "Pantala flavescens", GDragonfly, LIBELLULIDAE, 5.0, MEADOW | PARK | SHORE | PADDY, &[],
        Fly, Art::Dragonfly(0), [0.92, 0.70, 0.30], [0.50, 0.40, 0.25], 2.5, 1.6, 1.5, (6, 10), DAY | DUSK,
        L("夏の空を群れで飛び続けるオレンジ色のトンボ。お盆のころに多く「精霊トンボ」ともよばれる。", "An orange dragonfly that flies endlessly in summer swarms; called the 'spirit dragonfly' because it peaks at Obon.", "夏天成群不停飞翔的橙色蜻蜓。盂兰盆节前后特别多，也叫“精灵蜻蜓”。"),
        L("毎年、南の国から海をわたって日本へやってくる。冬はこせない。", "Every year it crosses the sea from the south; it cannot survive Japan's winter.", "每年从南方国家渡海来到日本，无法越冬。")),
    sp("asian-bluetail", L("アジアイトトンボ", "Asian bluetail", "东亚异痣蟌"), "Ischnura asiatica", GDragonfly, L("イトトンボ科", "Coenagrionidae", "蟌科"), 3.0, POND | PADDY, &[],
        Fly, Art::Dragonfly(2), [0.30, 0.65, 0.45], [0.30, 0.65, 0.95], 3.0, 0.9, 0.5, (4, 10), DAWN | DAY,
        L("糸のように細い小さなトンボ。オスは緑の胸と、しっぽの先の青が目印。田んぼの草の間を飛ぶ。", "A tiny, thread-thin damselfly; males have a green thorax and a blue tail tip. It flies between paddy grasses.", "像丝线一样细的小豆娘。雄性以绿色胸部和尾端的蓝色为标志，在稻田草间飞行。"),
        L("はねを閉じてとまるのがイトトンボの仲間の特ちょう。", "Damselflies rest with their wings folded together.", "停下时合起翅膀是豆娘类的特征。")),
    // ---------------------------------------------------------------- 甲虫
    sp("japanese-rhinoceros-beetle", L("カブトムシ", "Japanese rhinoceros beetle", "双叉犀金龟（独角仙）"), "Trypoxylus dichotomus", GBeetle, SCARABAEIDAE, 6.5, FOREST, &[],
        Perch, Art::Scarab(1), [0.30, 0.16, 0.10], [0.20, 0.12, 0.08], 1.2, 0.8, 0.10, (7, 8), DAWN | DUSK | NIGHT,
        L("夏の雑木林の王さま。オスには先が二つに分かれた大きな角がある。夜、クヌギの樹液に集まる。", "King of the summer woods. Males bear a great forked horn. They gather at oak sap at night.", "夏日杂木林之王。雄性有一个分叉的大角，夜里聚集在麻栎的树液处。"),
        L("幼虫は腐葉土を食べて育つ。角は樹液の場所をめぐるけんかに使う。", "The larva grows on leaf mould. The horn is used in fights over sap.", "幼虫吃腐叶土长大。角用于争夺树液的打斗。")),
    sp("saw-stag-beetle", L("ノコギリクワガタ", "Saw stag beetle", "锯锹甲"), "Prosopocoilus inclinatus", GBeetle, LUCANIDAE, 5.5, FOREST, &[],
        Perch, Art::Stag(1), [0.45, 0.20, 0.12], [0.30, 0.14, 0.08], 1.0, 0.8, 0.10, (6, 9), DAWN | DUSK | NIGHT,
        L("赤茶色の体に、のこぎりのようなギザギザの大あごをもつクワガタ。樹液の出る木で見つかる。", "A reddish-brown stag beetle with saw-toothed mandibles, found on sap-flowing trees.", "红褐色身体、有锯齿状大颚的锹甲，在流树液的树上能找到。"),
        L("大きなオスほどあごが大きく、強く曲がる。", "The bigger the male, the larger and more curved his jaws.", "雄虫越大，大颚越大越弯。")),
    sp("dorcus-rectus", L("コクワガタ", "Dorcus rectus", "小锹甲"), "Dorcus rectus", GBeetle, LUCANIDAE, 3.5, FOREST | PARK, &[],
        Perch, Art::Stag(0), [0.14, 0.12, 0.10], [0.10, 0.08, 0.08], 2.0, 0.7, 0.08, (5, 10), DAWN | DUSK | NIGHT,
        L("黒くて小型のクワガタ。いちばん出会いやすく、公園のクヌギやヤナギにもいる。", "A small black stag beetle, the easiest to find, even on park oaks and willows.", "黑色小型锹甲，最容易遇到，公园的麻栎和柳树上也有。"),
        L("成虫のまま木の中で冬をこし、2〜3年生きることもある。", "It overwinters as an adult inside trees and can live two or three years.", "以成虫在树中越冬，有时能活2到3年。")),
    sp("miyama-stag-beetle", L("ミヤマクワガタ", "Miyama stag beetle", "深山锹甲"), "Lucanus maculifemoratus", GBeetle, LUCANIDAE, 6.0, MOUNTAIN, &[],
        Perch, Art::Stag(2), [0.50, 0.32, 0.18], [0.70, 0.55, 0.30], 0.9, 0.8, 0.10, (7, 8), DAWN | DAY | NIGHT,
        L("頭に耳のような出っぱりがある大型のクワガタ。体に金色の毛が生える。すずしい山の林にすむ。", "A large stag beetle with ear-like bumps on its head and golden hairs, living in cool mountain woods.", "头部有耳朵状突起的大型锹甲，身上长着金色细毛，生活在凉爽的山林里。"),
        L("暑さに弱く、気温の高い平地ではあまり見られない。", "It dislikes heat and is rare in warm lowlands.", "怕热，在气温高的平原很少见。")),
    sp("kanabun", L("カナブン", "Kanabun flower chafer", "日本金龟（青铜花金龟）"), "Pseudotorynorrhina japonica", GBeetle, SCARABAEIDAE, 2.6, FOREST | PARK, &[],
        Perch, Art::Scarab(0), [0.42, 0.40, 0.18], [0.30, 0.50, 0.30], 2.0, 0.8, 0.10, (6, 8), DAWN | DAY | DUSK,
        L("金属のようにかがやく茶色や緑の甲虫。樹液に頭をつっこんでなめている。", "A metallic brown or green beetle that buries its head in tree sap.", "闪着金属光泽的褐色或绿色甲虫，把头钻进树液里舔食。"),
        L("前ばねを閉じたまま、すき間から後ろばねを出して飛べる。", "It can fly with its wing cases closed, sliding the hindwings out from the sides.", "能在不打开前翅的情况下，从缝隙伸出后翅飞行。")),
    sp("seven-spot-ladybird", L("ナナホシテントウ", "Seven-spot ladybird", "七星瓢虫"), "Coccinella septempunctata", GBeetle, L("テントウムシ科", "Coccinellidae", "瓢虫科"), 0.8, MEADOW | PARK, &[],
        Crawl, Art::Ladybug, [0.90, 0.16, 0.10], [0.06, 0.06, 0.06], 3.0, 0.5, 0.06, (1, 12), DAWN | DAY,
        L("赤い背中に7つの黒い点があるテントウムシ。草の上でアブラムシを食べる。", "A ladybird with seven black spots on a red back, eating aphids on plants.", "红色背上有七个黑点的瓢虫，在草上吃蚜虫。"),
        L("草の先まで登ると、そこから飛び立つくせがある。", "It climbs to the tip of a stem before taking off.", "有爬到草尖再飞起来的习惯。")),
    sp("harlequin-ladybird", L("ナミテントウ", "Asian lady beetle", "异色瓢虫"), "Harmonia axyridis", GBeetle, L("テントウムシ科", "Coccinellidae", "瓢虫科"), 0.7, PARK | FOREST, &[],
        Crawl, Art::Ladybug, [0.95, 0.55, 0.12], [0.08, 0.06, 0.06], 3.0, 0.5, 0.06, (1, 12), DAWN | DAY,
        L("模様がとても多いテントウムシ。黒地に赤い点のもの、オレンジに黒い点のものなどがいる。", "A ladybird of many patterns: red spots on black, black spots on orange and more.", "花纹非常多样的瓢虫，有黑底红点的，也有橙底黑点的。"),
        L("模様のちがいは親から子へ受けつがれる遺伝のしくみで決まる。", "Its pattern is inherited from its parents.", "花纹的不同由从亲代传给子代的遗传机制决定。")),
    sp("genji-firefly", L("ゲンジボタル", "Genji firefly", "源氏萤"), "Luciola cruciata", GBeetle, L("ホタル科", "Lampyridae", "萤科"), 1.5, RIVER, &[S_TAKAO, S_RANZAN, S_YORO, S_NAGATORO, S_SAYAMA],
        Flutter, Art::Firefly, [0.14, 0.12, 0.12], [0.95, 0.40, 0.30], 2.5, 1.0, 0.3, (6, 6), DUSK | NIGHT,
        L("初夏の夜、きれいな川辺で光るホタル。光の点めつで仲間と合図し合う。", "On early-summer nights it glows along clean streams, signalling to others with flashes of light.", "初夏夜晚在清澈河边发光的萤火虫，用光的闪烁和同伴传递信号。"),
        L("幼虫は水の中でカワニナという貝を食べて育つ。", "Its larvae live underwater and feed on freshwater snails.", "幼虫在水中以川蜷螺为食长大。")),
    sp("heike-firefly", L("ヘイケボタル", "Heike firefly", "平家萤"), "Aquatica lateralis", GBeetle, L("ホタル科", "Lampyridae", "萤科"), 0.9, PADDY | POND, &[S_SAYAMA, S_KASUMIGAURA, S_WATARASE, S_NARITA],
        Flutter, Art::Firefly, [0.14, 0.12, 0.12], [0.95, 0.55, 0.40], 2.5, 1.0, 0.3, (6, 8), NIGHT,
        L("ゲンジボタルより小さいホタル。田んぼや湿地で、弱い光をちかちかさせる。", "A smaller firefly that flickers faintly over rice paddies and marshes.", "比源氏萤小的萤火虫，在稻田和湿地里闪着微弱的光。"),
        L("農薬が減り、水の豊かな田んぼがあるところで見られる。", "It survives where paddies stay wet and pesticide use is low.", "能在农药少、水源充足的稻田看到。")),
    sp("chinese-diving-beetle", L("ゲンゴロウ", "Japanese diving beetle", "黄缘龙虱"), "Cybister chinensis", GBeetle, L("ゲンゴロウ科", "Dytiscidae", "龙虱科"), 3.8, POND | PADDY, &[S_SAYAMA, S_KASUMIGAURA, S_WATARASE],
        Swim, Art::DivingBeetle, [0.14, 0.18, 0.10], [0.80, 0.70, 0.30], 0.8, 1.2, 0.6, (4, 10), ALL_DAY,
        L("つやのある緑がかった黒い体に、黄色いふちどりがある水の甲虫。後ろ足をオールのようにこいで泳ぐ。", "A glossy greenish-black water beetle with yellow edges that rows with oar-like hind legs.", "有光泽的墨绿色身体镶着黄边的水生甲虫，用船桨般的后腿划水游泳。"),
        L("おしりの先に空気をためて、水の中で息をする。今はとても数が減っている。", "It carries an air bubble at its tail to breathe underwater. It has become very rare.", "在尾端储存空气，在水中呼吸。现在数量已大大减少。")),
    sp("rosalia-batesi", L("ルリボシカミキリ", "Rosalia batesi", "蓝丽天牛"), "Rosalia batesi", GBeetle, L("カミキリムシ科", "Cerambycidae", "天牛科"), 2.5, MOUNTAIN, &[S_TAKAO, S_TSUKUBA, S_NIKKO, S_NAGATORO],
        Perch, Art::Longhorn, [0.45, 0.70, 0.90], [0.06, 0.06, 0.08], 1.0, 0.9, 0.08, (7, 8), DAY,
        L("空色の体に黒い点がならぶ、とても美しいカミキリムシ。山の林の伐採木に来る。", "A stunning sky-blue longhorn with black spots, drawn to felled timber in mountain woods.", "天蓝色身体上排列着黑点的美丽天牛，会来到山林的伐木上。"),
        L("この青い色は、死ぬと赤茶色に変わってしまう。", "Its blue fades to reddish-brown after death.", "这种蓝色在死后会变成红褐色。")),
    sp("white-spotted-longicorn", L("ゴマダラカミキリ", "White-spotted longicorn", "星天牛"), "Anoplophora malasiaca", GBeetle, L("カミキリムシ科", "Cerambycidae", "天牛科"), 3.2, PARK | FOREST, &[],
        Perch, Art::Longhorn, [0.08, 0.08, 0.10], [0.95, 0.95, 0.95], 1.5, 0.9, 0.08, (6, 8), DAY | DUSK,
        L("黒い体に白い点が散らばるカミキリムシ。長い触角が白黒のしま模様。公園や庭の木にいる。", "A black longhorn sprinkled with white spots and long banded antennae, on park and garden trees.", "黑色身体上散布白点的天牛，长触角呈黑白条纹，在公园和庭院的树上。"),
        L("つかむと「キイキイ」と音を出す。幼虫は木の幹の中を食べ進む。", "It squeaks when held. Its larvae tunnel through tree trunks.", "被抓住时会发出“吱吱”声。幼虫在树干中钻食。")),
    sp("japanese-jewel-beetle", L("ヤマトタマムシ", "Japanese jewel beetle", "日本吉丁虫"), "Chrysochroa fulgidissima", GBeetle, L("タマムシ科", "Buprestidae", "吉丁虫科"), 3.5, FOREST | PARK, &[S_MEIJI, S_KAMAKURA, S_NARITA, S_SAYAMA, S_YORO],
        Perch, Art::Jewel, [0.20, 0.70, 0.35], [0.80, 0.30, 0.60], 0.8, 1.0, 0.08, (7, 8), DAY,
        L("緑色に赤むらさきのすじが入り、宝石のようにかがやく甲虫。神社の大きなエノキのまわりを飛ぶ。", "A gem-like beetle of shining green with purple-red stripes, flying around tall hackberries at shrines.", "绿色中带有紫红色条纹、像宝石一样闪耀的甲虫，在神社的大朴树周围飞行。"),
        L("このはねは法隆寺の「玉虫厨子」のかざりに使われた。", "Its wing cases decorated the famous Tamamushi Shrine at Horyuji temple.", "它的鞘翅曾被用来装饰法隆寺的“玉虫厨子”。")),
    sp("tiger-beetle", L("ナミハンミョウ", "Tiger beetle", "金斑虎甲"), "Cicindela japonica", GBeetle, L("オサムシ科", "Carabidae", "步甲科"), 2.0, FOREST | MOUNTAIN, &[],
        Hop, Art::TigerBeetle, [0.20, 0.45, 0.35], [0.90, 0.30, 0.20], 1.2, 1.4, 0.4, (4, 9), DAY,
        L("赤・緑・白の派手な模様の甲虫。山道で人の前を少し飛んではとまるので「道教え」とよばれる。", "A gaudy red, green and white beetle. It hops ahead of hikers along paths, earning the name 'road guide'.", "红绿白相间的鲜艳甲虫。在山路上总是飞到人前面一点再停下，所以叫“引路虫”。"),
        L("大きなあごで小さな虫をつかまえて食べる肉食の虫。", "A predator that seizes small insects with its large jaws.", "用大颚捕食小虫的肉食性昆虫。")),
    sp("dor-beetle", L("オオセンチコガネ", "Dor beetle", "大粪金龟"), "Phelotrupes auratus", GBeetle, L("センチコガネ科", "Geotrupidae", "粪金龟科"), 2.2, MOUNTAIN | FOREST, &[S_NIKKO, S_TAKAO, S_TSUKUBA, S_NAGATORO],
        Crawl, Art::Scarab(2), [0.65, 0.25, 0.50], [0.30, 0.60, 0.40], 1.0, 0.8, 0.15, (5, 10), DAWN | DAY | DUSK,
        L("赤むらさきや緑にかがやく丸い甲虫。山道で動物のふんや落ちたキノコを食べる。", "A round beetle gleaming purple-red or green that eats dung and fallen mushrooms on mountain trails.", "闪着紫红或绿色光泽的圆形甲虫，在山路上吃动物粪便和掉落的蘑菇。"),
        L("ふんを片づける「森のそうじ屋」。地域によって色がちがう。", "A 'forest cleaner' that clears dung. Its colour varies from region to region.", "清理粪便的“森林清洁工”，颜色因地区而异。")),
    // ---------------------------------------------------------------- セミ・カメムシ
    sp("large-brown-cicada", L("アブラゼミ", "Large brown cicada", "油蝉"), "Graptopsaltria nigrofuscata", GBug, CICADIDAE, 6.0, PARK | FOREST, &[],
        Perch, Art::Cicada(1), [0.20, 0.18, 0.14], [0.50, 0.32, 0.20], 3.0, 0.9, 0.10, (7, 9), DAWN | DAY | DUSK,
        L("茶色いはねのセミ。「ジージリジリ」と油であげものをするような声で鳴く。町の公園にいちばん多い。", "A cicada with opaque brown wings whose sizzling song sounds like deep-frying. The commonest cicada in city parks.", "翅膀褐色的蝉，叫声“吱—吱哩吱哩”像油炸东西的声音，城市公园里最多。"),
        L("はねが不透明なセミは、世界でもめずらしい。", "Opaque-winged cicadas like this are rare worldwide.", "翅膀不透明的蝉在世界上也很少见。")),
    sp("robust-cicada", L("ミンミンゼミ", "Robust cicada", "鸣鸣蝉"), "Hyalessa maculaticollis", GBug, CICADIDAE, 6.0, PARK | FOREST, &[],
        Perch, Art::Cicada(0), [0.22, 0.40, 0.32], [0.85, 0.92, 0.95], 2.0, 0.9, 0.10, (7, 9), DAY,
        L("緑と黒の体に、すき通ったはねのセミ。「ミーンミンミン」と鳴く。", "A green-and-black cicada with clear wings that sings 'meen-min-min'.", "绿黑色身体、翅膀透明的蝉，叫声“咪—咪咪”。"),
        L("昔は山のセミだったが、今は東京の都心でもふつうに鳴いている。", "Once a mountain species, it now sings commonly in central Tokyo.", "以前是山里的蝉，现在在东京市中心也很常见。")),
    sp("tsukutsukuboshi", L("ツクツクボウシ", "Walker's cicada", "寒蝉"), "Meimuna opalifera", GBug, CICADIDAE, 4.5, PARK | FOREST, &[],
        Perch, Art::Cicada(0), [0.30, 0.38, 0.24], [0.85, 0.92, 0.95], 2.0, 1.0, 0.10, (8, 9), DAY,
        L("夏の終わりに「オーシーツクツク」と鳴く、ほっそりしたセミ。この声が聞こえると夏休みも終わりが近い。", "A slender cicada whose 'oh-shii tsuku-tsuku' song signals the end of summer vacation.", "夏末鸣叫“哦—西促促”的纤细蝉，听到它的声音，暑假就快结束了。"),
        L("鳴き声のさいごは「ジー」と終わる。", "Its song finishes with a long buzz.", "叫声最后以“吱—”结束。")),
    sp("kaempfer-cicada", L("ニイニイゼミ", "Kaempfer cicada", "蟪蛄"), "Platypleura kaempferi", GBug, CICADIDAE, 3.5, PARK | FOREST, &[],
        Perch, Art::Cicada(2), [0.42, 0.38, 0.30], [0.60, 0.56, 0.42], 2.0, 0.8, 0.10, (6, 8), DAWN | DAY,
        L("木の皮そっくりのまだら模様の小さなセミ。梅雨明け前から「チー」と鳴きはじめる。", "A small cicada mottled like tree bark that starts its long 'cheee' before the rainy season ends.", "长着像树皮一样斑驳花纹的小蝉，梅雨结束前就开始“吱—”地叫。"),
        L("ぬけがらは小さくて丸く、泥がついていることが多い。", "Its shed skins are small, round and often muddy.", "蝉蜕小而圆，常沾着泥。")),
    sp("higurashi", L("ヒグラシ", "Evening cicada", "暮蝉"), "Tanna japonensis", GBug, CICADIDAE, 5.0, FOREST | MOUNTAIN, &[],
        Perch, Art::Cicada(0), [0.55, 0.38, 0.20], [0.85, 0.92, 0.95], 1.6, 1.0, 0.10, (6, 9), DAWN | DUSK,
        L("夕方や明け方に「カナカナカナ」とすずしげに鳴くセミ。うす暗い林にいる。", "A cicada that sings a cool 'kana-kana-kana' at dusk and dawn in shady woods.", "在傍晚和黎明发出清凉“卡那卡那”叫声的蝉，生活在昏暗的树林里。"),
        L("名前は「日を暮れさせる」という意味。", "Its name means 'bringing the day to an end'.", "名字的意思是“让太阳落山”。")),
    sp("pond-skater", L("アメンボ", "Pond skater", "水黾"), "Aquarius paludum", GBug, L("アメンボ科", "Gerridae", "黾蝽科"), 1.4, POND | PADDY | RIVER, &[],
        Skate, Art::Strider, [0.18, 0.16, 0.14], [0.40, 0.36, 0.30], 4.0, 0.9, 0.6, (4, 10), DAWN | DAY | DUSK,
        L("長い足で水面に立ち、すいすいすべる虫。水に落ちた虫の体液を吸う。", "It stands on the water on long legs and skims about, sucking the juices of insects that fall in.", "用长腿站在水面上滑行的虫子，吸食落水昆虫的体液。"),
        L("足の先の細かい毛が水をはじくので、しずまない。体からアメのようなにおいがする。", "Water-repellent hairs on its feet keep it afloat. It smells faintly of candy.", "脚尖细毛能拒水，所以不会下沉。身上有糖果一样的气味。")),
    sp("water-stick-insect", L("ミズカマキリ", "Water stick insect", "中华螳蝎蝽"), "Ranatra chinensis", GBug, L("タイコウチ科", "Nepidae", "蝎蝽科"), 4.5, POND | PADDY, &[],
        Swim, Art::WaterBug(0), [0.50, 0.40, 0.26], [0.36, 0.28, 0.18], 1.5, 1.0, 0.25, (4, 10), ALL_DAY,
        L("カマキリのような前足をもつ、細長い水の虫。水草にまぎれて小魚やオタマジャクシを待ちぶせる。", "A long, thin water bug with mantis-like forelegs that hides among weeds to ambush small fish and tadpoles.", "有螳螂般前足的细长水生昆虫，混在水草中伏击小鱼和蝌蚪。"),
        L("おしりの長い管を水面に出して呼吸する。", "It breathes through a long snorkel at its tail.", "把尾部的长管伸出水面呼吸。")),
    sp("water-scorpion", L("タイコウチ", "Water scorpion", "日本蝎蝽"), "Laccotrephes japonensis", GBug, L("タイコウチ科", "Nepidae", "蝎蝽科"), 3.5, PADDY, &[S_SAYAMA, S_KASUMIGAURA, S_NARITA],
        Swim, Art::WaterBug(1), [0.36, 0.28, 0.20], [0.26, 0.20, 0.14], 1.2, 0.9, 0.18, (4, 10), ALL_DAY,
        L("平たい茶色の体の水の虫。田んぼの浅い水底をゆっくり歩き、前足で獲物をつかまえる。", "A flat brown water bug that creeps slowly over shallow paddy bottoms, seizing prey with its forelegs.", "身体扁平的褐色水生昆虫，在稻田浅水底慢慢行走，用前足捕捉猎物。"),
        L("前足を動かすようすが太鼓をたたくように見えるのが名前の由来。", "Its name comes from how its forelegs seem to beat a drum.", "前足的动作像敲鼓，名字由此而来。")),
    sp("giant-water-bug-appasus", L("コオイムシ", "Toe-biter", "日本负子蝽"), "Appasus japonicus", GBug, L("コオイムシ科", "Belostomatidae", "负子蝽科"), 2.0, PADDY | POND, &[S_SAYAMA, S_KASUMIGAURA, S_WATARASE],
        Swim, Art::WaterBug(2), [0.50, 0.42, 0.30], [0.80, 0.70, 0.40], 1.2, 0.9, 0.2, (4, 10), ALL_DAY,
        L("茶色の平たい水の虫。オスが背中にたまごを背負って守る。", "A flat brown water bug whose males carry and guard the eggs on their backs.", "褐色扁平的水生昆虫，雄虫把卵背在背上保护。"),
        L("「子を負う虫」が名前の由来。オスは背中のたまごに新しい空気を当てる。", "Its name means 'child-carrying bug'. The male airs the eggs on his back.", "名字意为“背孩子的虫”。雄虫会让背上的卵接触新鲜空气。")),
    sp("striped-shield-bug", L("アカスジカメムシ", "Striped shield bug", "赤条蝽"), "Graphosoma rubrolineatum", GBug, L("カメムシ科", "Pentatomidae", "蝽科"), 1.1, MEADOW | PARK, &[],
        Crawl, Art::StinkBug, [0.90, 0.20, 0.12], [0.06, 0.06, 0.06], 2.5, 0.5, 0.06, (5, 8), DAWN | DAY,
        L("赤と黒のたてじまが目立つカメムシ。セリやニンジンの仲間の白い花にとまる。", "A shield bug with bold red-and-black stripes that sits on the white flower heads of parsley-family plants.", "红黑竖条纹醒目的蝽，停在芹菜和胡萝卜类植物的白花上。"),
        L("はでな色は「まずいぞ」という鳥への合図と考えられている。", "Its loud colours are thought to warn birds that it tastes bad.", "鲜艳的颜色被认为是在警告鸟类“我很难吃”。")),
    // ---------------------------------------------------------------- バッタ・カマキリ
    sp("migratory-locust", L("トノサマバッタ", "Migratory locust", "飞蝗"), "Locusta migratoria", GHopper, ACRIDIDAE, 5.0, MEADOW | SHORE, &[],
        Hop, Art::Hopper(0), [0.50, 0.62, 0.30], [0.55, 0.45, 0.30], 2.0, 1.5, 0.5, (7, 11), DAWN | DAY | DUSK,
        L("大きくて力強いバッタ。河原の草はらで、近づくと「パタパタ」と遠くまで飛んで逃げる。", "A large, powerful grasshopper of grassy riverbanks that whirs far away when approached.", "又大又有力的蝗虫，在河滩草地上，一靠近就“啪嗒啪嗒”地飞到远处。"),
        L("数がふえすぎると体の色や形が変わり、大群で移動することがある。", "In crowded years it can change colour and shape and travel in great swarms.", "数量过多时体色和体形会改变，有时会成群迁移。")),
    sp("oriental-longheaded-grasshopper", L("ショウリョウバッタ", "Oriental long-headed grasshopper", "中华剑角蝗"), "Acrida cinerea", GHopper, ACRIDIDAE, 8.0, MEADOW | PADDY | PARK, &[],
        Hop, Art::Hopper(1), [0.45, 0.70, 0.32], [0.70, 0.80, 0.50], 2.5, 1.2, 0.4, (7, 11), DAWN | DAY | DUSK,
        L("とがった頭の細長いバッタ。メスは日本のバッタでいちばん大きい。オスは飛ぶとき「キチキチ」と音を出す。", "A slender grasshopper with a pointed head; females are Japan's largest grasshoppers. Males click as they fly.", "头部尖尖的细长蝗虫，雌虫是日本最大的蝗虫。雄虫飞行时发出“吱吱”声。"),
        L("後ろ足を持つと、体を上下にゆらす。お盆（精霊）のころに目立つのが名前の由来。", "Held by the hind legs, it bobs up and down. It is named after the Obon festival season.", "抓住后腿时身体会上下摆动。因在盂兰盆节前后常见而得名。")),
    sp("smaller-longheaded-grasshopper", L("オンブバッタ", "Smaller long-headed grasshopper", "短额负蝗"), "Atractomorpha lata", GHopper, L("オンブバッタ科", "Pyrgomorphidae", "锥头蝗科"), 3.0, PARK | MEADOW, &[],
        Hop, Art::Hopper(1), [0.40, 0.68, 0.30], [0.60, 0.76, 0.40], 3.0, 0.9, 0.3, (8, 11), DAWN | DAY | DUSK,
        L("小さなオスが大きなメスの背中に乗っていることが多いバッタ。花だんや庭の葉をかじる。", "A small grasshopper whose tiny male often rides on the larger female's back; it nibbles garden leaves.", "小雄虫常趴在大雌虫背上的蝗虫，啃食花坛和庭院的叶子。"),
        L("「おんぶ」しているのは親子ではなく、オスとメス。", "The 'piggyback' pair is not parent and child but male and female.", "“背着”的不是亲子，而是雄虫和雌虫。")),
    sp("emma-field-cricket", L("エンマコオロギ", "Emma field cricket", "黄脸油葫芦"), "Teleogryllus emma", GHopper, L("コオロギ科", "Gryllidae", "蟋蟀科"), 3.0, MEADOW | PARK, &[],
        Crawl, Art::Hopper(3), [0.18, 0.14, 0.10], [0.40, 0.30, 0.18], 2.5, 0.9, 0.25, (8, 11), DUSK | NIGHT,
        L("日本でいちばん大きなコオロギ。秋の夜、「コロコロリー」ときれいな声で鳴く。", "Japan's largest cricket, singing a lovely 'koro-koro-ree' on autumn nights.", "日本最大的蟋蟀，秋夜发出“咕噜咕噜哩”的悦耳鸣声。"),
        L("顔のもようが、えんま大王のこわい顔に見えるのが名前の由来。", "Its face pattern looks like the scowl of Enma, king of the underworld.", "脸上的花纹像阎罗王的凶脸，因此得名。")),
    sp("japanese-katydid", L("ヒガシキリギリス", "Japanese katydid", "日本螽斯"), "Gampsocleis mikado", GHopper, L("キリギリス科", "Tettigoniidae", "螽斯科"), 4.0, MEADOW, &[],
        Hop, Art::Hopper(2), [0.48, 0.66, 0.30], [0.55, 0.45, 0.28], 1.5, 1.2, 0.3, (7, 9), DAWN | DAY,
        L("長い触角をもつ緑のキリギリス。夏の昼間、草むらで「ギーッチョン」と鳴く。", "A green katydid with long antennae that sings 'gii-chon' in summer meadows by day.", "有长触角的绿色螽斯，夏天白天在草丛中“吱—哟”地叫。"),
        L("小さな虫も食べる雑食性。近づくと鳴きやんで草のかげにかくれる。", "An omnivore that also eats small insects. It falls silent and hides when approached.", "是也吃小虫的杂食性昆虫。一靠近就停止鸣叫躲进草后。")),
    sp("bell-cricket", L("スズムシ", "Bell cricket", "铃虫"), "Meloimorpha japonica", GHopper, L("マツムシ科", "Oecanthidae", "树蟋科"), 1.8, FOREST | MEADOW, &[],
        Crawl, Art::Hopper(3), [0.14, 0.12, 0.08], [0.85, 0.80, 0.60], 1.5, 0.7, 0.2, (8, 10), DUSK | NIGHT,
        L("スイカのたねのような形の黒い虫。秋の夜に「リーンリーン」と鈴のような声で鳴く。", "A black, watermelon-seed-shaped cricket whose 'reen-reen' rings like a bell on autumn nights.", "形状像西瓜子的黑色昆虫，秋夜发出铃铛般“铃—铃—”的鸣声。"),
        L("江戸時代から、声を楽しむためにかごで飼われてきた。", "People have kept them in cages to enjoy their song since the Edo period.", "从江户时代起，人们就用笼子饲养它来欣赏鸣声。")),
    sp("chinese-mantis", L("オオカマキリ", "Chinese mantis", "中华大刀螳"), "Tenodera sinensis", GHopper, MANTIDAE, 8.5, MEADOW | PARK, &[],
        Perch, Art::Mantis(0), [0.45, 0.66, 0.30], [0.56, 0.48, 0.30], 1.5, 0.8, 0.08, (8, 11), DAWN | DAY | DUSK,
        L("日本でいちばん大きなカマキリ。草の上で、かまのような前足をかまえて虫を待ちぶせる。", "Japan's largest mantis, waiting in the grass with scythe-like forelegs raised to ambush insects.", "日本最大的螳螂，在草上举着镰刀般的前足伏击昆虫。"),
        L("秋に、あわでできたたまごのかたまり（卵のう）を枝に産む。", "In autumn it lays a foamy egg case on twigs.", "秋天在树枝上产下泡沫状的卵块（卵鞘）。")),
    sp("giant-asian-mantis", L("ハラビロカマキリ", "Giant Asian mantis", "广腹螳螂"), "Hierodula patellifera", GHopper, MANTIDAE, 6.0, PARK | FOREST, &[],
        Perch, Art::Mantis(1), [0.40, 0.70, 0.30], [0.95, 0.95, 0.80], 1.3, 0.8, 0.08, (8, 11), DAWN | DAY | DUSK,
        L("おなかのはばが広い、ずんぐりしたカマキリ。前ばねに白い点がある。木の上で暮らす。", "A stout mantis with a broad abdomen and a white spot on each forewing, living up in trees.", "腹部宽阔、身形粗壮的螳螂，前翅有白点，生活在树上。"),
        L("近年、よく似た外来のムネアカハラビロカマキリがふえている。", "A similar introduced species has recently been spreading.", "近年来，一种相似的外来螳螂正在增多。")),
    sp("small-mantis", L("コカマキリ", "Small mantis", "小螳螂"), "Statilia maculata", GHopper, MANTIDAE, 5.5, MEADOW | FOREST, &[],
        Crawl, Art::Mantis(0), [0.50, 0.40, 0.28], [0.30, 0.22, 0.16], 1.5, 0.8, 0.12, (8, 11), DAWN | DAY | DUSK,
        L("茶色い小型のカマキリ。落ち葉の上や地面を歩いて虫をさがす。", "A small brown mantis that walks over leaf litter and the ground searching for prey.", "褐色的小型螳螂，在落叶和地面上行走寻找昆虫。"),
        L("かまの内側に黒と白の模様があり、敵をおどかすときに見せる。", "The inside of its forelegs bears a black-and-white pattern flashed to scare enemies.", "镰足内侧有黑白花纹，在威吓敌人时展示。")),
    sp("stick-insect", L("ナナフシモドキ", "Japanese stick insect", "日本竹节虫"), "Ramulus mikado", GHopper, L("ナナフシ科", "Phasmatidae", "竹节虫科"), 9.0, FOREST, &[],
        Perch, Art::Stick, [0.40, 0.52, 0.28], [0.50, 0.42, 0.28], 0.9, 0.5, 0.04, (6, 11), ALL_DAY,
        L("木の枝にそっくりな細長い虫。サクラやクヌギの葉を食べ、じっとしているとほとんど見つからない。", "A long, thin insect that looks exactly like a twig. It feeds on cherry and oak leaves and is nearly invisible when still.", "长得和树枝一模一样的细长昆虫，吃樱树和麻栎叶，静止时几乎找不到。"),
        L("オスがとても少なく、メスだけでたまごを産んでふえることができる。", "Males are very rare; females can lay fertile eggs on their own.", "雄虫非常少，雌虫可以单独产卵繁殖。")),
    // ---------------------------------------------------------------- ハチ・アリなど
    sp("western-honey-bee", L("セイヨウミツバチ", "Western honey bee", "西方蜜蜂"), "Apis mellifera", GBee, APIDAE, 1.3, PARK | MEADOW, &[],
        Fly, Art::Bee(0), [0.85, 0.62, 0.22], [0.20, 0.14, 0.08], 3.5, 0.8, 0.6, (3, 11), DAY,
        L("はちみつをとるために人が飼っているミツバチ。花から花へ、蜜と花粉を集めて回る。", "The honey bee kept by beekeepers, visiting flower after flower for nectar and pollen.", "人类为采蜜而饲养的蜜蜂，在花间来回采集花蜜和花粉。"),
        L("8の字ダンスで、仲間に花の場所を教える。", "It tells nestmates where flowers are with a figure-of-eight dance.", "用8字舞告诉同伴花的位置。")),
    sp("japanese-honey-bee", L("ニホンミツバチ", "Japanese honey bee", "日本蜜蜂"), "Apis cerana japonica", GBee, APIDAE, 1.2, FOREST | PARK, &[],
        Fly, Art::Bee(0), [0.40, 0.32, 0.18], [0.14, 0.12, 0.08], 2.5, 0.8, 0.6, (3, 11), DAWN | DAY,
        L("昔から日本の森にすむ野生のミツバチ。木のうろなどに巣をつくる。", "Japan's native wild honey bee, nesting in tree hollows.", "自古生活在日本森林的野生蜜蜂，在树洞等处筑巢。"),
        L("オオスズメバチにおそわれると、みんなで取りかこんで熱で倒す。", "When a giant hornet attacks, the bees surround it and cook it with their body heat.", "遭到大胡蜂袭击时，大家一起包围它，用体温把它热死。")),
    sp("asian-giant-hornet", L("オオスズメバチ", "Asian giant hornet", "金环胡蜂"), "Vespa mandarinia", GBee, VESPIDAE, 4.0, FOREST | MOUNTAIN, &[],
        Fly, Art::Bee(1), [0.95, 0.65, 0.12], [0.18, 0.12, 0.06], 0.8, 2.6, 1.0, (4, 10), DAY,
        L("世界最大のスズメバチ。オレンジ色の大きな頭が目印。樹液にも来る。とても強い毒をもつので、決して近づかないこと。", "The world's largest hornet, marked by a big orange head. It visits sap too. Its venom is very strong — never go near it.", "世界上最大的胡蜂，以橙色大头为标志，也会来吃树液。毒性非常强，绝对不要靠近。"),
        L("黒いものをおそう習性がある。山では白っぽい服と帽子が安全。", "It tends to attack dark things, so wear light-coloured clothes and a hat in the hills.", "有攻击黑色物体的习性，上山时穿浅色衣服戴帽子比较安全。")),
    sp("yellow-hornet", L("キイロスズメバチ", "Yellow hornet", "黄胡蜂"), "Vespa simillima xanthoptera", GBee, VESPIDAE, 2.5, PARK | FOREST, &[],
        Fly, Art::Bee(1), [0.96, 0.78, 0.20], [0.30, 0.18, 0.08], 1.2, 2.4, 1.0, (4, 10), DAY,
        L("黄色っぽい体のスズメバチ。家の軒下や木に大きな巣をつくり、町でも見られる。", "A yellowish hornet that builds big nests under eaves and in trees, even in towns.", "身体偏黄的胡蜂，在屋檐下和树上筑大巢，城市里也能看到。"),
        L("巣に近づくとおそってくる。見つけたら静かにはなれよう。", "It attacks anyone near its nest. If you see one, move away quietly.", "靠近巢穴就会发动攻击，发现了要安静离开。")),
    sp("carpenter-bee", L("キムネクマバチ", "Yellow-chested carpenter bee", "黄胸木蜂"), "Xylocopa appendiculata", GBee, APIDAE, 2.3, PARK | MEADOW, &[],
        Fly, Art::Bee(2), [0.10, 0.10, 0.10], [0.95, 0.85, 0.30], 1.8, 1.4, 0.8, (4, 6), DAY,
        L("黒くて丸い大きなハチ。胸に黄色い毛がふさふさ。春、藤の花のまわりを「ブーン」と飛ぶ。", "A big round black bee with a fluffy yellow chest that buzzes around wisteria in spring.", "黑色圆滚滚的大蜂，胸部长着蓬松的黄毛，春天在紫藤花周围“嗡嗡”地飞。"),
        L("見た目はこわいが、とてもおとなしい。オスには針がない。", "It looks scary but is very gentle; males have no sting.", "看起来很吓人，其实很温和，雄蜂没有螫针。")),
    sp("japanese-carpenter-ant", L("クロオオアリ", "Japanese carpenter ant", "日本弓背蚁"), "Camponotus japonicus", GBee, L("アリ科", "Formicidae", "蚁科"), 1.0, PARK | MEADOW, &[],
        Crawl, Art::Ant, [0.08, 0.08, 0.08], [0.20, 0.18, 0.16], 5.0, 0.4, 0.25, (4, 10), DAWN | DAY | DUSK,
        L("日本でいちばん大きなアリの一つ。公園の地面に巣をつくり、行列をつくって食べものを運ぶ。", "One of Japan's biggest ants, nesting in park soil and carrying food in long lines.", "日本最大的蚂蚁之一，在公园地面筑巢，排成队伍搬运食物。"),
        L("初夏の夕方、はねのある女王アリとオスがいっせいに飛び立つ。", "On early-summer evenings, winged queens and males take off all at once.", "初夏的傍晚，有翅的蚁后和雄蚁会一齐飞出。")),
    sp("maritime-earwig", L("ハマベハサミムシ", "Maritime earwig", "海滨蠼螋"), "Anisolabis maritima", GBee, L("ハサミムシ科", "Anisolabididae", "肥螋科"), 2.5, SHORE, &[],
        Crawl, Art::Earwig, [0.20, 0.14, 0.10], [0.70, 0.55, 0.30], 2.5, 0.8, 0.25, (3, 11), ALL_DAY,
        L("海辺の砂浜や石の下にすむハサミムシ。おしりに大きなハサミがある。", "An earwig of beaches, hiding under stones, with large pincers at its tail.", "生活在海边沙滩和石头下的蠼螋，尾部有一对大钳子。"),
        L("母親はたまごのそばをはなれず、カビないようになめて守る。", "The mother stays with her eggs and licks them clean to keep off mould.", "母亲寸步不离地守护着卵，舔舐卵防止发霉。")),
];

/// 図鑑の項目の画像の名前
pub fn entry_slug(e: usize) -> &'static str {
    SPECIES[e].slug
}

pub fn n_species() -> usize {
    SPECIES.len()
}

/// 図鑑の項目の数
pub fn n_entries() -> usize {
    SPECIES.len()
}

/// その月に、そのスポットのその場面にいるか
/// その月に、そのスポットのその場面にいるか。
/// 活動していない時間帯でも休んでいる場所に、成虫の季節でなければ冬越しの姿で越冬場所にいる
pub fn lives_here(sp: &Species, spot: usize, hab: Habitat, month: u8) -> bool {
    lives_at(sp, spot, hab) && (sp.in_season(month) || sp.winter().map_or(false, |(_, site)| site.fits(hab)))
}

/// 季節を問わず、そのスポットのその場面にすむか
pub fn lives_at(sp: &Species, spot: usize, hab: Habitat) -> bool {
    sp.hab & hab.bit() != 0 && SPOTS[spot].habs.contains(&hab) && (sp.spots.is_empty() || sp.spots.contains(&spot))
}

/// 会える場所（スポット, 場面）の一覧
pub fn places_of(i: usize) -> Vec<(usize, Habitat)> {
    let sp = &SPECIES[i];
    let mut out = Vec::new();
    for s in 0..SPOTS.len() {
        for &h in SPOTS[s].habs {
            if lives_at(sp, s, h) {
                out.push((s, h));
            }
        }
    }
    out
}

/// その虫に会いに行くスポット・場面・月。いまの月に見られればそのまま、見られなければいちばん近い月へ。
/// 場所は near（経緯度）からいちばん近いところ
pub fn where_is(i: usize, near: (f32, f32), month: u8) -> (usize, Habitat, u8) {
    let sp = &SPECIES[i];
    let somewhere = |m: u8| places_of(i).into_iter().any(|(s, h)| lives_here(sp, s, h, m));
    // 冬越しの姿でもいれば月はそのまま。どこにもいなければ（南から来る虫）、いちばん近い成虫の月へ
    let month = (0..12u8)
        .flat_map(|d| [month as i32 + d as i32, month as i32 - d as i32])
        .map(|m| (m - 1).rem_euclid(12) as u8 + 1)
        .find(|&m| somewhere(m))
        .unwrap_or(month);
    let (s, h) = places_of(i)
        .into_iter()
        .filter(|&(s, h)| lives_here(sp, s, h, month))
        .min_by(|a, b| {
            let da = km_dist(near, (SPOTS[a.0].lon, SPOTS[a.0].lat));
            let db = km_dist(near, (SPOTS[b.0].lon, SPOTS[b.0].lat));
            da.partial_cmp(&db).unwrap()
        })
        .unwrap_or((0, Habitat::Forest));
    (s, h, month)
}

/// 図鑑の項目のタブ
pub fn entry_group(e: usize) -> Group {
    SPECIES[e].group
}

/// すみかの説明
pub fn hab_text(sp: &Species) -> String {
    HABITATS.iter().filter(|h| sp.hab & h.bit() != 0).map(|h| h.name()).collect::<Vec<_>>().join(crate::lang::sep())
}

/// 会えるスポットの名前
pub fn place_text(i: usize) -> String {
    let mut v: Vec<usize> = places_of(i).into_iter().map(|p| p.0).collect();
    v.dedup();
    if v.len() == SPOTS.len() {
        return t!("関東のあちこち", "All over Kanto", "关东各地").to_string();
    }
    v.iter().map(|&s| SPOTS[s].name.get()).collect::<Vec<_>>().join(crate::lang::sep())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_species_can_be_found() {
        for (i, sp) in SPECIES.iter().enumerate() {
            assert!(!places_of(i).is_empty(), "{} has no place", sp.slug);
            assert!(sp.len_cm > 0.0 && sp.rarity > 0.0);
        }
    }

    #[test]
    fn every_month_has_insects_and_seasons_wrap() {
        for m in 1..=12u8 {
            assert!(SPECIES.iter().filter(|s| s.in_season(m)).count() >= 3, "month {}", m);
        }
        let moth = SPECIES.iter().find(|s| s.slug == "winter-moth").unwrap();
        assert!(moth.in_season(12) && moth.in_season(1) && !moth.in_season(7));
        let i = SPECIES.iter().position(|s| s.slug == "japanese-rhinoceros-beetle").unwrap();
        // 12月のカブトムシは、幼虫で土の中にいる
        let (s12, h12, m12) = where_is(i, (139.7, 35.7), 12);
        assert_eq!(m12, 12);
        assert!(!h12.is_water() && lives_here(&SPECIES[i], s12, h12, 12));
        assert_eq!(SPECIES[i].winter(), Some((Stage::Grub, Site::Soil)));
        // ウスバキトンボは関東で冬を越せないので、成虫の月へ
        let g = SPECIES.iter().position(|s| s.slug == "wandering-glider").unwrap();
        assert!(SPECIES[g].in_season(where_is(g, (139.7, 35.7), 1).2));
        // 冬越しの場所が、どの虫にも少なくとも一つの場面にある
        for (k, sp) in SPECIES.iter().enumerate() {
            if sp.winter().is_some() {
                assert!(places_of(k).iter().any(|&(s, h)| lives_here(sp, s, h, 1) || sp.in_season(1)), "{} has no winter site", sp.slug);
            }
        }
        assert_eq!(where_is(i, (139.7, 35.7), 7).2, 7);
        for t in 0..4u8 {
            assert!(SPECIES.iter().filter(|s| s.active_at(t) && s.in_season(7)).count() >= 8, "time {}", t);
        }
    }

    #[test]
    fn slugs_are_unique() {
        for (i, a) in SPECIES.iter().enumerate() {
            for b in &SPECIES[i + 1..] {
                assert_ne!(a.slug, b.slug);
            }
        }
    }

    #[test]
    fn every_group_has_entries() {
        for g in GROUPS {
            assert!(SPECIES.iter().filter(|s| s.group == g).count() >= 4, "{:?}", g);
        }
    }

    #[test]
    fn every_spot_scene_has_insects() {
        for (s, spot) in SPOTS.iter().enumerate() {
            for &h in spot.habs {
                let n = SPECIES.iter().filter(|sp| lives_here(sp, s, h, 7)).count();
                assert!(n >= 3, "{} {:?} has only {} in July", spot.name.0, h, n);
            }
        }
    }
}
