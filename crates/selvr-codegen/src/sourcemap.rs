//! VLQ-encoded source map generator (spec: source-map v3).
//!
//! Usage:
//!   let mut sm = SourceMap::new("output.js", &["input.self"]);
//!   // For each generated token, call sm.add_mapping(...)
//!   let json = sm.to_json();


pub struct SourceMap {
    /// Generated file name.
    pub file: String,
    /// Original source file paths.
    pub sources: Vec<String>,
    /// Accumulates raw mapping entries before encoding.
    mappings: Vec<Vec<Mapping>>,
    /// Current state for delta encoding.
    #[allow(dead_code)]
    prev_src_line: i64,
    #[allow(dead_code)]
    prev_src_col: i64,
    #[allow(dead_code)]
    prev_src_file: i64,
    #[allow(dead_code)]
    prev_name: i64,
}

#[derive(Debug, Clone)]
pub struct Mapping {
    pub gen_col: u32,
    pub src_file: Option<u32>,
    pub src_line: Option<u32>,
    pub src_col: Option<u32>,
    pub name: Option<u32>,
}

impl SourceMap {
    pub fn new(file: impl Into<String>, sources: &[&str]) -> Self {
        Self {
            file: file.into(),
            sources: sources.iter().map(|s| s.to_string()).collect(),
            mappings: vec![vec![]],
            prev_src_line: 0,
            prev_src_col: 0,
            prev_src_file: 0,
            prev_name: 0,
        }
    }

    pub fn new_line(&mut self) {
        self.mappings.push(vec![]);
    }

    pub fn add_mapping(&mut self, m: Mapping) {
        self.mappings.last_mut().unwrap().push(m);
    }

    pub fn to_json(&self) -> String {
        let sources_json: Vec<String> = self.sources.iter().map(|s| format!("{s:?}")).collect();
        let mappings = self.encode_mappings();
        format!(
            r#"{{"version":3,"file":{:?},"sources":[{}],"mappings":{:?}}}"#,
            self.file,
            sources_json.join(","),
            mappings,
        )
    }

    fn encode_mappings(&self) -> String {
        // Stub — full VLQ encoding to be implemented.
        // Returns an empty mapping string so the source map is structurally valid.
        self.mappings.iter()
            .map(|_| String::new())
            .collect::<Vec<_>>()
            .join(";")
    }
}

#[allow(dead_code)]
fn encode_vlq(value: i64) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    let signed = if value < 0 { ((-value) << 1) | 1 } else { value << 1 };
    let mut vlq = signed;
    loop {
        let mut digit = vlq & 0x1F;
        vlq >>= 5;
        if vlq > 0 { digit |= 0x20; }
        result.push(CHARS[digit as usize] as char);
        if vlq == 0 { break; }
    }
    result
}
