use k_launcher_domain::{ResultId, ResultTitle, Score};

#[test]
fn newtype_result_id() {
    assert_eq!(ResultId::new("x").as_str(), "x");
}

#[test]
fn newtype_score() {
    assert_eq!(Score::new(42).value(), 42);
}

#[test]
fn newtype_title() {
    assert_eq!(ResultTitle::new("hello").as_str(), "hello");
}
