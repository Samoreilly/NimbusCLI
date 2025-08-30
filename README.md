## NimbusCLI - Smart File Finder 🔍 FOR LINUX/DEBIAN ONLY

A lightning-fast command line tool that helps you find files when you can't remember exact names.
Uses fuzzy matching to understand what you're looking for.


## Download and install in one command
```
curl -LO https://github.com/Samoreilly/NimbusCLI/releases/download/cli/nimbuscli && chmod +x nimbuscli && sudo mv nimbuscli /usr/local/bin/
```

## Verify it works
```
nimbuscli --help
```


### 1. Find files when you remember part of the name
```
nimbuscli --file-name "config"
```
Finds: config.toml, configuration.yaml, my_config.txt, etc.

### 2. Search for specific file types
```
nimbuscli --extension ".txt" --file-name "readme"
```
Finds all text files containing "readme"

### 3. Find files by size

Find large log files

```
nimbuscli --extension ".log" --min "10MB"
```

Finds small config files  
```
nimbuscli --max "1KB" --file-name "config"
```

### 4. Search inside file contents

Find files containing "TODO" comments
```
nimbuscli --content "TODO" --extension ".rs"
```

## 5. Advanced fuzzy search

### Finds "multipurposecli.txt" even if you type it wrong
```
nimbuscli --file-name "mltpurpose" --extension ".txt"
```

Why You'll Love It

    Understands typos - finds files even if you spell names wrong

    Blazing fast - written in Rust for instant results

    Smart ranking - shows most relevant files first

    No setup - works immediately after download

Need Help?
```
## nimbuscli --help  # Show all options and examples
```
