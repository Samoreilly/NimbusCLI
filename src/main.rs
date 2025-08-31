//NOTES ARE IN MULTIPURPOSECLI.TXT IN HOME DIRECTORY

mod cache;

use std::ascii::AsciiExt;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use clap::{Parser, ArgAction};
use tokio::fs::File;
use walkdir::WalkDir;
use crate::cache::Cache;
use zip::{write, CompressionMethod, ZipWriter};
use zip::write::FileOptions;
//TO TEST OUT IN TERMINAL

//SINGLE ARGUMENT TO SEARCH FOR A FILE
//cargo run -- --file-name "multipurposecli.txt"

//TWO ARGUMENTS TO SEARCH FOR A FILE IN A SPECIFIC FOLDER
//cargo run -- --folder-name "clionprojects" --file-name "cargo"

//THREE ARGUMENTS TO SEARCH FOR A FILE IN A SPECIFIC FOLDER WITH A SPECIFIC EXTENSION
//cargo run -- --folder-name "CLionProjects" --file-name "cargo" --extension ".txt"

//cargo run -- --file-name "m" --extension ".txt"

//cargo run -- --folder-name "CLionProjects" --extension ".txt"

//cargo run -- --folder-name "CLionProjects" --extension ".txt" --content "a"

//cargo run -- --file-name "multipurposecli.txt" --ignore ".txt"

//cargo run -- --file-name "multipurposecli.txt" --max "100kb"
//cargo run -- --file-name "multipurposecli.txt" --min "870kb"

//cargo run -- --file-name "multipurposecli.txt" --rename "mlpcli.txt"



#[derive(Eq, PartialEq)]
struct MatchItem {
    substring_len: usize,
    subsequence_len: usize,
    file_name: String,
    path: String,
}
#[derive(Parser, Debug)]
#[command(name = "multipurposecli")]
#[command(author = "Sam O'Reilly")]
#[command(version = "1.0")]
#[command(about = "Search files with fuzzy matching")]//CLAP CONVERTS SNAKE CASE TO KEBAB CASE so use dash in arguments
pub struct CliArgs {
    #[arg(short = 'd', long, default_value = "/home")]// default to current directory
    folder_name: PathBuf,
    #[arg(short = 'f', long, default_value = "")]
    file_name: String,
    #[arg(short = 'e', long)]
    extension: Option<String>,
    #[arg(short = 'l', long, default_value = "10")]
    limit: Option<usize>,
    #[arg(short = 'c', long)]
    content: Option<String>,
    #[arg(short = 'i', long)]
    ignore: Option<String>,
    #[arg(short = 'm', long)]
    max: Option<String>,
    #[arg(short = 'n', long)]
    min: Option<String>,
    #[arg(short = 'r', long)]
    rename: Option<String>,
    #[arg(short = 'z', long, action = ArgAction::SetTrue)]
    zip: bool,
    #[arg(short = 'p', long, action = ArgAction::SetTrue)]
    preview: bool


}
#[derive(Parser, Debug)]
#[command(name = "multipurposecli")]
#[command(author = "Sam O'Reilly")]
#[command(version = "1.0")]
#[command(about = "Search files with fuzzy matching")]
pub struct FzFinder{
    #[arg(short, long)]
    pub folder_name: PathBuf,
    pub file_name: String,
    pub file_ext: Option<String>,
    pub limit: Option<usize>,
    pub content: Option<String>,
    pub ignore: Option<String>,
    pub max: Option<String>,
    pub min: Option<String>,
    pub rename: Option<String>,
    pub zip: bool,
    pub preview: bool
}

impl From<CliArgs> for FzFinder{
    fn from(cli: CliArgs) -> Self {
        Self {
            folder_name: cli.folder_name,
            file_name: cli.file_name,
            file_ext: cli.extension,
            limit: cli.limit,
            content: cli.content,
            ignore: cli.ignore,
            max: cli.max,
            min: cli.min,
            rename: cli.rename,
            zip: cli.zip,
            preview: cli.preview
        }
    }
}
impl FzFinder{
    pub fn fuzzy_finder(&self, cache: Arc<Cache<String, String>>) -> Vec<String> {

        let mut bool_match = false;
        let mut seen: HashSet<String> = HashSet::new();
        let mut heap = BinaryHeap::new();

        if self.file_name.is_empty() && self.preview {
            println!("\n\x1b[1;91mWarning: You didn’t specify a file name to search\x1b[0m");

            println!("\x1b[31;1mWarning: You tried to preview a folder\x1b[0m");

            println!("\n\x1b[33m--help for more information");
            return Vec::new();
        }

        //INVALID ARGUMENTS
        if let Some(ignore) = &self.ignore {
            if !self.file_name.is_empty(){
                if self.file_name.trim().ends_with(ignore) {
                    if self.file_ext.is_some() {
                        println!(
                            "\n\x1b[35mYou cannot ignore the file extension you are looking for\x1b[0m\n\
                     \x1b[31mUse --help for more information\x1b[0m"
                        );
                    } else {
                        println!(
                            "\n\x1b[35mNote: You cannot search for a file that matches the ignore pattern\x1b[0m\n\
                     \x1b[31mUse --help for more information\x1b[0m"
                        );
                    }
                    return Vec::new();
                }
            }
        }

        let folder_only = self.file_name.is_empty()
            && self.file_ext.as_ref().map_or(true, |s| s.is_empty())
            && self.folder_name != PathBuf::from("/home");

        // let extension_only = self.file_name.is_empty()
        //     && self.file_ext.as_ref().map_or(false, |s| !s.is_empty())
        //     && self.folder_name == PathBuf::from("/home");

        let mut folder_path: PathBuf = if self.folder_name == PathBuf::from("/home") {
            self.folder_name.clone()
        }else{
            find_folder(&self.folder_name.to_string_lossy())
        };
        //cargo run -- --extension "d"
        //invalid command for e.g.
        if folder_only  && !self.zip  && !self.preview {
            println!("\n\x1b[35mYou can use the following valid commands:\x1b[0m");
            let valid_set = valid_commands_set();
            for cmd in &valid_set {
                println!("{} \n", cmd);
            }
            return Vec::new();
        }
        //PREVIEW IMPLEMENTATION


        //PREVIEW IMPLEMENTATION

        //CACHE IMPLEMENTATION
        if !self.file_name.is_empty() && self.rename.is_none() && !self.preview && self.content.is_none() {
            if let Some(cached_path) = cache.get_value(&self.file_name) {
                if self.folder_name == PathBuf::from("/home") && self.file_ext.is_none() {
                    println!("{}", cached_path);
                    return vec![cached_path];
                }
            } else {
                cache.insert(self.file_name.clone(), folder_path.to_string_lossy().to_string());
                cache.write_to_file("dashmap.txt");
            }
        }


        // if !self.file_name.is_empty()
        //     && let Some(ignore) = &self.ignore
        //     && !self.file_name.ends_with(ignore)
        //     && let Some(ext) = &self.file_ext
        //     && ignore.as_str() != ext.as_str()
        // {
        //     if let Some(cached_path) = cache.get_value(&self.file_name) {
        //        // folder_path = PathBuf::from(cached_path);
        //         if self.folder_name == PathBuf::from("/home")  && self.file_ext.is_none(){
        //             println!("{}", cached_path.clone().to_string());
        //             return vec![cached_path];
        //         }
        //         // println!("Found a cached path")
        //     } else {
        //         cache.insert(self.file_name.clone(), folder_path.to_string_lossy().to_string());
        //         cache.clone().write_to_file("dashmap.txt");
        //         // println!("Cached: {} {}", &self.file_name, folder_path.display());
        //     }
        // }

        //CACHE IMPLEMENTATION
        let _home_dir = dirs::home_dir().expect("Could not find home directory");
        if self.zip {
            let zip_path = folder_path.with_extension("zip");
            match zip_folder(&folder_path, &zip_path) {
                Ok(_) => {
                    println!("Successfully zipped folder: {} -> {}", folder_path.display(), zip_path.display());
                    return vec![zip_path.to_string_lossy().to_string()];
                }
                Err(e) => {
                    eprintln!("Failed to zip folder: {}", e);
                    return Vec::new();
                }
            }
        }
        let max_files = 10000000;
        let mut file_count = 0;

        for entry in WalkDir::new(folder_path).into_iter().filter_map(|e| e.ok()) {
            file_count += 1;
            if file_count >= max_files {
                println!("Reached maximum number of files to search");
                return Vec::new();
            }
            //compare every entry with a substring of the input
            let path = entry.path();
            if self.content.is_some(){
                let ext: Option<&str> = self.file_ext.as_deref();
                let result = find_content(self.content.as_ref().unwrap(), &path, ext);

                for item in result {
                    println!("{}", item);
                }

               return Vec::new();
            }

            if path.is_file() && self.preview {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if self.file_name.eq_ignore_ascii_case(file_name) {
                        if let Some(ext) = &self.file_ext {
                            if file_name.to_lowercase().ends_with(&ext.to_lowercase()) {
                                if let Ok(contents) = fs::read_to_string(path) {
                                    println!("\n{}", contents);
                                }
                                return Vec::new();
                            }
                        } else {
                            if let Ok(contents) = fs::read_to_string(path) {
                                println!("\n{}", contents);
                            }
                            return Vec::new();
                        }
                    }
                }
            }

            //ZIP FILE

            if path.is_file() && path.file_name().unwrap().to_string_lossy() == self.file_name && self.rename.is_some() && !self.file_name.is_empty(){
                let new_path = path.with_file_name(self.rename.clone().unwrap());
                if let Err(err) = fs::rename(&path, &new_path) {
                    eprintln!("\n\x1b[31mFailed to rename {}: {}\x1b[0m", path.display(), err);
                }
                println!(
                    "\n\x1b[32mRenamed {} to {}\x1b[0m", path.display(), new_path.display());
                return Vec::new();
            }

            if let Some(max) = &self.max {
                let max = get_memory_usage(&max);//gets memory usage in bytes
                if path.is_file(){
                    if let Ok(metadata) = path.metadata() {
                        if metadata.size() > max {
                            continue;
                        }
                    }
                }
            }
            //changed from else if to if so both can run
            if let Some(min) = &self.min {
                let min = get_memory_usage(&min);//gets memory usage in bytes
                if path.is_file(){
                    if let Ok(metadata) = path.metadata() {
                        if metadata.size() < min {
                            continue;
                        }
                    }
                }
            }


            if let Some(file_name) = path.file_name(){
                if let Some(file_name_str) = file_name.to_str(){


                    //IF CONTENT ARGUMENT IS PRESENT, CHECK IF CONTENT IS IN FILE
                    // cargo run -- --folder-name "CLionProjects" --extension ".txt" --content "a"

                    //CHECKING FOR CONTENT
                    let mut count = 0;
                    if let Some(content) = &self.content {
                        if path.is_file() {
                            match fs::read_to_string(path) {
                                Ok(contents) => {
                                    for (index, line) in contents.lines().enumerate() {
                                        if line.contains(content){
                                            if(count >= 15){
                                                break;
                                            }
                                            count += 1;
                                            println!(
                                                "Found in file {} on line {}: {} \n",
                                                path.display(),
                                                index + 1,
                                                line
                                            );
                                        }
                                    }
                                }
                                Err(_err) => {

                                }
                            }
                            return Vec::new();
                        } else {
                            // println!("Skipping directory: {}", path.display());
                        }

                    }


                    // let file_name_str = entry.file_name().to_str().unwrap();

                    let subsequence_len = get_subsequences(&self.file_name, file_name_str, &self.file_ext);
                    let substring_len = get_substring(&self.file_name, file_name_str, &self.file_ext);

                    if self.file_name.is_empty(){
                        if let (Some(ext), Some(_folder_name)) = (&self.file_ext, &self.folder_name.to_str()) {
                            if file_name_str.to_lowercase().ends_with(&ext.to_lowercase()) {
                                heap.push(MatchItem {
                                    substring_len,
                                    subsequence_len,
                                    file_name: file_name_str.to_string(),
                                    path: path.display().to_string(),
                                });
                            }
                        }
                    } else {
                        if seen.insert(file_name_str.to_string()) {
                            heap.push(MatchItem {
                                substring_len,
                                subsequence_len,
                                file_name: file_name_str.to_string(),
                                path: path.display().to_string(),
                            });
                        }
                    }


                    //gets the exact file path if match is found
                    if file_name_str.eq_ignore_ascii_case(&self.file_name) {
                        if let Some(ext) = &self.file_ext {
                            // Check if the file ends with the extension
                            if file_name_str.to_lowercase().ends_with(&ext.to_lowercase()) {
                                seen.clear();
                                heap.clear();
                                seen.insert(path.display().to_string());
                                // println!("Found: {}", path.display());
                                bool_match = true;
                                break;
                            }
                        } else {
                            // No extension specified, just match by name
                            seen.clear();
                            heap.clear();
                            seen.insert(path.display().to_string());
                            // println!("Found: {}", path.display());
                            bool_match = true;
                            break;
                        }
                    }
                }
            }

        }
        if bool_match{
            let top_matches: Vec<String> = seen.into_iter().collect();
            let formatted = top_matches.join("\n\n");
            println!("{}", formatted);
            return top_matches;
        }

        let mut top_matches = Vec::new();
        while let Some(item) = heap.pop() {
            top_matches.push(item.path);//shows path
            if top_matches.len() >= self.limit.unwrap() {
                break;
            }
        }

        let formatted = top_matches.join("\n\n");

        println!("{}", formatted);
        top_matches

    }
}
fn find_content(content: &str, folder_path: &Path, extension: Option<&str>) -> Vec<String> {
    let mut top_matches = Vec::new();

    for entry in WalkDir::new(folder_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        if let Some(ext_filter) = extension {    //if extension exists check if extension matches
            let ext_filter = ext_filter.trim_start_matches('.');
            let file_ext = path.extension().and_then(|e| e.to_str());

            if file_ext.map_or(true, |e| !e.ends_with(ext_filter)) {
                continue;
            }
        }


        if let Ok(contents) = fs::read_to_string(path) {
            for (index, line) in contents.lines().enumerate() {
                if line.contains(content) {
                    let result = format!("{}:{}\n{}\n", path.display(), index + 1, line);
                    top_matches.push(result);
                    break;
                }
            }
        }
    }

    top_matches
}
fn zip_folder(folder_path: &Path, zip_output_path: &Path) -> zip::result::ZipResult<()> {
    println!("Zipping folder: {}", folder_path.display());
    let zip_file = fs::File::create(zip_output_path)?;
    let mut zip = ZipWriter::new(zip_file);

    let options = FileOptions::<()>::default().compression_method(CompressionMethod::Deflated);

    let folder_len = folder_path.components().count();

    for entry in WalkDir::new(folder_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path.components().skip(folder_len).collect::<std::path::PathBuf>();

        if path.is_file() {
            zip.start_file(name.to_string_lossy(), options)?;
            let mut f = fs::File::open(path)?;
            std::io::copy(&mut f, &mut zip)?;
        } else if path.is_dir() && !name.as_os_str().is_empty() {
            zip.add_directory(name.to_string_lossy(), options)?;
        }
    }
    zip.finish()?;
    Ok(())
}

fn get_memory_usage(amount: &str) -> u64{
    let mut size: u64  = 0;
    let mut unit= "b".to_string();

    for(index, val) in amount.chars().enumerate() {
        if val.is_alphabetic() {
            if let Ok(parsed_size) = amount[0..index].parse() {
                size = parsed_size;
            } else {
                eprintln!("Warning: Invalid size format '{}'", amount);
                return 0; // Or handle error appropriately
            }
            unit = amount[index..].to_lowercase();
            break;
        }
    }
    match unit.as_str() {
        "b" => size,
        "kb" => size.saturating_mul(1024),
        "mb" => size.saturating_mul(1024 * 1024),
        "gb" => size.saturating_mul(1024 * 1024 * 1024),
        "tb" => size.saturating_mul(1024 * 1024 * 1024 * 1024),
        _ => size,
    }
}
fn find_folder(folder_name: &str) -> PathBuf {// find folder that will be used to find the folder in fuzzy finder with folder and file arguments

    let home_dir = dirs::home_dir().expect("Could not find home directory");
    for entry in WalkDir::new(home_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        if let Some(name) = entry.file_name().to_str() {
            if name.eq_ignore_ascii_case(folder_name.to_lowercase().as_str()) {
                return entry.path().to_path_buf();
            }
        }
    }
    let home_path: PathBuf = PathBuf::from("/home");
    // eprintln!("Folder not found, using default home directory");
    home_path

}

fn get_substring(input: &str, entry: &str, extension: &Option<String>) -> usize {

    if input.is_empty() {
        return 0;
    }
    let mut longest = 0;//keep track of longest substring
    let input_chars: Vec<char> = input.chars().collect();
    let entry_str = entry;

    if let Some(ext) = extension {//if extension exists, check if the entry ends with the extension
        if !entry.to_lowercase().ends_with(&ext.to_lowercase()){
            return 0;
        }
    }

    for start in 0..input_chars.len() {
        for end in start + 1..= input_chars.len() {
            let slice: String = input_chars[start..end].iter().collect();

            if entry_str.to_lowercase().contains(&slice.to_lowercase()) {
                if slice.len() > longest {
                    longest = slice.len();
                }
            }
        }
    }
    longest
}
fn get_subsequences(input: &str, entry: &str, extension: &Option<String>) -> usize {

    if input.is_empty() {
        return 0;
    }

    if let Some(ext) = extension {//if extension exists, check if the entry ends with the extension
        if !entry.to_lowercase().ends_with(ext.to_lowercase().as_str()){
            return 0;
        }
    }
    let mut entry_text = entry.chars();
    input.chars().filter(|&c| entry_text.any(|x| x == c)).count()
}

impl MatchItem {
    fn calculate_score(&self) -> f64{


        let mut score: f64 =  (self.substring_len * 10 + self.subsequence_len) as f64;

        score += self.folder_names();
        score += self.path_depth();

        score
    }
    fn path_depth(&self) -> f64{
        let mut score: f64 = 0.0;
        let mut depth = 0;
        for c in self.path.chars() {
            if c == '/' {
                depth += 1;
            }
        }
        if depth < 3{
            score += 15.0;
        }else if depth < 5{
            score += 10.0;
        }else if depth < 8{
            score += 5.0;
        }
        score

    }
    fn folder_names(&self) -> f64{
        let mut score: f64 = 0.0;
        if(self.path.contains(".txt") || self.path.contains("/src") || self.path.contains("/bin") || self.path.contains("/lib")|| self.path.contains("/docs") || self.path.contains("/public") || self.path.contains("/app")|| self.path.contains("/core")){
            score += 20.0;
        }
        if self.path.contains("node_modules") || self.path.contains("/target/") {
            score -= 20.0;
        }

        score
    }
}
// custom ordering: max-heap
impl Ord for MatchItem {
    fn cmp(&self, other: &Self) -> Ordering { // Ordering goes as substring_len -> subsequence_len -> file_name
        self.calculate_score()
            .partial_cmp(&other.calculate_score())
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for MatchItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}



fn valid_commands_set() -> HashSet<&'static str> {
    [
        // Single argument
        "\x1b[32m--file-name\x1b[0m \x1b[37m<example-file>\x1b[0m",
        "\x1b[36m--folder-name\x1b[0m \x1b[37m<example-dir-name>\x1b[0m",

        // Two arguments
        "\x1b[36m--folder-name\x1b[0m \x1b[37m<example-dir-name>\x1b[0m \x1b[32m--file-name\x1b[0m \x1b[37m<example-file>\x1b[0m",
        //...
        "\x1b[32m--file-name\x1b[0m \x1b[37m<example-file>\x1b[0m \x1b[31m--extension\x1b[0m \x1b[33m<.sh | .png | .txt>\x1b[0m",

    //...
        "\x1b[36m--folder-name\x1b[0m \x1b[37m<example-dir-name>\x1b[0m \x1b[31m--extension\x1b[0m \x1b[33m<.sh | .png | .txt>\x1b[0m",

    //...
        // Three arguments
        "\x1b[36m--folder-name\x1b[0m \x1b[37m<example-dir-name>\x1b[0m \
\x1b[32m--file-name\x1b[0m \x1b[37m<example-file>\x1b[0m \
\x1b[31m--extension\x1b[0m \x1b[33m<.sh | .png | .txt>\x1b[0m"

    ]
        .iter()
        .cloned()
        .collect()
}

#[tokio::main]
async fn main() {
    let cache = Arc::new(crate::cache::Cache::<String, String>::new());//arcs allows cache to be shared between threads
    cache.read_from_file("dashmap.txt");
    // cache.clone().clean_lfu();

    let cache_clone = cache.clone();
    let cli = CliArgs::parse();
    let fz_finder: FzFinder = cli.into();

    fz_finder.fuzzy_finder(cache_clone.clone());

}
