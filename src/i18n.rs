// UI text, from locales.csv (built into the program).
//
// locales.csv is one sheet: one row per language, one column per message.
// To add a language, add one row whose "lang" cell is a language tag
// ("fr", "pt-BR", "zh-Hant", ...). Write line breaks inside a cell as \n and
// keep {0}, {1} placeholders. Empty cells fall back to English.
// The "name" column is the language's own name, shown in the Language menu.

use std::collections::HashMap;

const SHEET: &str = include_str!("../locales.csv");

pub struct Strings {
    pub lang: String,
    /// (code, own name) in sheet order
    pub languages: Vec<(String, String)>,
    rows: HashMap<String, HashMap<String, String>>,
}

fn parse_csv(text: &str) -> Vec<Vec<String>> {
    let text = text.trim_start_matches('\u{feff}');
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if quoted {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    field.push('"');
                    chars.next();
                } else {
                    quoted = false;
                }
            } else {
                field.push(c);
            }
        } else {
            match c {
                '"' => quoted = true,
                ',' => row.push(std::mem::take(&mut field)),
                '\r' => {}
                '\n' => {
                    row.push(std::mem::take(&mut field));
                    rows.push(std::mem::take(&mut row));
                }
                _ => field.push(c),
            }
        }
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    rows
}

impl Strings {
    /// `override_lang`: "language" from ccdc.json; empty = follow the system.
    pub fn load(override_lang: &str) -> Strings {
        let rows = parse_csv(SHEET);
        let header = rows.first().cloned().unwrap_or_default();
        let mut table = HashMap::new();
        let mut languages = Vec::new();
        for r in rows.iter().skip(1) {
            let code = r.first().map(|s| s.trim().to_string()).unwrap_or_default();
            if code.is_empty() {
                continue;
            }
            let map: HashMap<String, String> = header.iter().cloned().zip(r.iter().cloned()).collect();
            let name = map.get("name").filter(|n| !n.is_empty()).cloned().unwrap_or_else(|| code.clone());
            languages.push((code.clone(), name));
            table.insert(code.to_lowercase(), map);
        }
        let lang = pick(&table, override_lang, &crate::platform::ui_language());
        Strings { lang, languages, rows: table }
    }

    /// Text for `key` with {0}, {1}, ... filled in.
    pub fn t(&self, key: &str, args: &[&dyn std::fmt::Display]) -> String {
        let get = |lang: &str| self.rows.get(&lang.to_lowercase()).and_then(|m| m.get(key)).filter(|s| !s.is_empty()).cloned();
        let mut s = get(&self.lang).or_else(|| get("en")).unwrap_or_else(|| key.to_string());
        s = s.replace("\\n", "\n");
        for (i, a) in args.iter().enumerate() {
            s = s.replace(&format!("{{{i}}}"), &a.to_string());
        }
        s
    }

}

/// The sheet's row for the override, else the system language: full tag
/// (pt-BR), then Chinese script (zh-Hant / zh-Hans), then the language (pt), then English.
fn pick(table: &HashMap<String, HashMap<String, String>>, override_lang: &str, system: &str) -> String {
    let mut candidates: Vec<String> = Vec::new();
    if !override_lang.is_empty() {
        candidates.push(override_lang.to_string());
    }
    let sys = system.replace('_', "-");
    let parts: Vec<&str> = sys.split('-').collect();
    let base = parts.first().copied().unwrap_or("en").to_lowercase();
    candidates.push(sys.clone());
    if parts.len() >= 2 {
        candidates.push(format!("{}-{}", parts[0], parts[1]));
    }
    if base == "zh" {
        let up = sys.to_uppercase();
        let hant = up.contains("HANT") || up.contains("-TW") || up.contains("-HK") || up.contains("-MO");
        candidates.push(if hant { "zh-Hant" } else { "zh-Hans" }.into());
    }
    candidates.push(base);
    candidates.push("en".into());
    for c in candidates {
        if let Some(row) = table.get(&c.to_lowercase()) {
            return row.get("lang").map(|s| s.trim().to_string()).unwrap_or(c);
        }
    }
    "en".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sheet_parses() {
        let s = Strings::load("ko");
        assert_eq!(s.lang, "ko");
        assert!(s.languages.len() >= 20);
        assert_eq!(s.t("speed_normal", &[]), "보통 (60MB/s)");
        assert!(s.t("ev_copy_done", &[&3, &"1.2"]).contains('3'));
    }

    #[test]
    fn picks_languages() {
        let s = Strings::load("");
        let t = &s.rows;
        assert_eq!(pick(t, "", "zh-TW"), "zh-Hant");
        assert_eq!(pick(t, "", "zh-Hans-CN"), "zh-Hans");
        assert_eq!(pick(t, "", "pt-BR"), "pt");
        assert_eq!(pick(t, "", "xx-YY"), "en");
        assert_eq!(pick(t, "ja", "ko-KR"), "ja");
    }
}
