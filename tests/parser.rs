use mgx::from_file::from_file;
use mgx::record::Version;

#[test]
fn aok_test() {
    let filename = "tests/recs/aok_4v4_fast.mgl";
    let (retval, parser) = from_file(filename).unwrap(); // This guid is from MgxParser
    println!("aok versave: {:?}", retval.versave);
    parser.draw_map(&format!("{}.png", filename)).unwrap();
    std::fs::remove_file(format!("{}.png", filename)).unwrap();
    assert_eq!(retval.ver, Some(Version::AoK));
}

#[test]
fn aoc10_test() {
    let filename = "tests/recs/28083e6497dc1a0a3f8ca3a54c2622c2.mgx";
    let (retval, parser) = from_file(filename).unwrap(); // This guid is from MgxParser
    println!("aoc10 versave: {:?}", retval.versave);
    parser.draw_map(&format!("{}.png", filename)).unwrap();
    std::fs::remove_file(format!("{}.png", filename)).unwrap();
    assert_eq!(retval.ver, Some(Version::AoC10a));
    println!("Encoding: {:?}", retval.detect_encoding().unwrap());

    println!("{:?}", retval.dump_json().unwrap());
}

#[test]
fn aoc10c_test() {
    let filename = "tests/recs/aoc-1.0c.mgx";
    let (retval, parser) = from_file(filename).unwrap();
    println!("aoc10c versave: {:?}", retval.versave);
    parser.draw_map(&format!("{}.png", filename)).unwrap();
    std::fs::remove_file(format!("{}.png", filename)).unwrap();
    assert_eq!(retval.ver, Some(Version::AoC10c));
    println!("Encoding: {:?}", retval.detect_encoding().unwrap());
}

#[test]
fn up15_test() {
    let filename = "tests/recs/up1.5.mgz";
    let (retval, parser) = from_file(filename).unwrap();
    println!("up15 versave: {:?}", retval.versave);
    parser.draw_map(&format!("{}.png", filename)).unwrap();
    std::fs::remove_file(format!("{}.png", filename)).unwrap();
    assert_eq!(retval.ver, Some(Version::UP15));
    println!("Encoding: {:?}", retval.detect_encoding().unwrap());
    println!("{:?}", retval.dump_json().unwrap());
}

#[test]
fn up14_scenario_test() {
    let filename = "tests/recs/scenario-with-messages.mgz";
    let (retval, parser) = from_file(filename).unwrap();
    parser.draw_map(&format!("{}.png", filename)).unwrap();
    std::fs::remove_file(format!("{}.png", filename)).unwrap();
    assert!(retval.verscenario.unwrap() - 1.22 < 0.0001);
    assert_eq!(retval.ver, Some(Version::UP14));
    println!("Encoding: {:?}", retval.detect_encoding().unwrap());
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
    let (retval, parser) = from_file(filename).unwrap();
    parser.draw_map(&format!("{}.png", filename)).unwrap();
    std::fs::remove_file(format!("{}.png", filename)).unwrap();
    assert_eq!(retval.include_ai, Some(true));
    println!("speed: {:?}", retval.speed);
    println!("map: {:?}, {:?}", retval.mapx, retval.mapy);
    println!("Encoding: {:?}", retval.detect_encoding().unwrap());
}
