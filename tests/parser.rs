use mgx::from_file;
use mgx::Version;
use mgx::draw_map;

#[test]
fn aok_trial_test() {
    let filename = "tests/recs/aok_trial.mgl";
    let (rec, parser) = from_file(filename).unwrap();
    
    assert_eq!(rec.ver, Some(Version::AoKTrial));
    assert_eq!(rec.duration, 1933820);
    assert_eq!(rec.matchup, Some(vec![1, 1, 1]));
    assert_eq!(rec.guid, Some("c346c0c9238f25317bbdb27246b4d56a".to_string()));

    draw_map(&rec, &parser, &format!("{}.png", filename)).unwrap();
    std::fs::remove_file(format!("{}.png", filename)).unwrap();
}

#[test]
fn aok_test() {
    // Conquest Game: Win this game by destroying all enemy villagers, military units, boats, and buildings.
    // Map Size: Large (8 player)
    // Map Type: Black Forest
    // Age: Post-Imperial Age
    // Resources: Standard
    // Difficulty Level: Easiest
    // Fixed Positions: Yes
    // Reveal Map: Yes
    // Full Tech Tree: No
    // Enable Cheating: No
    // Population Limit: 200

    let filename = "tests/recs/aok_4v4_fast.mgl";
    let (mut rec, parser) = from_file(filename).unwrap();
    rec.translate("en");
    assert_eq!(rec.ver, Some(Version::AoK));
    assert_eq!(rec.speed, Some("Fast".to_string()));
    assert_eq!(rec.duration, 9770100);
    assert_eq!(rec.matchup, Some(vec![4, 4]));
    assert_eq!(rec.guid, Some("f94380bd153af62786c7ad2a0e01d114".to_string()));

    draw_map(&rec, &parser, &format!("{}.png", filename)).unwrap();
    std::fs::remove_file(format!("{}.png", filename)).unwrap();
}

/// This test also checks if different views of the same game have the same GUID
#[test]
fn aoc10a_test() {
    // 征服制游戏: 消灭敌人所有村民,部队，船舰和建筑.
    // 地图尺寸: 大型 (8个玩家)
    // 地图类型: 阿拉伯半岛
    // 年代: 标准
    // 资源: 标准
    // 难度: 标准
    // 固定地点: 是
    // 显示地图: 否
    // 完全科技树: 否
    // 允许作弊: 否
    // 人口上限: 200
    let filename = "tests/recs/aoc10a_4v4_standard_1.mgx";
    let (mut rec, parser) = from_file(filename).unwrap();
    rec.translate("zh");
    assert_eq!(rec.md5, Some("75b33de419109bbdff74aa4e51adf801".to_string()));
    assert_eq!(rec.ver, Some(Version::AoC10a));
    assert_eq!(rec.speed, Some("正常".to_string()));
    assert_eq!(rec.duration, 3235875);
    assert_eq!(rec.matchup, Some(vec![4, 4]));
    assert_eq!(rec.guid, Some("aead4c4da21c523f458be8e8399227e1".to_string()));
    assert_eq!(rec.chat.len(), 2);
    assert!(rec.haswinner);

    let filename2 = "tests/recs/aoc10a_4v4_standard_2.mgx";
    let (rec2, _) = from_file(filename2).unwrap();
    assert_eq!(rec2.guid, Some("aead4c4da21c523f458be8e8399227e1".to_string()));

    draw_map(&rec, &parser, &format!("{}.png", filename)).unwrap();
    std::fs::remove_file(format!("{}.png", filename)).unwrap();

    let guid = "ecd4756dc58f671707cfbe18bdd72f71".to_string();
    let filename3 = "tests/recs/aoc10a_303_3p.mgx";
    let (rec3, _) = from_file(filename3).unwrap();
    assert_eq!(rec3.guid, Some(guid.clone()));

    let filename4 = "tests/recs/aoc10a_303_5p.mgx";
    let (rec4, _) = from_file(filename4).unwrap();
    assert_eq!(rec4.guid, Some(guid));
}

#[test]
fn aoc10a_1v7_ai_test() {
    let filename = "tests/recs/1v7_hardest_spain_aoc10.mgx";
    let (rec, parser) = from_file(filename).unwrap();
    assert_eq!(rec.include_ai, Some(true));
    assert_eq!(rec.matchup, Some(vec![1, 7]));

    draw_map(&rec, &parser, &format!("{}.png", filename)).unwrap();
    std::fs::remove_file(format!("{}.png", filename)).unwrap();
}

#[test]
fn aoc10c_test() {
    // 征服制游戏: 消灭敌人所有村民,部队，船舰和建筑.
    // 地图尺寸: 微型 (2 玩家)
    // 地图类型: 阿拉伯半岛
    // 年代: 标准
    // 资源: 标准
    // 难度: 标准
    // 固定地点: 是
    // 显示地图: 否
    // 完全科技树: 否
    // 允许作弊: 否
    // 人口上限: 200
    let filename = "tests/recs/aoc10c_1v1_with_spectator.mgx";
    let (mut rec, parser) = from_file(filename).unwrap();
    rec.translate("en");
    assert_eq!(rec.ver, Some(Version::AoC10c));
    assert_eq!(rec.speed, Some("Normal".to_string()));
    assert_eq!(rec.duration, 1710630);
    assert_eq!(rec.matchup, Some(vec![1, 1]));
    assert_eq!(rec.guid, Some("1e3be847550bcc56008d952c2241e7ff".to_string()));
    assert!(rec.haswinner);

    draw_map(&rec, &parser, &format!("{}.png", filename)).unwrap();
    std::fs::remove_file(format!("{}.png", filename)).unwrap();
}

#[test]
fn aoc10c_ai_test() {
    let filename = "tests/recs/aoc10c_with_AI.mgx";
    let (rec, parser) = from_file(filename).unwrap();
    assert_eq!(rec.include_ai, Some(true));
    
    draw_map(&rec, &parser, &format!("{}.png", filename)).unwrap();
    std::fs::remove_file(format!("{}.png", filename)).unwrap();
}

/// This record contains a nextpos next to header length
#[test]
fn up12_ai_test() {
    // Partie en mode Conquête : Le joueur qui détruit tous les villageois, unités militaires, navires de combats et bâtiments ennemis remporte la partie.
    // Taille de la carte : Très longue
    // Type de carte : Montagnes
    // Âge : Standard
    // Ressources : Standard
    // Niveau de difficulté : Très difficile
    // Positions fixes : Oui
    // Révéler la carte : Non
    // Arbre complet des technologies : Non
    // Permettre le mode triche : Non
    // Limite de population : 1000
    let filename = "tests/recs/up12_3v3_with_ai.mgx";
    let (rec, parser) = from_file(filename).unwrap();
    assert_eq!(rec.include_ai, Some(true));
    assert_eq!(rec.matchup, Some(vec![3, 3]));
    assert_eq!(rec.poplimit, Some(1000));
    
    draw_map(&rec, &parser, &format!("{}.png", filename)).unwrap();
    std::fs::remove_file(format!("{}.png", filename)).unwrap();
}

#[test]
fn up14_scenario_test() {
    let filename = "tests/recs/scenario-with-messages.mgz";
    let (rec, parser) = from_file(filename).unwrap();
    assert!(rec.verscenario.unwrap() - 1.22 < 0.0001);
    assert_eq!(rec.ver, Some(Version::UP14));

    draw_map(&rec, &parser, &format!("{}.png", filename)).unwrap();
    std::fs::remove_file(format!("{}.png", filename)).unwrap();
}

#[test]
fn up15_test() {
    let filename = "tests/recs/up1.5.mgz";
    let (rec, parser) = from_file(filename).unwrap();    
    assert_eq!(rec.ver, Some(Version::UP15));

    draw_map(&rec, &parser, &format!("{}.png", filename)).unwrap();
    std::fs::remove_file(format!("{}.png", filename)).unwrap();

    // UP15 records often have unexpected bytes(paddings) in body, read operations or some other int data by first 2 bytes(LE)
    // may works for some of them. Like:
    // let op_type = val!(b.get_i32());
    // to
    // let op_type = val!(b.get_i16());
    // b.mov(2);
    // Not sure if this will have any side effects on other versions, so not implemented.
    // let filename2 = "tests/recs/up15_with_bad_command.mgz";
    // let (rec2, _) = from_file(filename2).unwrap();
    // assert_eq!(rec2.ver, Some(Version::UP15));
}

#[test]
fn hd_test() {
    let filename = "tests/recs/HD-FE.mgx2";
    match from_file(filename) {
        Err(e) => {
            assert!(e.to_string().contains("DE/HD or higher versions are not supported"));
        }
        _ => (),
    }
}

#[test]
fn de63_test() {
    let filename = "tests/recs/de-63.0.aoe2record";
    match from_file(filename) {
        Err(e) => {
            assert!(e.to_string().contains("DE/HD or higher versions are not supported"));
        }
        _ => (),
    }
}

#[test]
fn headerlen_missing_test() {
    //! This record has no header length(0x00 0x00 0x00 0x00)
    let filename = "tests/recs/headerlen_is_missing.mgx";
    let (rec, _) = from_file(filename).unwrap();    
    assert_eq!(rec.ver, Some(Version::AoC10a));
}

#[test]
fn next_chapter_test() {
    //! This record has no header length(0x00 0x00 0x00 0x00)
    let filename = "tests/recs/next_chapter_1.mgx";
    let (rec, _) = from_file(filename).unwrap();    
    assert_eq!(rec.ver, Some(Version::AoC10c));
    assert_eq!(rec.duration, 3179880);
}

#[test]
fn extra_bytes_after_header_test() {
    //! This record has extra bytes after the compressed header
    let filename = "tests/recs/extra_bytes_after_compressed_header.mgx";
    let (rec, _) = from_file(filename).unwrap();
    assert_eq!(rec.ver, Some(Version::AoC10a));
}
