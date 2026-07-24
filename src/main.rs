use std::{fs, path::Path};

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
         let mut contents = fs::read_to_string(&fpath).unwrap();
         let mut imports = vec![];
         contents.split("\n").enumerate().for_each(|x| if x.1.starts_with("use ") {imports.push(x);});
         
         for (idx,v) in imports{
            let l = v.len();
            let mut i = 0;
            let v: Vec<char> = v.chars().collect();
            let mut stack = vec![];

            while i < l{
                match v[i]{
                    ':' => {i+=1;stack.push(vec![]);},
                    '{' => {stack.push(vec![]);},
                    '}' => {stack.pop().unwrap();},
                    ',' => {
                        let k = stack.last_mut().unwrap();
                        k.push(String::new());
                    },
                    _ => {
                        let k = stack.last_mut().unwrap();
                        let last_str_in_scope = k.last_mut().unwrap();
                        last_str_in_scope.push(v[i]);
                    }
                }
                i+=1;
            }



         }

    }

_ => {}
};

}
