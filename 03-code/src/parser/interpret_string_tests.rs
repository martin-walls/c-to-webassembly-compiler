#[cfg(test)]
mod interpret_string_tests {
    use super::super::interpret_string;

    #[test]
    fn string_with_no_escapes() {
        assert_eq!(interpret_string("hello world"), Ok("hello world".into()));
    }

    #[test]
    fn empty_string() {
        assert_eq!(interpret_string(""), Ok("".into()));
    }

    #[test]
    fn escaped_single_quote() {
        assert_eq!(interpret_string("hello\\'world"), Ok("hello\'world".into()));
    }

    #[test]
    fn escaped_double_quote() {
        assert_eq!(interpret_string("foo\\\" bar"), Ok("foo\" bar".into()));
    }

    #[test]
    fn escaped_question_mark() {
        assert_eq!(interpret_string("foobaz\\?"), Ok("foobaz?".into()));
    }

    #[test]
    fn escaped_backslash() {
        assert_eq!(interpret_string("\\\\"), Ok("\\".into()));
    }

    #[test]
    fn escaped_newline() {
        assert_eq!(interpret_string("\\n"), Ok("\n".into()));
    }

    #[test]
    fn escaped_carriage_return() {
        assert_eq!(interpret_string("\\r"), Ok("\r".into()));
    }

    #[test]
    fn escaped_tab() {
        assert_eq!(interpret_string("foo\\tbar"), Ok("foo\tbar".into()));
    }

    #[test]
    fn escaped_hex_code() {
        assert_eq!(interpret_string("\\xc0"), Ok("\u{c0}".into()));
    }

    #[test]
    fn escaped_octal_code() {
        assert_eq!(interpret_string("\\144"), Ok("d".into()));
    }

    #[test]
    fn escaped_octal_is_limited_to_three_digits() {
        assert_eq!(interpret_string("\\14356"), Ok("c56".into()));
    }

    #[test]
    fn escaped_null() {
        assert_eq!(interpret_string("\\0"), Ok("\0".into()));
    }

    #[test]
    fn invalid_escape_returns_err() {
        assert!(matches!(interpret_string("\\g"), Err(_)));
    }
}