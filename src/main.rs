use std::env;
use std::io;
use std::process;
#[derive(Debug, Clone, PartialEq)]
enum PatternAtom {
    Char(char),
    Digit,
    Word,
    PositiveGroup(String),
    NegativeGroup(String),
    Star(Box<PatternAtom>),
    Start(Vec<PatternAtom>),
}
fn split_pattern(pattern: &str) -> Vec<PatternAtom> {
    let mut i = 0;
    let mut pattern_atoms: Vec<PatternAtom> = vec![];
    while i < pattern.len() {
        let mut current_pattern: Option<PatternAtom> = None;
        if pattern.chars().nth(i) == Some('\\') {
            if pattern.chars().nth(i+1) == Some('d'){
                current_pattern = Some(PatternAtom::Digit);
            }else if  pattern.chars().nth(i+1) == Some('w') {
                current_pattern = Some(PatternAtom::Word);
            }else if  pattern.chars().nth(i+1) == Some('\\') {
                current_pattern = Some(PatternAtom::Char('\\'));
            }
            else{
                panic!("unhandles special symbol in pattern")
            }
            i += 2;
        } else if pattern.chars().nth(i) == Some('[') {
            let option_end_index = pattern[i + 1..].find("]");
            if let Some(length_of_subpattern) = option_end_index {
                // Check if rule positive or negative group
                if pattern.chars().nth(i+1) == Some('^'){
                    current_pattern = Some(PatternAtom::NegativeGroup(
                        (&pattern[i + 2..i + 1 + length_of_subpattern]).to_string(),
                    ));
                    i += 1 + length_of_subpattern + 1;

                }else{
                    current_pattern = Some(PatternAtom::PositiveGroup(
                        (&pattern[i + 1..i + 1 + length_of_subpattern]).to_string(),
                    ));
                    i += 1 + length_of_subpattern + 1;

                }
            } else {
                panic!("Invalid pattern, found '[' without a ']'")
            }
        } else if pattern.chars().nth(i) == Some('^'){
            // Start of string anchor
            let subpattern_vec = split_pattern(&pattern[1..]);
            current_pattern = Some(PatternAtom::Start(subpattern_vec));
            i = pattern.len();

        } else {
            let option_pattern_char = pattern.chars().nth(i);
            match option_pattern_char {
                Some(pattern_char) => {
                    current_pattern = Some(PatternAtom::Char(pattern_char));
                    i += 1;
                }
                None => panic!("Something happened and pattern doesn't have ith char"),
            }
        }
        match current_pattern {
            Some(sub_pattern) => pattern_atoms.push(sub_pattern),
            None => panic!("current_pattern should not be None"),
        }
    }
    pattern_atoms
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_start_anchor() {
        assert_eq!(split_pattern("^\\d"), vec![ PatternAtom::Start(vec![PatternAtom::Digit] )]);
    }

    #[test]
    fn test_single_digit() {
        assert_eq!(split_pattern("\\d"), vec![PatternAtom::Digit]);
    }

    #[test]
    fn test_two_digits() {
        assert_eq!(split_pattern("\\d\\d"), vec![PatternAtom::Digit, PatternAtom::Digit]);
    }

    #[test]
    fn test_two_digits_and_char() {
        assert_eq!(split_pattern("\\d\\dd"), vec![PatternAtom::Digit, PatternAtom::Digit, PatternAtom::Char('d')]);
    }

    #[test]
    fn test_two_words() {
        assert_eq!(split_pattern("\\w\\w"), vec![PatternAtom::Word, PatternAtom::Word]);
    }

    #[test]
    fn test_two_words_and_char() {
        assert_eq!(split_pattern("\\w\\ww"), vec![PatternAtom::Word, PatternAtom::Word, PatternAtom::Char('w')]);
    }

    #[test]
    fn test_positive_group() {
        assert_eq!(split_pattern("[abc]"), vec![PatternAtom::PositiveGroup("abc".to_string())]);
    }

    #[test]
    fn test_positive_group_and_char() {
        assert_eq!(split_pattern("[abc]a"), vec![PatternAtom::PositiveGroup("abc".to_string()), PatternAtom::Char('a')]);
    }

    #[test]
    fn test_recognizes_digit_at_start() {
        assert_eq!(find_pattern_atom_at_start("1", &PatternAtom::Digit), true);
        assert_eq!(find_pattern_atom_at_start("a", &PatternAtom::Digit), false);

        assert_eq!(find_pattern_atom_at_start("1 dsf", &PatternAtom::Digit), true);
        assert_eq!(find_pattern_atom_at_start("a dsf", &PatternAtom::Digit), false);
    }

    #[test]
    fn test_recognizes_word_at_start() {
        assert_eq!(find_pattern_atom_at_start("&", &PatternAtom::Word), false);
        assert_eq!(find_pattern_atom_at_start("a", &PatternAtom::Word), true);

        assert_eq!(find_pattern_atom_at_start("& dsf", &PatternAtom::Word), false);
        assert_eq!(find_pattern_atom_at_start("a dsf", &PatternAtom::Word), true);
    }

    #[test]
    fn test_recognizes_positive_group_at_start() {
        assert_eq!(find_pattern_atom_at_start("1", &PatternAtom::PositiveGroup("abc".to_string())), false);
        assert_eq!(find_pattern_atom_at_start("a", &PatternAtom::PositiveGroup("abc".to_string())), true);
        assert_eq!(find_pattern_atom_at_start("aft", &PatternAtom::PositiveGroup("abc".to_string())), true);
        assert_eq!(find_pattern_atom_at_start("123", &PatternAtom::PositiveGroup("abc".to_string())), false);
    }


    #[test]
    fn test_recognizes_negative_group_at_start() {
        assert_eq!(find_pattern_atom_at_start("1", &PatternAtom::NegativeGroup("abc".to_string())), true);
        assert_eq!(find_pattern_atom_at_start("a", &PatternAtom::NegativeGroup("abc".to_string())), false);
        assert_eq!(find_pattern_atom_at_start("aft", &PatternAtom::NegativeGroup("abc".to_string())), false);
        assert_eq!(find_pattern_atom_at_start("123", &PatternAtom::NegativeGroup("abc".to_string())), true);
    }
    #[test]
    fn test_recognizes_char_at_start(){
        assert_eq!(find_pattern_atom_at_start("ffe", &PatternAtom::Char('c')), false);
        assert_eq!(find_pattern_atom_at_start("ffe", &PatternAtom::Char('f')), true);
    }

    #[test]
    fn test_recognizes_start_anchor() {
        assert_eq!(find_pattern_atom_at_start("ffe", &PatternAtom::Start(vec![PatternAtom::Digit])), false);
        assert_eq!(find_pattern_atom_at_start("1", &PatternAtom::Digit), true);
        assert_eq!(find_pattern_atom_at_start("1", &PatternAtom::Start(vec![PatternAtom::Digit])), true);
    }


    #[test]
    fn test_regression_chars(){
        assert_eq!(match_pattern("ffe", "ffe"), true);
        assert_eq!(match_pattern("1e", "\\d\\w"), true);
        assert_eq!(match_pattern("e1", "\\d\\w"), false);
        assert_eq!(match_pattern("1e", "^\\d\\w"), true);
        assert_eq!(match_pattern("e1e", "^\\d\\w"), false);
    }
}

fn find_pattern_atom_at_start(input_line: &str, pattern_atom: &PatternAtom) -> bool {
    match pattern_atom{
        PatternAtom::Char(c)=> input_line.find(*c) == Some(0),
        PatternAtom::Digit=> {
            let option_char =  input_line.chars().nth(0);
            let is_digit = match option_char {
                Some(c)=>  c.is_numeric(),
                None => false
            };
            return is_digit;
        }
        PatternAtom::Word=>{
            let option_char =  input_line.chars().nth(0);
            let is_word = match option_char {
                Some(c)=>  c.is_alphanumeric() || c == '_',
                None => false
            };
            return is_word;

        }
        PatternAtom::PositiveGroup(positive_group)=>{
            let option_char =  input_line.chars().nth(0);
            let is_part_of_positive_group = match option_char {
                Some(c)=>  positive_group.contains(c),
                None => false
            };
            return is_part_of_positive_group;
        }
        PatternAtom::NegativeGroup(negative_group)=>{
            let option_char =  input_line.chars().nth(0);
            let is_part_of_negative_group = match option_char {
                Some(c)=>  negative_group.contains(c),
                None => false
            };
            return !is_part_of_negative_group;
        }
        PatternAtom::Start(subpattern)=>{
            return match_string_exactly(input_line, subpattern);
        }
        _ => panic!("Unhandled pattern atom")
    }
}

fn match_string_exactly(input: &str, pattern: &Vec<PatternAtom>) -> bool{
    let mut i = 0;
    for pattern_atom in pattern{
        if find_pattern_atom_at_start(&input[i..], pattern_atom){
            i+=1;
        }
        else{
           return false
        }
    }
    true
}

// }
fn match_pattern(input_line: &str, pattern: &str)-> bool{
    let pattern_vec = split_pattern(pattern);
    for (position, _) in input_line.char_indices(){
        if match_string_exactly(&input_line[ position..input_line.len() ], &pattern_vec){
            return true;
        }
        if let PatternAtom::Start(_) = pattern_vec[0]{
            return false;
        }
    }
    return false;
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");

    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    // Uncomment this block to pass the first stage
    if match_pattern(&input_line, &pattern) {
        eprintln!("Yes");
        process::exit(0)
    } else {
        eprintln!("No");
        process::exit(1)
    }
}
