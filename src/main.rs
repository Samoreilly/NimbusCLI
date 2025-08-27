//NOTES ARE IN MULTIPURPOSECLI.TXT IN HOME DIRECTORY

use std::cmp::Ordering;
use std::collections::HashSet;
use walkdir::WalkDir;
use std::env::current_dir;
use std::collections::BinaryHeap;
use std::path::{Path, PathBuf};
use clap::Parser;
use std::env;

//cargo run -- --folder-name "clionprojects" --file-name "cargo"
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
    #[arg(short = 'f', long)]
    file_name: String,
    #[arg(short = 'e', long)]
    extension: Option<String>,
}
//cargo run -- --file-name "notes"
#[derive(Parser, Debug)]
#[command(name = "multipurposecli")]
#[command(author = "Sam O'Reilly")]
#[command(version = "1.0")]
#[command(about = "Search files with fuzzy matching")]
pub struct FzFinder{
    #[arg(short, long)]
    pub folder_name: PathBuf,
    pub file_name: String,
}
impl From<CliArgs> for FzFinder{
    fn from(cli: CliArgs) -> Self {
        Self {
            folder_name: cli.folder_name,
            file_name: cli.file_name,
        }
    }
}
impl FzFinder{
    pub fn fuzzy_finder(&self) -> Vec<String> {//cargo run -- --file-name "multipurposecli.txt"
        let mut bool_match = false;
        let mut seen: HashSet<String> = HashSet::new();
        let mut heap = BinaryHeap::new();

        let folder_path = find_folder(&self.folder_name.to_string_lossy());

        let _home_dir = dirs::home_dir().expect("Could not find home directory");

        for entry in WalkDir::new(&folder_path).into_iter().filter_map(|e| e.ok()) {

            //compare every entry with a substring of the input
            let path = entry.path();
            if let Some(file_name) = path.file_name(){
                if let Some(file_name_str) = file_name.to_str() {


                    // let file_name_str = entry.file_name().to_str().unwrap();
                    let subsequence_len = get_subsequences(&self.file_name, file_name_str);
                    let substring_len = get_substring(&self.file_name, file_name_str);

                    if seen.insert(file_name_str.to_string()) {//this returns true if the item was not in the set
                        heap.push(MatchItem {
                            substring_len,
                            subsequence_len,
                            file_name: file_name_str.to_string(),
                            path: path.display().to_string(),
                        });
                    }

                    //gets the exact result - no others

                    if file_name_str == &self.file_name {
                        seen.clear();
                        heap.clear();
                        seen.insert(path.display().to_string());
                        println!("Found: {}", path.display());
                        bool_match = true;
                        break;
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
            if top_matches.len() >= 10 {
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
            if name == folder_name {
                return entry.path().to_path_buf();
            }
        }
    }
    let home_path: PathBuf = PathBuf::from("/home");
    eprintln!("Folder not found, using default home directory");
    home_path

}

fn get_substring(input: &str, entry: &str) -> usize {

    let mut longest = 0;//keep track of longest substring
    let input_chars: Vec<char> = input.chars().collect();
    let entry_str = entry;

    for start in 0..input_chars.len() {
        for end in start + 1..= input_chars.len() {
            let slice: String = input_chars[start..end].iter().collect();
            if entry_str.contains(&slice) {
                if slice.len() > longest {
                    longest = slice.len();
                }
            }
        }
    }
    longest
}
fn get_subsequences(input: &str, entry: &str) -> usize{

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

fn _find_file(input: &String){
    println!("Looking for files in");

    let curr_dir = current_dir().expect("Could not find current directory");
    println!("Current Directory: {}", curr_dir.display());


    let home_dir = dirs::home_dir().expect("Could not find home directory");

    for entry in WalkDir::new(home_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();


        if path.is_file(){
            if let Some(file_name) = path.file_name(){
                if let Some(file_name_str) = file_name.to_str() {
                    if file_name_str == input {
                        println!("Found: {}", path.display());

                    }

                }
            }
        }
    }
    println!("Exiting File Finder");
}



fn parse_by_argument(){
}

fn main() {

    let cli = CliArgs::parse();
    let fz_finder: FzFinder = cli.into();
    fz_finder.fuzzy_finder();

    parse_by_argument()





    // fuzzy_finder(&input);//works -- COMMENTED OUT WHILE WORKING ON OTEHR FEATURES
    // find_file(&input);
    // println!("Finished");

}

