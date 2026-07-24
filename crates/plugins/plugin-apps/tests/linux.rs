#[cfg(target_os = "linux")]
mod linux_tests {
    use plugin_apps::linux::clean_exec;

    #[test]
    fn strips_bare_field_code() {
        assert_eq!(clean_exec("app --file %f"), "app --file");
    }

    #[test]
    fn strips_multiple_field_codes() {
        assert_eq!(clean_exec("app %U --flag"), "app --flag");
    }

    #[test]
    fn preserves_quoted_value() {
        assert_eq!(
            clean_exec(r#"app --arg="value" %U"#),
            r#"app --arg="value""#
        );
    }

    #[test]
    fn handles_plain_exec() {
        assert_eq!(clean_exec("firefox"), "firefox");
    }
}
