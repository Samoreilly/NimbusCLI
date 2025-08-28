//NOTES ARE IN MULTIPURPOSECLI.TXT IN HOME DIRECTORY

use std::cmp::Ordering;
use std::collections::HashSet;
use walkdir::WalkDir;
use std::env::current_dir;
use std::collections::BinaryHeap;
use std::io;
use std::path::{PathBuf};
use clap::Parser;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::enable_raw_mode;
use ratatui::{DefaultTerminal, Frame, Terminal};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
//TO TEST OUT IN TERMINAL

//SINGLE ARGUMENT TO SEARCH FOR A FILE
//cargo run -- --file-name "multipurposecli.txt"

//TWO ARGUMENTS TO SEARCH FOR A FILE IN A SPECIFIC FOLDER
//cargo run -- --folder-name "clionprojects" --file-name "cargo"

//THREE ARGUMENTS TO SEARCH FOR A FILE IN A SPECIFIC FOLDER WITH A SPECIFIC EXTENSION
//cargo run -- --folder-name "CLionProjects" --file-name "cargo" --extension ".txt"

//cargo run -- --file-name "m" --extension ".txt"

//cargo run -- --folder-name "CLionProjects" --extension ".txt"

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
}
#[derive(Parser, Debug, Clone)]
#[command(name = "multipurposecli")]
#[command(author = "Sam O'Reilly")]
#[command(version = "1.0")]
#[command(about = "Search files with fuzzy matching")]
pub struct FzFinder{
    #[arg(short, long)]
    pub folder_name: PathBuf,
    pub file_name: String,
    pub file_ext: Option<String>,
}
impl From<CliArgs> for FzFinder{
    fn from(cli: CliArgs) -> Self {
        Self {
            folder_name: cli.folder_name,
            file_name: cli.file_name,
            file_ext: cli.extension,
        }
    }
}
impl FzFinder{
    pub fn fuzzy_finder(&self) -> Vec<String> {

        let mut bool_match = false;
        let mut seen: HashSet<String> = HashSet::new();
        let mut heap = BinaryHeap::new();

        let folder_path: PathBuf = if self.folder_name == PathBuf::from("/home") {
            self.folder_name.clone()
        }else{
            find_folder(&self.folder_name.to_string_lossy())
        };


        let _home_dir = dirs::home_dir().expect("Could not find home directory");

        for entry in WalkDir::new(folder_path).into_iter().filter_entry(|e| !is_hidden(e)).filter_map(|e| e.ok()) {

            //compare every entry with a substring of the input
            let path = entry.path();
            if let Some(file_name) = path.file_name(){
                if let Some(file_name_str) = file_name.to_str() {


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
fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
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
    eprintln!("Folder not found, using default home directory");
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
                    if file_name_str.to_lowercase() == input.to_lowercase() {
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

fn main()  {



    let cli = CliArgs::parse();
    let fz_finder: FzFinder = cli.into();


    parse_by_argument();

    run_tui(fz_finder);


    // fuzzy_finder(&input);//works -- COMMENTED OUT WHILE WORKING ON OTEHR FEATURES
    // find_file(&input);
    // println!("Finished");

}


fn run_tui(fz_finder: FzFinder) -> io::Result<()> {
    enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut input = String::new();
    let mut results: Vec<String> = fz_finder.fuzzy_finder(); // initial results

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
                .split(f.size());

            // Input box
            let input_box = Paragraph::new(Text::from(input.clone()))
                .block(Block::default().title("Search").borders(Borders::ALL));
            f.render_widget(input_box, chunks[0]);

            // Results list
            let list_items: Vec<ListItem> = results.iter().map(|i| ListItem::new(i.clone())).collect();
            let list = List::new(list_items)
                .block(Block::default().title("Matches").borders(Borders::ALL));
            f.render_widget(list, chunks[1]);
        })?;


        // Handle keyboard input
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char(c) => {
                    input.push(c);
                }
                KeyCode::Backspace => {
                    input.pop();

                }
                KeyCode::Enter => {
                    // Update search results when Enter is pressed
                    results = FzFinder { file_name: input.clone(), ..fz_finder.clone() }.fuzzy_finder();
                }
                KeyCode::Esc | KeyCode::Char('q') => break,
                _ => {}
            }
        }
    }

    Ok(())
}




