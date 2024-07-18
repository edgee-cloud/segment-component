pub trait StringExt {
    fn or<'a>(&'a self, default: &'a str) -> &str;
}

impl StringExt for str {
    fn or<'a>(&'a self, default: &'a str) -> &str {
        if self.is_empty() {
            default
        } else {
            self
        }
    }
}

impl StringExt for String {
    fn or<'a>(&'a self, default: &'a str) -> &str {
        if self.is_empty() {
            default
        } else {
            self
        }
    }
}
