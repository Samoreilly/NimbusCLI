## NimbusCLI - Smart File Finder 🔍 FOR LINUX/DEBIAN ONLY

No more remembering exact names. Find files naturally with fuzzy search with convenience.


## Download and install in one command
```
curl -LO https://github.com/Samoreilly/NimbusCLI/releases/download/cli/nimbuscli
&& chmod +x nimbuscli && sudo mv nimbuscli /usr/local/bin/
```

## Verify it works
```
nimbuscli --help
```


### 1. Find files when you remember part of the name
* Finds: config.toml, configuration.yaml, my_config.txt, etc.
```
nimbuscli --file-name "config"
```

### 2. Search for specific file types
* Finds all text files containing "readme"
```
nimbuscli --extension ".txt" --file-name "readme"
```

### 3. Find files by size

* Find large log files

```
nimbuscli --extension ".log" --min "10MB"
```

* Finds small config files  
```
nimbuscli --max "1KB" --file-name "config"
```

### 4. Search inside file contents

* Find files containing "TODO" comments
```
nimbuscli --content "TODO" --extension ".rs"
```

## 5. Advanced fuzzy search

* Finds "multipurposecli.txt" even if you type it wrong
```
nimbuscli --file-name "mltpurpose" --extension ".txt"
```

# Why You'll Reach for NimbusCLI

It just works:

    ✅ No complex syntax to remember

    ✅ Finds files when you can't recall exact names

    ✅ Understands what you mean, not just what you type

    ✅ Works immediately after download

Perfect for when you:

    Can't remember if it's config.yaml or configuration.yml

    Need to find all large log files quickly

    Want to search inside files without grep complexity

    Prefer typing naturally over perfect spelling
    
Need Help?
```
nimbuscli --help  # Shows simple examples and options
```
