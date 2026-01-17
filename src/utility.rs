pub mod string {

    pub fn expand_or_truncate(mut string: String, max_width: usize) -> String {
        if string.len() > max_width {
            string.truncate(max_width);
        }
        else {
            string = format!("{}{}", string, " ".repeat(max_width - string.len()));
        }
        string
    }
    
    pub fn center(text: String, max_width: usize) -> String {
        assert!(max_width >= text.len(), "Max width must be greater than or equal to the text length");
        let padding = " ".repeat((max_width - text.len())/2);
        format!("{}{}{}", padding, text, padding)
    }

}

#[cfg(test)]
mod test {
    use super::string::{expand_or_truncate, center};
    #[test]
    fn test_expand_or_truncate() {
        assert_eq!(expand_or_truncate("Hello".to_string(), 10), "Hello     ".to_string());
    }
    #[test]
    fn test_center() {
        assert_eq!(center("Hello".to_string(), 10), "  Hello  ".to_string());
    }
}
