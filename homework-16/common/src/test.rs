#[cfg(test)]
mod tests {
    use crate::cli;
    use crate::util::{base64_decode, base64_encode};
    use cli::{parse_args, CliArg, HOST_DEFAULT, PORT_DEFAULT};

    // Test CLI parsing
    #[test]
    fn given_no_parameters_specified_when_cli_parsed_then_defaults_returned() {
        // Given
        let args = vec![CliArg::Host, CliArg::Port];
        let app_name = "test_app";

        // When
        let result = parse_args(app_name, &args).unwrap();

        // Then
        assert_eq!(result[0], HOST_DEFAULT);
        assert_eq!(result[1], PORT_DEFAULT);
    }

    #[test]
    fn given_text_when_base64_encode_and_base64_decode_then_same_text() {
        // Given
        let text = "Hello, world!";

        // When
        let encoded = base64_encode(text.as_bytes());
        let decoded = base64_decode(&encoded).unwrap();

        // Then
        assert_eq!(text.as_bytes(), &decoded[..]);
    }
}
