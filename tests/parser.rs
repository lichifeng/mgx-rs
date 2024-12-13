use mgx::from_file;
use mgx::Version;
use mgx::draw_map;

#[test]
fn aok_test() {
    let filename = "tests/recs/aok_4v4_fast.mgl";
    let (rec, parser) = from_file(filename).unwrap(); // This guid is from MgxParser
    println!("aok versave: {:?}", rec.versave);
    draw_map(&rec, &parser, &format!("{}.png", filename)).unwrap();
    std::fs::remove_file(format!("{}.png", filename)).unwrap();
    assert_eq!(rec.ver, Some(Version::AoK));
}

#[test]
fn aoc10_test() {
    let filename = "tests/recs/28083e6497dc1a0a3f8ca3a54c2622c2.mgx";
    let (mut rec, parser) = from_file(filename).unwrap(); // This guid is from MgxParser
    println!("aoc10 versave: {:?}", rec.versave);
    draw_map(&rec, &parser, &format!("{}.png", filename)).unwrap();
    std::fs::remove_file(format!("{}.png", filename)).unwrap();
    assert_eq!(rec.ver, Some(Version::AoC10a));
    println!("Encoding: {:?}", rec.detect_encoding().unwrap());
    rec.translate("zh");
    println!("{:?}", rec.dump_json().unwrap());
}

#[test]
fn aoc10c_test() {
    let filename = "tests/recs/aoc-1.0c.mgx";
    let (mut rec, parser) = from_file(filename).unwrap();
    println!("aoc10c versave: {:?}", rec.versave);
    draw_map(&rec, &parser, &format!("{}.png", filename)).unwrap();
    std::fs::remove_file(format!("{}.png", filename)).unwrap();
    assert_eq!(rec.ver, Some(Version::AoC10c));
    println!("Encoding: {:?}", rec.detect_encoding().unwrap());
    println!("{:?}", rec.dump_json().unwrap());
}

#[test]
fn up15_test() {
    let filename = "tests/recs/up1.5.mgz";
    let (mut rec, parser) = from_file(filename).unwrap();
    println!("up15 versave: {:?}", rec.versave);
    draw_map(&rec, &parser, &format!("{}.png", filename)).unwrap();
    std::fs::remove_file(format!("{}.png", filename)).unwrap();
    assert_eq!(rec.ver, Some(Version::UP15));
    println!("Encoding: {:?}", rec.detect_encoding().unwrap());
    println!("{:?}", rec.dump_json().unwrap());
}

#[test]
fn up14_scenario_test() {
    let filename = "tests/recs/scenario-with-messages.mgz";
    let (rec, parser) = from_file(filename).unwrap();
    draw_map(&rec, &parser, &format!("{}.png", filename)).unwrap();
    std::fs::remove_file(format!("{}.png", filename)).unwrap();
    assert!(rec.verscenario.unwrap() - 1.22 < 0.0001);
    assert_eq!(rec.ver, Some(Version::UP14));
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
fn ai_test() {
    let filename = "tests/recs/1v7_hardest_spain_aoc10.mgx";
    let (rec, parser) = from_file(filename).unwrap();
    draw_map(&rec, &parser, &format!("{}.png", filename)).unwrap();
    std::fs::remove_file(format!("{}.png", filename)).unwrap();
    assert_eq!(rec.include_ai, Some(true));
    println!("speed: {:?}", rec.speed_raw);
    println!("map: {:?}, {:?}", rec.mapx, rec.mapy);
    println!("Encoding: {:?}", rec.detect_encoding().unwrap());
}
