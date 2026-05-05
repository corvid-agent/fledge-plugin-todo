use std::env;
use std::path::Path;

use fledge_plugin_todo::{format_json, format_table, walk};

fn main() {
    let args: Vec<String> = env::args().collect();
    let dir = args.get(1).map(|s| s.as_str()).unwrap_or(".");
    let json = args.iter().any(|a| a == "--json");

    let mut findings = Vec::new();
    walk(Path::new(dir), &mut findings);

    if json {
        print!("{}", format_json(&findings));
    } else {
        print!("{}", format_table(&findings));
    }
}
