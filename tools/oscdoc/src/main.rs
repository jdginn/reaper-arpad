use regex::Regex;
use serde::Serialize;
use serde_yaml;
use std::fs;

#[derive(Debug, Serialize)]
struct OscDoc {
    osc_address: String,
    arguments: Vec<OscArg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    direction: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    comments: Vec<String>,
}

#[derive(Debug, Serialize)]
struct OscArg {
    name: String,
    r#type: String,
    description: String,
}

fn main() {
    let src = fs::read_to_string("src/osc_routes.rs").expect("No src/lib.rs found");
    let re = Regex::new(r"(?s)/// ?@osc-doc\n(.*?)(?:fn (\w+)[^\n]*\{)").unwrap();

    let mut docs = Vec::new();

    for cap in re.captures_iter(&src) {
        let docblock = &cap[1];

        let mut comments = Vec::new();
        let mut osc_address = None;
        let mut arguments = Vec::new();

        let osc_re = Regex::new(r"^.*///\s*OSC Address:\s*(.*)$").unwrap();
        let arg_re = Regex::new(r"^.*///\s*-\s*(\w+)\s*\((\w+)\):\s*(.*)$").unwrap();

        let mut in_osc_section = false;

        let mut direction = None;

        for line in docblock.lines() {
            if line.contains("@readonly") {
                direction = Some("readonly".to_string());
                continue;
            }
            if line.contains("@writeonly") {
                direction = Some("writeonly".to_string());
                continue;
            }

            if osc_re.is_match(line) {
                osc_address = Some(osc_re.captures(line).unwrap()[1].to_string());
                in_osc_section = true;
                continue;
            }
            if in_osc_section {
                if let Some(arg_cap) = arg_re.captures(line) {
                    arguments.push(OscArg {
                        name: arg_cap[1].to_string(),
                        r#type: arg_cap[2].to_string(),
                        description: arg_cap[3].to_string(),
                    });
                }
            } else {
                // Collect as comment (strip leading /// and whitespace)
                comments.push(line.trim_start_matches("///").trim().to_string());
            }
        }

        docs.push(OscDoc {
            osc_address: osc_address.unwrap_or_default(),
            arguments,
            direction,
            comments: comments.into_iter().filter(|c| !c.is_empty()).collect(),
        });
    }

    // Output yaml
    let yaml = serde_yaml::to_string(&docs).unwrap();
    fs::write("osc_docs.yaml", yaml).unwrap();
}
