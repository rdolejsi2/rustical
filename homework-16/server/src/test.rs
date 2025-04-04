#[cfg(test)]
mod tests {
    use crate::file::{post_process_image, store_file};
    use common::util::base64_encode;
    use image::{ImageFormat, ImageReader};
    use std::fs;
    use std::fs::File;
    use std::io::{Cursor, Read};
    use tempfile::tempdir;

    // Test file storage
    #[test]
    fn test_store_file() {
        // Given
        let dir = tempdir().unwrap();
        let file_dir = dir.path().to_str().unwrap().to_string();
        let filename = "test.txt";
        let content = "Hello world";
        let content_encoded = base64_encode(content.as_ref());

        // When
        let result = store_file(filename, &file_dir, &content_encoded, None).unwrap();

        // Then
        assert!(result.contains("Stored"));
        // retrieve the file path from result last argument
        let parts: Vec<&str> = result.split_whitespace().collect();
        let stored_file_path = parts.last().unwrap();
        let stored_content = fs::read_to_string(stored_file_path).unwrap();
        assert_eq!(stored_content, content);
    }

    // Test image post-processing
    #[test]
    fn test_post_process_image() {
        // Given
        let source_file = "resources/2x2.jpg";
        let mut file = File::open(source_file).unwrap();
        let mut buffer = Vec::new();
        let size = file.read_to_end(&mut buffer);
        assert!(size.is_ok());
        assert!(buffer.len() > 0);

        // When
        let (processed_buffer, new_target_file, converted) =
            post_process_image(&*buffer, source_file).unwrap();

        // Then
        assert!(converted);
        assert_eq!(new_target_file, "resources/2x2.png");
        let reader = ImageReader::new(Cursor::new(processed_buffer));
        let format = reader.with_guessed_format().unwrap().format();
        assert_eq!(format, Some(ImageFormat::Png));
    }
}
