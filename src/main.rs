use clap::Parser;
use serde::{Serialize, Deserialize};
use std::{fs, io::BufRead};
use std::path::Path;

#[derive(Parser, Debug)]
#[command(name="findId",author="szj",version ="1.0", about="find zh ID for chromium", long_about=None)]
struct Args {
    #[arg(short, long)]
    pub search: Option<String>,
    #[arg(short, long)]
    pub make: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FileCategories {
    zh_cn_files: Vec<String>,
    en_us_files: Vec<String>,
    en_gb_files: Vec<String>,
    grd_files: Vec<String>,
    grdp_files: Vec<String>,
}

fn extract_id(line: &str) -> Option<&str> {
    let id_start = line.find("id=\"")?;
    let id_start = id_start + 4; // Skip past 'id="'
    let id_end = line[id_start..].find('"')? + id_start;
    Some(&line[id_start..id_end])
}
fn extract_message(line: &str) -> Option<&str> {
    let message_start = line.find('>')?;
    let message_start = message_start + 1;
    let message_end = line[message_start..].find('<')? + message_start;
    Some(&line[message_start..message_end])
}

fn visit_dirs(
    dir: &Path,
    zh_cn_files: &mut Vec<String>,
    en_us_files: &mut Vec<String>,
    en_gb_files: &mut Vec<String>,
    grd_files: &mut Vec<String>,
    grdp_files: &mut Vec<String>,
) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, zh_cn_files, en_us_files, en_gb_files, grd_files, grdp_files)?;
            } else {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                match file_name {
                    name if name.ends_with("zh-CN.xtb") => zh_cn_files.push(path.to_str().unwrap().to_string()),
                    name if name.ends_with("en-US.xtb") => en_us_files.push(path.to_str().unwrap().to_string()),
                    name if name.ends_with("en-GB.xtb") => en_gb_files.push(path.to_str().unwrap().to_string()),
                    name if name.ends_with(".grd") => grd_files.push(path.to_str().unwrap().to_string()),
                    name if name.ends_with(".grdp") => grdp_files.push(path.to_str().unwrap().to_string()),
                    _ => (),
                }
            }
        }
    }
    Ok(())
}

fn main() {
    let args = Args::parse();
    println!("{:?}", args);
    
    let search = args.search.unwrap();

    let mut ids = Vec::new();
    let mut messages = Vec::new();

    //let current_exe = std::env::current_dir().unwrap();
    let current_exe = Path::new(r"E:\snow\snow_browser");
    let categories_file = current_exe.join("find-id-data.json");

    let categories = if categories_file.exists() {
        let file = fs::File::open(categories_file).unwrap();
        serde_json::from_reader(file).unwrap()
    } else {
        let mut zh_cn_files = Vec::new();
        let mut en_us_files = Vec::new();
        let mut en_gb_files = Vec::new();
        let mut grd_files = Vec::new();
        let mut grdp_files = Vec::new();

        visit_dirs(
            current_exe,
            &mut zh_cn_files,
            &mut en_us_files,
            &mut en_gb_files,
            &mut grd_files,
            &mut grdp_files,
        )
        .unwrap();

        let categories = FileCategories {
            zh_cn_files,
            en_us_files,
            en_gb_files,
            grd_files,
            grdp_files,
        };
        let file = fs::File::create(categories_file).unwrap();
        serde_json::to_writer(file, &categories).unwrap();
        categories
    };

    for file in categories.zh_cn_files {
        let file_path = Path::new(&file);
        if !file_path.exists() {
            continue;
        }
        let file = std::fs::File::open(file_path).unwrap();
        let reader = std::io::BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap();
            if line.contains(&search) {
                if let Some(id) = extract_id(&line) {
                    println!("id: {}", id);
                    ids.push(id.to_string());
                }
            }
        }
    }

    let mut combined_files = categories.en_us_files;
    combined_files.extend(categories.en_gb_files);
    for file in combined_files {
        let file_path = Path::new(&file);
        if !file_path.exists() {
            continue;
        }
        let content = std::fs::read_to_string(file_path).unwrap();
        let translations = content.split("<translation");
        let filtered_items: Vec<_> = translations.filter(|item| {
            ids.iter().any(|id| item.contains(id))
        }).collect();

        for item in filtered_items {
            if let Some(message) = extract_message(item) {
                println!("Found match: {}", message);
                messages.push(message.to_string());
            }
        }
    }

    let mut combined_grd_files = categories.grd_files;
    combined_grd_files.extend(categories.grdp_files);
    for file in combined_grd_files {
        let file_path = Path::new(&file);
        if !file_path.exists() {
            continue;
        }
   
        let content = fs::read_to_string(file_path).unwrap();
        let translations = content.split("<message");
        let filtered_items: Vec<_> = translations.filter(|item| {
            messages.iter().any(|message| item.contains(message))
        }).collect();

        for item in filtered_items {
            println!("Found match: {}", item);
        }
    }
}
