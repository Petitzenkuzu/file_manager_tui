pub mod string {

    pub fn expand_or_truncate(mut string: String, max_width: usize) -> String {
        if string.chars().count() > max_width {
            string.truncate(max_width-4);
            string.push_str("...");
            string = format!("{}{}", string, " ".repeat(max_width-string.chars().count()));
        }
        else {
            string = format!("{}{}", string, " ".repeat(max_width - string.chars().count()));
        }
        string
    }
    
    pub fn center(text: String, max_width: usize) -> String {
        assert!(max_width >= text.len(), "Max width must be greater than or equal to the text length");
        let padding = " ".repeat((max_width - text.len())/2);
        format!("{}{}{}", padding, text, padding)
    }

}

pub mod float {
    pub fn truncate(float: f32, precision: usize) -> f32 {
        if precision == 0 {
            return float.trunc();
        }
        let factor = 10.0_f32.powi(precision as i32);
        (float * factor).trunc() / factor
    }
}

#[cfg(test)]
mod test {
    use super::string::{expand_or_truncate, center};
    use super::float::truncate;
    #[test]
    fn test_expand_or_truncate() {
        assert_eq!(expand_or_truncate("Hello".to_string(), 10), "Hello     ".to_string());
        assert_eq!(expand_or_truncate("Hello World".to_string(), 10), "Hello ... ".to_string());
        assert_eq!(expand_or_truncate("Hello".to_string(), 5), "Hello".to_string());
    }
    #[test]
    fn test_center() {
        assert_eq!(center("Hello".to_string(), 10), "  Hello  ".to_string());
        assert_eq!(center("Hello World".to_string(), 15), "  Hello World  ".to_string());
        assert_eq!(center("Hello".to_string(), 6), "Hello".to_string());
        assert_eq!(center("Hello ".to_string(), 6), "Hello ".to_string());
        assert_eq!(center(" Hello".to_string(), 6), " Hello".to_string());
        assert_eq!(center("Hello ".to_string(), 8), " Hello  ".to_string());
    }
    #[test]
    fn test_truncate() {
        assert_eq!(truncate(1.23456789, 2), 1.23);
        assert_eq!(truncate(1.23456789, 0), 1.0);
        assert_eq!(truncate(1.23456789, 3), 1.234);
    }
}