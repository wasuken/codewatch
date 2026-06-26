use regex::Regex;

pub fn parse_imports(file_content: &str, pattern: &str) -> Vec<String> {
    let re = match Regex::new(pattern) {
        Ok(re) => re,
        Err(_) => return Vec::new(),
    };
    let mut imports = Vec::new();
    for cap in re.captures_iter(file_content) {
        if let Some(m) = cap.get(1) {
            imports.push(m.as_str().to_string());
        }
    }
    imports
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_imports() {
        let content = r#"
            import { foo } from 'bar';
            import baz from "./baz";
            const x = require('lib');
        "#;
        let pattern = r#"from\s+['"]([^'"]+)['"]"#;
        let imports = parse_imports(content, pattern);
        assert_eq!(imports, vec!["bar".to_string(), "./baz".to_string()]);
    }

    #[test]
    fn test_parse_imports_empty() {
        let content = "const x = 10;";
        let pattern = r#"from\s+['"]([^'"]+)['"]"#;
        let imports = parse_imports(content, pattern);
        assert!(imports.is_empty());
    }
}


