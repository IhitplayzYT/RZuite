pub mod Helper{
 

use regex::Regex;

pub fn non_pub_decl(source: &str) -> Vec<usize>{
    let re = Regex::new(
        r"(?ms)^(?:\s*#\[[^\]]*]\s*\n)*(?!\s*pub\b)\s*(?:fn|struct)\b",
    )
    .unwrap();

    re.find_iter(source).map(|m| m.start()).collect()
}

}