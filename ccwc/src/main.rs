use anyhow::{Context, Result};
use clap::Parser;

/// wc - word, line, character, and byte count
#[derive(Parser)]
struct Cli {
    /// The number of bytes in each input file
    #[arg(short = 'c')]
    bytes: bool,

    /// The number of lines in each input file
    #[arg(short)]
    lines: bool,

    /// The number of words in each input file
    #[arg(short)]
    words: bool,

    /// The number of characters in each input file
    #[arg(short = 'm')]
    chars: bool,

    /// The path to the file to read
    path: Option<std::path::PathBuf>,
}

struct Input {
    path: String,
    content: String,
}

fn get_input(option_path: Option<std::path::PathBuf>) -> Result<Input> {
    if option_path == None {
        let stdin = std::io::stdin();
        // TODO: Handle case when no input is passed in
        let content: Vec<String> = stdin.lines().map(|l| l.unwrap()).collect();
        Ok(
            Input {
                path: String::new(),
                content: content.join("\r\n") + "\r\n", // Newline at end of file gets stripped, so
                                                        // add it back in
            }
        )
    } else {
        let path = option_path.unwrap();
        Ok(
            Input {
                path: path.display().to_string(),
                content: std::fs::read_to_string(&path)
                    .with_context(|| format!("could not read file `{}`", path.display()))?,
            }
        )
    }
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let input = get_input(args.path).unwrap();

    let content = input.content;
    let display_path = input.path;

    if args.bytes {
        println!("    {} {}", content.len(), display_path);
    }  else if args.lines {
        println!("    {} {}", content.lines().count(), display_path);
    } else if args.words {
        let word_count = count_words(&content);
        println!("    {} {}", word_count, display_path);
    } else if args.chars {
        println!("    {} {}", content.chars().count(), display_path);
    } else {
        println!("    {} {} {} {}", content.lines().count(), count_words(&content), content.chars().count(), display_path);
    }

    Ok(())
}

fn count_words(content: &str) -> i32 {
    let mut total_word_count = 0;
    let mut word_char_count = 0;
    let mut non_word_char_count = 0;

    // Iterate over string char by char.
    // If we encounter a word char, mark that we're in a word.
    // If we encounter a non-word char, and we are in a word, then we've encountered the end of the
    // word.
    for ch in content.chars() {
        if ch.is_ascii_whitespace() {
            if word_char_count > 0 {
                total_word_count = total_word_count + 1;
                word_char_count = 0;
            }
            non_word_char_count = non_word_char_count + 1;
        } else {
            word_char_count = word_char_count + 1;
            non_word_char_count = 0;
        }
    }

    if word_char_count > 0 {
        total_word_count = total_word_count + 1;
    }

    return total_word_count;
}

#[test]
fn test_count_words_empty_string() {
    let result = count_words("");
    assert_eq!(result, 0);
}

#[test]
fn test_count_words() {
    let result = count_words("lorem ipsum dolor sit amet");
    assert_eq!(result, 5);
}
