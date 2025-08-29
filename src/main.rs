//NOTES ARE IN MULTIPURPOSECLI.TXT IN HOME DIRECTORY

mod cache;

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use clap::{Parser};
use walkdir::WalkDir;
use crate::cache::Cache;
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
}
impl From<CliArgs> for FzFinder{
    fn from(cli: CliArgs) -> Self {
        Self {
            folder_name: cli.folder_name,
            file_name: cli.file_name,
            file_ext: cli.extension,
            limit: cli.limit,
            content: cli.content,
        }
    }
}
impl FzFinder{
    pub fn fuzzy_finder(&self, cache: Arc<Cache<String, String>>) -> Vec<String> {


        let mut bool_match = false;
        let mut seen: HashSet<String> = HashSet::new();
        let mut heap = BinaryHeap::new();

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
        if folder_only {
            println!("\n\x1b[35mYou can use the following valid commands:\x1b[0m\n");
            let valid_set = valid_commands_set();
            for cmd in &valid_set {
                println!("{} \n", cmd);
            }
            return Vec::new();
        }
        //CACHE IMPLEMENTATION

        if !self.file_name.is_empty() {
            if let Some(cached_path) = cache.get_value(&self.file_name) {
               // folder_path = PathBuf::from(cached_path);
                if self.folder_name == PathBuf::from("/home")  && self.file_ext.is_none(){
                    println!("{}", cached_path.clone().to_string());
                    return vec![cached_path];
                }
                // println!("Found a cached path")
            } else {
                cache.insert(self.file_name.clone(), folder_path.to_string_lossy().to_string());
                cache.clone().write_to_file("dashmap.txt");
                // println!("Cached: {} {}", &self.file_name, folder_path.display());
            }
        }

        //CACHE IMPLEMENTATION
        let _home_dir = dirs::home_dir().expect("Could not find home directory");

        for entry in WalkDir::new(folder_path).into_iter().filter_map(|e| e.ok()) {

            //compare every entry with a substring of the input
            let path = entry.path();

            if let Some(file_name) = path.file_name(){
                if let Some(file_name_str) = file_name.to_str(){

                    //IF CONTENT ARGUMENT IS PRESENT, CHECK IF CONTENT IS IN FILE
                    // cargo run -- --folder-name "CLionProjects" --extension ".txt" --content "a"

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

                    if self.file_name.is_empty() {
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


                    //gets the exact result - no others

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
            return Vec::new();
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

// custom ordering: max-heap
impl Ord for MatchItem {
    fn cmp(&self, other: &Self) -> Ordering { // Ordering goes as substring_len -> subsequence_len -> file_name
        self.substring_len //this compares substring len with other if same, then compares subsequence len
            .cmp(&other.substring_len)
            .then(self.subsequence_len.cmp(&other.subsequence_len))
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
