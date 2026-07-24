use std::{fs, path::Path};
use regex::Regex;

use crate::helper::Helper::non_pub_decl;

mod helper;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<String>>();
    let cwd = std::env::current_dir().unwrap();    


    match &args[0].to_lowercase()[..]{
        "gen_modules" | "gen_mods" | "module_init" | "mod_init" => {
    let fpath = cwd.join(Path::new(&args[1]));
    let mods = std::fs::read_to_string(fpath).unwrap();
    mods.lines().for_each(|x| {
        let fname = x.split_whitespace().last().unwrap().to_string();
        let mod_name= &fname[..fname.len() - 1];
        let fname= mod_name.to_string() + ".rs";
        std::fs::write(
            cwd.join(Path::new(&fname)),
            format!("pub mod {}{{\n}}", mod_name.to_ascii_lowercase()),
        )
        .unwrap();
    });
},
    "mk_pub" | "pub" => {
        let fpath = cwd.join(Path::new(&args[1]));
        let mut contents = fs::read_to_string(&fpath).unwrap();
        let mut idxs = non_pub_decl(&contents[..]);
        idxs.reverse();
        for i in idxs{
            contents.insert_str(i, "pub ");
        }
        std::fs::write(fpath, contents).unwrap();
    },
    "rm_unused_import" | "rm_import" => {
         let fpath = cwd.join(Path::new(&args[1]));
         let contents = fs::read_to_string(&fpath).unwrap();
         let mut imports = vec![];
         contents.split("\n").enumerate().for_each(|x| if x.1.starts_with("use ") {imports.push(x);});
         
         let mut unused_imports = vec![];

         for (idx, v) in imports {
            let l = v.len();
            let mut i = 0;
            let v: Vec<char> = v.chars().collect();
            let mut stack = vec![vec![]];

            while i < l {
                match v[i] {
                    ':' => {
                        i += 1;
                        if i < l && v[i] == ':' {
                            i += 1;
                            stack.push(vec![]);
                        }
                    },
                    '{' => {stack.push(vec![]);},
                    '}' => {stack.pop().unwrap();},
                    ',' => {
                        let k = stack.last_mut().unwrap();
                        k.push(String::new());
                    },
                    ';' => break,
                    _ => {
                        if let Some(k) = stack.last_mut() {
                            if let Some(last_str_in_scope) = k.last_mut() {
                                last_str_in_scope.push(v[i]);
                            }
                        }
                    }
                }
                i += 1;
            }

            let imported: Vec<String> = stack.iter()
                .flat_map(|level| level.iter())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            let is_used = imported.iter().any(|symbol| {
                contents.lines().enumerate().any(|(line_idx, line)| {
                    line_idx != idx && line.contains(symbol)
                })
            });

            if !is_used {
                unused_imports.push(idx);
            }
         }
         unused_imports.reverse();

         let lines: Vec<&str> = contents.lines().collect();
         let filtered_lines: Vec<&str> = lines.iter().enumerate().filter(|(i, _)| !unused_imports.contains(i)).map(|(_, &line)| line).collect();
         
         let res = filtered_lines.join("\n");
         std::fs::write(fpath, res).unwrap();
    }, 
    "add_doc_str" | "doc_str" | "add_doc" | "add_docs" => {
        let fpath = cwd.join(Path::new(&args[1]));
        let mut contents = fs::read_to_string(&fpath).unwrap();
        let (fxn_doc,struct_doc) = (
            "
/// {FXN_NAME}
/// 
/// Summary
///
/// Description.
///
/// # Arguments
///
/// {ARGS}
///
/// # Returns
/// 
/// {RETURNS}
///
/// Description of the return value.
///
/// # Errors
///
/// Description of possible errors function can throw
///
/// # Panics
///
/// Description of possible conditions for panics
///
/// # Examples
///
/// ```rust
/// 
/// ```",
"
/// {STRUCT_NAME}
/// 
/// Summary
///
/// Description
///
/// Fields 
/// 
/// {FIELDS}
/// 
/// # Examples
///
/// ```rust
/// 
/// ```
"
);

        let fxn_regex = Regex::new(r"(pub\s+)?fn\s+(\w+)\s*\(([^)]*)\)\s*(->\s*[^{;]+)?").unwrap();
        let strct_regex = Regex::new(r"(pub\s+)?struct\s+(\w+)\s*\{([^}]*)\}").unwrap();
        let fields_regex = Regex::new(r"pub\s+(\w+)\s*:\s*([^,]+)").unwrap();
        let args_regex = Regex::new(r"(\w+)\s*:\s*([^,]+)").unwrap();
        
        let lines: Vec<String> = contents.lines().map(|l| l.to_string()).collect();
        let mut insertions = vec![];

        for (idx, line) in lines.iter().enumerate() {
            if let Some(caps) = fxn_regex.captures(line) {
                let fn_name = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                let args_str = caps.get(3).map(|m| m.as_str()).unwrap_or("");
                let ret_str = caps.get(4).map(|m| m.as_str().trim_start_matches("->").trim()).unwrap_or("()");

                let args: Vec<String> = args_regex.captures_iter(args_str)
                    .map(|c| {
                        let name = c.get(1).map(|m| m.as_str()).unwrap_or("");
                        let typ = c.get(2).map(|m| m.as_str().trim()).unwrap_or("");
                        format!("* `{}` - `{}` -> Description", name, typ)
                    })
                    .collect();
                let args_formatted = if args.is_empty() { "None".to_string() } else { args.join("\n/// ") };

                let doc = fxn_doc
                    .replace("{FXN_NAME}", fn_name)
                    .replace("{ARGS}", &args_formatted)
                    .replace("{RETURNS}", ret_str);

                insertions.push((idx, doc));
            }
        }

        for (idx, line) in lines.iter().enumerate() {
            if let Some(caps) = strct_regex.captures(line) {
                let struct_name = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                let fields_str = caps.get(3).map(|m| m.as_str()).unwrap_or("");
                let fields: Vec<String> = fields_regex.captures_iter(fields_str)
                    .map(|c| {
                        let name = c.get(1).map(|m| m.as_str()).unwrap_or("");
                        let typ = c.get(2).map(|m| m.as_str().trim()).unwrap_or("");
                        format!("* `{}` - `{}` -> Description", name, typ)
                    })
                    .collect();
                let fields_formatted = if fields.is_empty() { "None".to_string() } else { fields.join("\n/// ") };
                let doc = struct_doc
                    .replace("{STRUCT_NAME}", struct_name)
                    .replace("{FIELDS}", &fields_formatted);

                insertions.push((idx, doc));
            }
        }

        insertions.sort_by(|a, b| b.0.cmp(&a.0));
        for (idx, doc) in insertions {
            let byte_offset = lines[..idx].iter().map(|l| l.len() + 1).sum::<usize>();
            contents.insert_str(byte_offset, &doc);
        }

        std::fs::write(fpath, contents).unwrap();
    }

_ => {}
};

}
