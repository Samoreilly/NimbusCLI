//NOTES ARE IN MULTIPURPOSECLI.TXT IN HOME DIRECTORY

use std::cmp::Ordering;
use std::collections::HashSet;
use std::io;
use walkdir::WalkDir;
use std::env;
use std::env::current_dir;
use std::hash::Hash;
use std::collections::BinaryHeap;

#[derive(Eq, PartialEq)]
struct MatchItem {
    substring_len: usize,
    subsequence_len: usize,
    file_name: String,
    path: String,
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

fn find_file(input: &String){
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

fn fuzzy_finder(input: &str) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut heap = BinaryHeap::new();

    let home_dir = dirs::home_dir().expect("Could not find home directory");

    for entry in WalkDir::new(home_dir).into_iter().filter_map(|e| e.ok()) {
        //compare every entry with a substring of the input
        let path = entry.path();
        if let Some(file_name) = path.file_name(){
            if let Some(file_name_str) = file_name.to_str() {


                let file_name_str = entry.file_name().to_str().unwrap();
                let subsequence_len = get_subsequences(input, file_name_str);
                let substring_len = get_substring(input, file_name_str);

                if seen.insert(file_name_str.to_string()) {//this returns true if the item was not in the set
                    heap.push(MatchItem {
                        substring_len,
                        subsequence_len,
                        file_name: file_name_str.to_string(),
                        path: path.display().to_string(),
                    });
                }

                //gets the exact result - no others

                // if file_name_str == input {
                //     seen.clear();
                //     heap.clear();
                //     seen.insert(file_name_str.to_string());
                //     println!("Found: {}", path.display());
                //     break;
                // }


            }
        }
    }

    let mut top_matches = Vec::new();
    while let Some(item) = heap.pop() {
        top_matches.push(item.path);//shows path
        if top_matches.len() >= 25 {
            break;
        }
    }

    println!("{:?}", top_matches);
    top_matches

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

fn main() {
    println!("Enter a file you are looking for");
    let mut input = String::new();

    io::stdin().read_line(&mut input).expect("Failed to read line");
    let input = input.trim().to_string();

    fuzzy_finder(&input);
    // find_file(&input);
    // println!("Finished");

}

