use anyhow::Result;

pub struct TemplateEngine;

impl TemplateEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, template: &str, variables: &std::collections::HashMap<String, String>) -> Result<String> {
        let mut result = template.to_string();
        
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }
        
        Ok(result)
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
} 