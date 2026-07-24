use plugin_apps::frecency::FrecencyStore;

#[test]
fn record_increments_count() {
    let store = FrecencyStore::new_for_test();
    store.record("app-firefox");
    store.record("app-firefox");
    assert!(store.frecency_score("app-firefox") > 0);
}

#[test]
fn top_ids_returns_sorted_order() {
    let store = FrecencyStore::new_for_test();
    store.record("app-firefox");
    store.record("app-code");
    store.record("app-code");
    store.record("app-code");
    let top = store.top_ids(2);
    assert_eq!(top[0], "app-code");
    assert_eq!(top[1], "app-firefox");
}
