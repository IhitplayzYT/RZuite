use std::path::Path;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let cwd = std::env::current_dir().unwrap();
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
}
