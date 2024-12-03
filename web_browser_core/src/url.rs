use alloc::{string::{String,ToString}, vec::Vec};

#[derive(Debug, Clone, PartialEq)]
pub struct Url {
    url: String,
    host: String,
    port: String,
    path: String,
    searchpart: String,
}

impl Url {
    pub fn new(url: String) -> Self {
        Self {
            url,
            host: "".to_string(),
            port: "".to_string(),
            path: "".to_string(),
            searchpart: "".to_string(),
        }
    }
}

impl Url {
    pub fn parse(&mut self) -> Result<Self, String> {
        if !self.is_http() {
            return Err("Only HTTP scheme is supported.".to_string());
        }
        self.host = self.extract_host();
        self.port = self.extract_port();
        self.path = self.extract_path();
        self.searchpart = self.extract_searchpart();

        Ok(self.clone())
    }
}

impl Url {
    fn is_http(&mut self) -> bool {
        if self.url.contains("http://") {
            return true
        }
        false
    }
}

impl Url {
    fn extract_host(&self) -> String {
        let url_parts = self.create_url_parts();

        if let Some(index) = url_parts[0].find(":") {
            url_parts[0][..index].to_string()
        } else {
            url_parts[0].to_string()
        }
    }
}

impl Url {
    fn extract_port(&self) -> String {
        let url_parts = self.create_url_parts();

        if let Some(index) = url_parts[0].find(":") {
            url_parts[0][index + 1..].to_string()
        } else {
            "80".to_string()
        }
    }
}

impl Url {
    fn extract_path(&self) -> String {
        let url_parts = self.create_url_parts();

        if url_parts.len() < 2 {
            return "".to_string();
        }

        let path_and_searchpart: Vec<&str> = url_parts[1].splitn(2, "?").collect();
        path_and_searchpart[0].to_string()
    }
}

impl Url {
    fn extract_searchpart(&self) -> String {
        let url_parts = self.create_url_parts();

        if url_parts.len() < 2 {
            return "".to_string();
        }

        let path_and_searchpart: Vec<&str> = url_parts[1].splitn(2, "?").collect();
        if path_and_searchpart.len() < 2 {
            "".to_string()
        } else {
            path_and_searchpart[1].to_string()
        }
    }
}

impl Url {
    fn create_url_parts(&self) -> Vec<&str> {
        self.url.trim_start_matches("http://").splitn(2, "/").collect()
    }
}
