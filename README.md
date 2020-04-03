# gsub

A CLI tool for string substitutions in a directory.

## Why?

Have you ever used `grep`, wanting to replace the pattern you were searching for? Me too!

Tools like `sed` exist, but can be a bit clunky when used in conjunction with one of the many `*grep` variations.

For example, this is how the [ripgrep faq](https://github.com/BurntSushi/ripgrep/blob/master/FAQ.md#how-can-i-search-and-replace-with-ripgrep) recommends using `rg` and `sed` to search and replace.

```
rg foo --files-with-matches | xargs sed -i 's/foo/bar/g'
```

That's a very verbose command. And for _really_ large directories, can take 30+ minutes, or crash my terminal on my 5+ year old Mac Book 😢. I'm very impatient, and hate googling syntax for a specific commands. So `gsub` was born.

With gsub, the above command becomes

```
gsub "foo" "bar"
```

And for _really_ large directories, only takes 20-30 seconds (instead of minutes) without crashing my ancient laptop. 😊

Also, I wanted to learn Rust, and this seemed like a good starter project.

## How?

Here's the help page. Hopefully it's self explanatory enough.

```
gsub 0.1.0
Regex substitution for files and directories

USAGE:
    gsub [FLAGS] [OPTIONS] <pattern> <replacement> [--] [files]...

FLAGS:
    -c, --copy-on-write    Copies files instead of editing them
    -d, --dry-run          
        --help             Prints help information
    -h, --hidden           Do not skip hidden files and directories
    -V, --version          Prints version information
    -v, --verbose          

OPTIONS:
    -e, --except <files-to-skip>...                 Files/Directories to skip
    -m, --skip-files-larger-than <max-file-size>    Skip files larger than the given number of bytes [default: 4194304]

ARGS:
    <pattern>        The pattern you want to replace
    <replacement>    String for replacement
    <files>...       List of files/directories you want to gsub on. If unspecified, uses the current directory
```
