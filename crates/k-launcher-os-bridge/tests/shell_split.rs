use k_launcher_os_bridge::shell_split;

#[test]
fn split_simple() {
    assert_eq!(shell_split("firefox"), vec!["firefox"]);
}

#[test]
fn split_with_args() {
    assert_eq!(
        shell_split("firefox --new-window"),
        vec!["firefox", "--new-window"]
    );
}

#[test]
fn split_quoted_path() {
    assert_eq!(shell_split(r#""My App" --flag"#), vec!["My App", "--flag"]);
}

#[test]
fn split_quoted_with_spaces() {
    assert_eq!(
        shell_split(r#"env "FOO BAR" baz"#),
        vec!["env", "FOO BAR", "baz"]
    );
}

#[test]
fn split_empty() {
    assert!(shell_split("").is_empty());
}

#[test]
fn split_extra_whitespace() {
    assert_eq!(shell_split("  a   b  "), vec!["a", "b"]);
}
