//! 学校の宿題（ミッション）：トキ先生から出される課題。撮影して図鑑に収録すると合格
use crate::lang::L;
use crate::life::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Target {
    /// 成虫
    Sp(usize),
    /// 冬越しの姿（たまご・幼虫・さなぎ）
    Stage(usize),
}

impl Target {
    /// 図鑑の項目番号
    pub fn entry(self) -> usize {
        match self {
            Target::Sp(i) | Target::Stage(i) => i,
        }
    }
    pub fn name(self) -> String {
        match self {
            Target::Sp(i) => SPECIES[i].name.get().to_string(),
            Target::Stage(i) => stage_name(i),
        }
    }
}

pub struct Mission {
    pub title: L,
    /// 先生からの課題の文
    pub brief: L,
    /// 合格したときの先生のことば
    pub done: L,
    /// 撮って図鑑に収録する虫（slug。「slug@stage」は冬越しの姿）
    pub targets: &'static [&'static str],
}

/// slug から図鑑の番号
pub fn species_index(slug: &str) -> usize {
    SPECIES.iter().position(|s| s.slug == slug).unwrap_or_else(|| panic!("unknown species {}", slug))
}

impl Mission {
    /// (対象, 必要な収録の数 1)
    pub fn target_list(&self) -> Vec<(Target, u8)> {
        self.targets
            .iter()
            .map(|s| match s.strip_suffix("@stage") {
                Some(slug) => (Target::Stage(species_index(slug)), 1),
                None => (Target::Sp(species_index(s)), 1),
            })
            .collect()
    }
}

pub static MISSIONS: &[Mission] = &[
    Mission {
        title: L("はじめての観察", "First observations", "第一次观察"),
        brief: L(
            "まずはカプセルに慣れましょう。公園や草はらで、モンシロチョウとナナホシテントウ、アブラゼミを撮影してきてください。",
            "First, get used to the capsule. In a park or meadow, photograph a small white, a seven-spot ladybird and a large brown cicada.",
            "先熟悉观测舱吧。请在公园或草地拍摄菜粉蝶、七星瓢虫和油蝉。",
        ),
        done: L("よくできました！ 身近な場所にも、こんなにいろいろな虫がいるのです。", "Well done! Even familiar places are full of different insects.", "做得好！身边的地方也有这么多种昆虫。"),
        targets: &["cabbage-white", "seven-spot-ladybird", "large-brown-cicada"],
    },
    Mission {
        title: L("公園のセミの合唱", "The cicada chorus", "公园里的蝉之合唱"),
        brief: L(
            "夏の公園では、何種類ものセミが鳴いています。アブラゼミ・ミンミンゼミ・ツクツクボウシ・ニイニイゼミを見つけましょう。",
            "Several kinds of cicada sing in summer parks. Find the large brown, robust, Walker's and Kaempfer cicadas.",
            "夏天的公园里有好几种蝉在鸣叫。请找到油蝉、鸣鸣蝉、寒蝉和蟪蛄。",
        ),
        done: L("セミは何年も土の中ですごし、地上に出てからは数週間しか生きません。", "Cicadas spend years underground and live only a few weeks above it.", "蝉在地下度过好几年，钻出地面后只能活几周。"),
        targets: &["large-brown-cicada", "robust-cicada", "tsukutsukuboshi", "kaempfer-cicada"],
    },
    Mission {
        title: L("雑木林の王者", "Kings of the woods", "杂木林之王"),
        brief: L(
            "夏の雑木林の樹液には、甲虫が集まります。カブトムシ・ノコギリクワガタ・コクワガタ・カナブンを撮影しましょう。",
            "Beetles gather at tree sap in summer woods. Photograph a rhinoceros beetle, a saw stag beetle, a Dorcus rectus and a kanabun.",
            "夏天杂木林的树液处聚集着甲虫。请拍摄独角仙、锯锹甲、小锹甲和日本金龟。",
        ),
        done: L("クヌギやコナラの樹液は、たくさんの虫のレストランなのです。", "Oak sap is a restaurant for a crowd of insects.", "麻栎和枹栎的树液是许多昆虫的餐厅。"),
        targets: &["japanese-rhinoceros-beetle", "saw-stag-beetle", "dorcus-rectus", "kanabun"],
    },
    Mission {
        title: L("田んぼの生きもの", "Life in the paddies", "稻田里的生物"),
        brief: L(
            "田んぼは水の虫のすみかです。アメンボ・ミズカマキリ・シオカラトンボを観察しましょう。",
            "Rice paddies are home to water insects. Observe a pond skater, a water stick insect and a white-tailed skimmer.",
            "稻田是水生昆虫的家。请观察水黾、中华螳蝎蝽和白尾灰蜻。",
        ),
        done: L("田んぼは、人がつくった湿地。たくさんの虫を育てています。", "Paddies are man-made wetlands that raise a great many insects.", "稻田是人造的湿地，养育着许多昆虫。"),
        targets: &["pond-skater", "water-stick-insect", "common-skimmer"],
    },
    Mission {
        title: L("草はらのバッタ", "Meadow hoppers", "草地上的蝗虫"),
        brief: L(
            "草はらや河原には、いろいろなバッタがいます。トノサマバッタ・ショウリョウバッタ・オンブバッタ・エンマコオロギを撮りましょう。",
            "Meadows and riverbanks hold many grasshoppers. Shoot a migratory locust, an oriental long-headed grasshopper, a smaller long-headed grasshopper and an Emma field cricket.",
            "草地和河滩上有各种蝗虫。请拍摄飞蝗、中华剑角蝗、短额负蝗和黄脸油葫芦。",
        ),
        done: L("バッタの仲間は、後ろ足の力で体の何十倍も跳べます。", "Grasshoppers can leap dozens of times their body length with their hind legs.", "蝗虫类靠后腿的力量能跳出体长几十倍的距离。"),
        targets: &["migratory-locust", "oriental-longheaded-grasshopper", "smaller-longheaded-grasshopper", "emma-field-cricket"],
    },
    Mission {
        title: L("カマキリのかま", "The mantis's scythes", "螳螂的镰刀"),
        brief: L(
            "待ちぶせの名人、カマキリを探しましょう。オオカマキリ・ハラビロカマキリ・コカマキリです。",
            "Look for the masters of ambush: the Chinese mantis, the giant Asian mantis and the small mantis.",
            "去寻找伏击高手螳螂吧：中华大刀螳、广腹螳螂和小螳螂。",
        ),
        done: L("カマキリは、動くものだけを獲物だと見分けます。", "Mantises recognise prey only by its movement.", "螳螂只把会动的东西认作猎物。"),
        targets: &["chinese-mantis", "giant-asian-mantis", "small-mantis"],
    },
    Mission {
        title: L("アゲハのなかま", "Swallowtails", "凤蝶家族"),
        brief: L(
            "大きくて美しいアゲハチョウ。ナミアゲハ・キアゲハ・クロアゲハ・アオスジアゲハを撮影しましょう。",
            "Large, beautiful swallowtails: photograph the Asian swallowtail, Old World swallowtail, spangle and common bluebottle.",
            "大而美丽的凤蝶。请拍摄柑橘凤蝶、金凤蝶、蓝凤蝶和青凤蝶。",
        ),
        done: L("アゲハの幼虫は、種類ごとに食べる植物が決まっています。", "Each swallowtail's caterpillar feeds on its own particular plants.", "每种凤蝶的幼虫只吃特定的植物。"),
        targets: &["asian-swallowtail", "old-world-swallowtail", "spangle", "common-bluebottle"],
    },
    Mission {
        title: L("国蝶をさがせ", "Find the national butterfly", "寻找国蝶"),
        brief: L(
            "日本の国蝶オオムラサキと、同じエノキで育つゴマダラチョウ・アカボシゴマダラを観察しましょう。",
            "Observe Japan's national butterfly, the great purple emperor, and the hackberry-feeding Hestina persimilis and red ring skirt.",
            "请观察日本国蝶大紫蛱蝶，以及同样以朴树为食的黑脉蛱蝶和红星斑蛱蝶。",
        ),
        done: L("アカボシゴマダラのように、人が持ちこんだ虫がふえると、もとの虫がすみかを失うこともあります。", "When introduced insects like the red ring skirt spread, native ones can lose their homes.", "像红星斑蛱蝶这样人为带入的昆虫增多后，本地昆虫可能会失去栖息地。"),
        targets: &["great-purple-emperor", "hestina-persimilis", "red-ring-skirt"],
    },
    Mission {
        title: L("トンボの王さま", "Dragonfly royalty", "蜻蜓之王"),
        brief: L(
            "大きなヤンマと赤とんぼを撮りましょう。オニヤンマ・ギンヤンマ・アキアカネです。",
            "Shoot the big darners and the red dragonfly: the jumbo dragonfly, the lesser emperor and the autumn darter.",
            "请拍摄大型的蜓和红蜻蜓：无霸勾蜓、碧伟蜓和秋赤蜻。",
        ),
        done: L("トンボは4枚のはねを別々に動かして、空中で止まったり後ろへ飛んだりできます。", "Dragonflies move their four wings separately, so they can hover and even fly backwards.", "蜻蜓的四片翅膀能分别活动，所以能悬停甚至向后飞。"),
        targets: &["golden-ringed-dragonfly", "lesser-emperor", "autumn-darter"],
    },
    Mission {
        title: L("水辺の光", "Lights by the water", "水边的光"),
        brief: L(
            "初夏の水辺へ。ゲンジボタル・ヘイケボタル・ハグロトンボを観察しましょう。",
            "Head to the waterside in early summer. Observe the Genji and Heike fireflies and the black-winged damselfly.",
            "初夏去水边吧。请观察源氏萤、平家萤和黑色蟌。",
        ),
        done: L("ホタルが光るのは、きれいな水とエサの貝がいる場所だけです。", "Fireflies glow only where there is clean water and the snails they eat.", "萤火虫只在有干净水源和作为食物的螺类的地方发光。"),
        targets: &["genji-firefly", "heike-firefly", "black-winged-damselfly"],
    },
    Mission {
        title: L("山の虫", "Mountain insects", "山地昆虫"),
        brief: L(
            "涼しい山の森へ。ミヤマクワガタ・ルリボシカミキリ・アサギマダラ・オオセンチコガネを探しましょう。",
            "Head for cool mountain forests and find the Miyama stag beetle, Rosalia batesi, chestnut tiger and dor beetle.",
            "去凉爽的山林吧。请寻找深山锹甲、蓝丽天牛、大绢斑蝶和大粪金龟。",
        ),
        done: L("同じ関東でも、高い山には平地とちがう虫がくらしています。", "Even within Kanto, high mountains hold different insects from the lowlands.", "即使同在关东，高山上生活着与平原不同的昆虫。"),
        targets: &["miyama-stag-beetle", "rosalia-batesi", "chestnut-tiger", "dor-beetle"],
    },
    Mission {
        title: L("ハチに気をつけて", "Mind the wasps", "小心蜂类"),
        brief: L(
            "ハチは花の受粉を助ける大切な虫ですが、危ないものもいます。ズームを使って、遠くからセイヨウミツバチ・キムネクマバチ・オオスズメバチを撮りましょう。",
            "Bees are vital pollinators, but some are dangerous. Use your zoom to shoot a western honey bee, a carpenter bee and an Asian giant hornet from a distance.",
            "蜂是帮助花朵授粉的重要昆虫，但也有危险的。请用变焦从远处拍摄西方蜜蜂、黄胸木蜂和金环胡蜂。",
        ),
        done: L("スズメバチに会ったら、さわがず、ゆっくりはなれるのが一番です。", "If you meet a hornet, stay calm and slowly back away.", "遇到胡蜂时，不要慌张，慢慢离开是最好的办法。"),
        targets: &["western-honey-bee", "carpenter-bee", "asian-giant-hornet"],
    },
    Mission {
        title: L("池と沼のなかま", "Pond and marsh life", "池塘与沼泽的伙伴"),
        brief: L(
            "水草の多い池や沼へ。ゲンゴロウ・コオイムシ・チョウトンボを観察しましょう。",
            "Visit weedy ponds and marshes to observe the Japanese diving beetle, the toe-biter and the butterfly dragonfly.",
            "去水草多的池塘和沼泽吧。请观察黄缘龙虱、日本负子蝽和黑丽翅蜻。",
        ),
        done: L("どれも数が減っている虫です。水辺を守ることが、虫を守ることになります。", "These insects are all declining. Protecting wetlands protects them.", "这些昆虫的数量都在减少。保护水边就是保护昆虫。"),
        targets: &["chinese-diving-beetle", "giant-water-bug-appasus", "butterfly-dragonfly"],
    },
    Mission {
        title: L("かくれんぼ名人", "Masters of disguise", "捉迷藏高手"),
        brief: L(
            "まわりにとけこむ虫を見つけましょう。ナナフシモドキ・ルリタテハ・ニイニイゼミです。",
            "Find insects that blend into their surroundings: the stick insect, the blue admiral and the Kaempfer cicada.",
            "找出与周围融为一体的昆虫：日本竹节虫、琉璃蛱蝶和蟪蛄。",
        ),
        done: L("鳥に見つからないように、虫は枝や木の皮にそっくりな姿になりました。", "To hide from birds, these insects came to look just like twigs and bark.", "为了不被鸟发现，这些昆虫变得和树枝、树皮一模一样。"),
        targets: &["stick-insect", "blue-admiral", "kaempfer-cicada"],
    },
    Mission {
        title: L("宝石のような虫", "Living jewels", "宝石般的昆虫"),
        brief: L(
            "神社の森や山で、かがやく虫を探しましょう。ヤマトタマムシ・ナミハンミョウ・オオミズアオです。",
            "Look for shining insects in shrine forests and mountains: the Japanese jewel beetle, the tiger beetle and the Japanese luna moth.",
            "在神社的森林和山里寻找闪耀的昆虫：日本吉丁虫、金斑虎甲和绿尾大蚕蛾。",
        ),
        done: L("タマムシの色は色素ではなく、細かい構造が光を反射してできる「構造色」です。", "The jewel beetle's colour comes not from pigment but from tiny structures that reflect light.", "吉丁虫的颜色不是色素，而是细微结构反射光线形成的“结构色”。"),
        targets: &["japanese-jewel-beetle", "tiger-beetle", "japanese-luna-moth"],
    },
    Mission {
        title: L("春の訪れ", "Signs of spring", "春天来了"),
        brief: L(
            "春になると、冬をこした虫やさなぎからかえった虫が動き出します。モンシロチョウ・ベニシジミ・キムネクマバチを撮影しましょう。地図の上の月を春に合わせてね。",
            "In spring, insects that survived the winter come out. Photograph a small white, a small copper and a carpenter bee. Set the month on the map to spring.",
            "春天到了，越冬的昆虫开始活动。请拍摄菜粉蝶、红灰蝶和黄胸木蜂。记得把地图上的月份调到春天。",
        ),
        done: L("春の虫は、花がさきはじめるのに合わせて現れます。", "Spring insects appear just as the flowers begin to bloom.", "春天的昆虫会配合花开的时间出现。"),
        targets: &["cabbage-white", "small-copper", "carpenter-bee"],
    },
    Mission {
        title: L("冬の虫さがし", "Winter insect hunt", "寻找冬天的昆虫"),
        brief: L(
            "冬は虫が少ない季節ですが、いないわけではありません。冬に飛ぶシロオビフユシャクと、集まって冬をこすナミテントウ・ナナホシテントウを探しましょう。",
            "Winter has few insects, but not none. Find the Japanese winter moth that flies in winter, and the Asian and seven-spot lady beetles that gather to overwinter.",
            "冬天虫子很少，但并不是没有。请寻找在冬天飞行的白带冬尺蛾，以及聚在一起越冬的异色瓢虫和七星瓢虫。",
        ),
        done: L("虫たちは、たまご・幼虫・さなぎ・成虫など、種類ごとにちがう姿で冬をこします。", "Each kind of insect gets through winter in its own stage: egg, larva, pupa or adult.", "每种昆虫以不同的形态越冬：卵、幼虫、蛹或成虫。"),
        targets: &["winter-moth", "harlequin-ladybird", "seven-spot-ladybird"],
    },
    Mission {
        title: L("夜の観察会", "Night watch", "夜间观察会"),
        brief: L(
            "夜にだけ活動する虫もいます。時間帯を「夜」にして、カブトムシ・エンマコオロギ・オオミズアオを探しましょう。暗いのでライトを忘れずに。",
            "Some insects come out only at night. Set the time to night and look for a rhinoceros beetle, an Emma field cricket and a Japanese luna moth. Don't forget your light.",
            "有些昆虫只在夜里活动。把时间调到夜晚，寻找独角仙、黄脸油葫芦和绿尾大蚕蛾。天黑，别忘了开灯。",
        ),
        done: L("夜の虫は、目よりも触角やにおい、音を手がかりに仲間や食べものを探します。", "Night insects find mates and food with their antennae, smell and sound more than their eyes.", "夜行昆虫更多依靠触角、气味和声音而不是眼睛来寻找同伴和食物。"),
        targets: &["japanese-rhinoceros-beetle", "emma-field-cricket", "japanese-luna-moth"],
    },
    Mission {
        title: L("冬越しのすがた", "Winter forms", "越冬的样子"),
        brief: L(
            "成虫が見られない季節も、虫はたまご・幼虫・さなぎで冬を越しています。カブトムシの幼虫、オオカマキリのたまごのかたまり、モンシロチョウのさなぎを見つけましょう。土の中や枝、木の幹をよくさがしてね。",
            "Even when no adults are about, insects survive the winter as eggs, larvae or pupae. Find a rhinoceros beetle larva, a Chinese mantis egg case and a small white pupa. Search the soil, twigs and tree trunks.",
            "即使看不到成虫的季节，昆虫也以卵、幼虫或蛹的形态越冬。请找到独角仙的幼虫、中华大刀螳的卵鞘和菜粉蝶的蛹。仔细找找土里、树枝和树干。",
        ),
        done: L("同じ虫でも、時期によって姿もすむ場所もまったくちがいます。", "The same insect can look and live completely differently at different times of year.", "同一种昆虫在不同时期，样子和栖息地完全不同。"),
        targets: &["japanese-rhinoceros-beetle@stage", "chinese-mantis@stage", "cabbage-white@stage"],
    },
];

/// 冬越しの姿の名前（「カブトムシの幼虫」）
pub fn stage_name(i: usize) -> String {
    let sp = &SPECIES[i];
    match sp.winter() {
        Some((st, _)) => tf!("{}の{}", "{} {}", "{}的{}", sp.name.get(), st.name()),
        None => sp.name.get().to_string(),
    }
}

/// 別に集める冬越しの姿があるか（たまご・幼虫・さなぎ。成虫のまま冬を越す虫は成虫として集める）
pub fn has_stage(i: usize) -> bool {
    matches!(SPECIES[i].winter(), Some((st, _)) if st != crate::life::Stage::Adult)
}

/// 宿題に合格したか
pub fn mission_done(m: &Mission, stars: &[u8], stage_stars: &[u8]) -> bool {
    m.target_list().iter().all(|&(t, _)| match t {
        Target::Sp(i) => stars.get(i).copied().unwrap_or(0) >= 1,
        Target::Stage(i) => stage_stars.get(i).copied().unwrap_or(0) >= 1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targets_exist_and_can_be_found() {
        for m in MISSIONS {
            assert!(m.targets.len() >= 3, "{}", m.title.0);
            for &(t, _) in &m.target_list() {
                assert!(!places_of(t.entry()).is_empty());
                if let Target::Stage(i) = t {
                    assert!(has_stage(i), "{}", SPECIES[i].slug);
                }
            }
            // 宿題の虫が、そろって見られる月がある
            assert!((1..=12u8).any(|mo| m.target_list().iter().all(|&(t, _)| match t {
                Target::Sp(i) => SPECIES[i].in_season(mo),
                Target::Stage(i) => !SPECIES[i].in_season(mo),
            })), "{}", m.title.0);
        }
    }
}
