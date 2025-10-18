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
    Wild,
    Star(Box<PatternAtom>),
    Question(Box<PatternAtom>),
    Start(Vec<PatternAtom>),
    End,
    Alternation(Vec<Vec<PatternAtom>>),
    BackreferenceGroup(Vec<PatternAtom>),
    BackreferenceInt(usize),
}
fn split_pattern(pattern: &str) -> Vec<PatternAtom> {
    let mut i = 0;
    let mut pattern_atoms: Vec<PatternAtom> = vec![];
    while i < pattern.len() {
        let mut current_pattern: Option<PatternAtom> = None;
        if pattern.chars().nth(i) == Some('\\') {
            if pattern.chars().nth(i + 1) == Some('d') {
                current_pattern = Some(PatternAtom::Digit);
            } else if pattern.chars().nth(i + 1) == Some('w') {
                current_pattern = Some(PatternAtom::Word);
            } else if pattern.chars().nth(i + 1) == Some('\\') {
                current_pattern = Some(PatternAtom::Char('\\'));
            } else if pattern.chars().nth(i + 1).is_some_and(|c| c.is_digit(10)) {
                let option_len_of_backreference_int: Option<usize> =
                    pattern[i + 1..].find(|c: char| !c.is_digit(10));
                match option_len_of_backreference_int {
                    Some(int_length) => {
                        let the_int: usize = pattern[i + 1..i + 1 + int_length].parse().unwrap();
                        current_pattern = Some(PatternAtom::BackreferenceInt(the_int));
                        i += int_length - 1;
                    }
                    None => {
                        current_pattern = Some(PatternAtom::BackreferenceInt(
                            pattern[i + 1..].parse().unwrap(),
                        ));
                        i = pattern.len();
                    }
                }
            } else {
                panic!("unhandles special symbol in pattern")
            }
            i += 2;
        } else if pattern.chars().nth(i) == Some('[') {
            let option_end_index = pattern[i + 1..].find("]");
            if let Some(length_of_subpattern) = option_end_index {
                // Check if rule positive or negative group
                if pattern.chars().nth(i + 1) == Some('^') {
                    current_pattern = Some(PatternAtom::NegativeGroup(
                        (&pattern[i + 2..i + 1 + length_of_subpattern]).to_string(),
                    ));
                    i += 1 + length_of_subpattern + 1;
                } else {
                    current_pattern = Some(PatternAtom::PositiveGroup(
                        (&pattern[i + 1..i + 1 + length_of_subpattern]).to_string(),
                    ));
                    i += 1 + length_of_subpattern + 1;
                }
            } else {
                panic!("Invalid pattern, found '[' without a ']'")
            }
        } else if pattern.chars().nth(i) == Some('(') {
            let option_end_index = pattern[i + 1..].find(")");
            if let Some(length_of_subpattern) = option_end_index {
                let vec_of_subpatterns_strings: Vec<String> = pattern
                    [i + 1..i + 1 + length_of_subpattern]
                    .split('|')
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect();
                if vec_of_subpatterns_strings.len() == 1 {
                    current_pattern = Some(PatternAtom::BackreferenceGroup(split_pattern(
                        &vec_of_subpatterns_strings[0],
                    )));
                } else {
                    current_pattern = Some(PatternAtom::Alternation(
                        vec_of_subpatterns_strings
                            .iter()
                            .map(|p| split_pattern(p))
                            .collect(),
                    ));
                }
                i += 1 + length_of_subpattern + 1;
            } else {
                panic!("Invalid pattern, found '(' without a ')'")
            }
        } else if pattern.chars().nth(i) == Some('^') {
            // Start of string anchor
            let subpattern_vec = split_pattern(&pattern[1..]);
            current_pattern = Some(PatternAtom::Start(subpattern_vec));
            i = pattern.len();
        } else if i == pattern.len() - 1 && pattern.chars().nth(i) == Some('$') {
            current_pattern = Some(PatternAtom::End);
            i += 1;
        } else if pattern.chars().nth(i) == Some('+') {
            current_pattern = Some(PatternAtom::Star(Box::new(
                pattern_atoms[pattern_atoms.len() - 1].clone(),
            )));
            i += 1;
        } else if pattern.chars().nth(i) == Some('*') {
            let option_last_pattern_atom = pattern_atoms.pop();
            match option_last_pattern_atom {
                Some(last_pattern_atom) => {
                    current_pattern = Some(PatternAtom::Star(Box::new(last_pattern_atom)));
                    i += 1;
                }
                None => panic!(
                    "There was no last pattern for star. Likely, you have star at start of your pattern, which is not allowed"
                ),
            }
        } else if pattern.chars().nth(i) == Some('?') {
            let option_last_pattern_atom = pattern_atoms.pop();
            match option_last_pattern_atom {
                Some(last_pattern_atom) => {
                    current_pattern = Some(PatternAtom::Question(Box::new(last_pattern_atom)));
                    i += 1;
                }
                None => panic!(
                    "There was no last pattern for star. Likely, you have star at start of your pattern, which is not allowed"
                ),
            }
        }
        else if pattern.chars().nth(i) == Some('.') {
            current_pattern = Some(PatternAtom::Wild);
            i += 1;
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

fn find_pattern_atom_at_start(input_line: &str, pattern_atom: &PatternAtom) -> bool {
    // eprintln!("fpa: {}, {:?}", input_line, pattern_atom);
    match pattern_atom {
        PatternAtom::Char(c) => input_line.find(*c) == Some(0),
        PatternAtom::Digit => {
            let option_char = input_line.chars().nth(0);
            let is_digit = match option_char {
                Some(c) => c.is_numeric(),
                None => false,
            };
            return is_digit;
        }
        PatternAtom::Word => {
            let option_char = input_line.chars().nth(0);
            let is_word = match option_char {
                Some(c) => c.is_alphanumeric() || c == '_',
                None => false,
            };
            return is_word;
        }
        PatternAtom::PositiveGroup(positive_group) => {
            let option_char = input_line.chars().nth(0);
            let is_part_of_positive_group = match option_char {
                Some(c) => positive_group.contains(c),
                None => false,
            };
            return is_part_of_positive_group;
        }
        PatternAtom::NegativeGroup(negative_group) => {
            let option_char = input_line.chars().nth(0);
            let is_part_of_negative_group = match option_char {
                Some(c) => negative_group.contains(c),
                None => false,
            };
            return !is_part_of_negative_group;
        }
        PatternAtom::End => {
            if input_line.len() == 0 {
                return true;
            } else {
                return false;
            }
        }
        PatternAtom::Wild => {
            if input_line.len() == 0 {
                return false;
            }
            return true;
        }
        _ => panic!("Unhandled pattern atom: {:?}", pattern_atom),
    }
}

fn match_string_exactly(
    input: &str,
    pattern: &[PatternAtom],
    backreference_groups: &[String],
) -> Option<String> {
    let mut i = 0;
    let mut local_backreference_group = vec![];
    // eprintln!("input: {:?}, pattern: {:?}",input, pattern);
    local_backreference_group.extend_from_slice(backreference_groups);
    for (index, pattern_atom) in pattern.iter().enumerate() {
        // eprintln!("i: {:?}, p: {:?}, b: {:?}", &input[i..], &pattern[index..], local_backreference_group);
        match pattern_atom {
            PatternAtom::Star(subpattern_atom) => loop {
                if pattern.len()-index > 1 {
                    if let Some(matched_string) = match_string_exactly(
                        &input[i..],
                        &pattern[index + 1..],
                        &local_backreference_group,
                    ) {
                        return Some(input[0..i].to_string() + &matched_string);
                    }
                }
                if i == input.len(){
                    break
                } else if !find_pattern_atom_at_start(&input[i..], subpattern_atom) {
                    break;
                } else {
                    i += 1;
                    while i < input.len() && !input.is_char_boundary(i) {
                        i += 1;
                    }
                }

                if i > input.len() {
                    return None;
                }
            },

            PatternAtom::Question(subpattern_atom) => {
                if pattern.len()-index > 1 {
                    if let Some(matched_string) = match_string_exactly(
                        &input[i..],
                        &pattern[index + 1..],
                        &local_backreference_group,
                    ) {
                        return Some(input[0..i].to_string() + &matched_string);
                    }
                }
                if i == input.len(){
                    // eprintln!("break quest 1");
                } else if !find_pattern_atom_at_start(&input[i..], subpattern_atom) {
                    // eprintln!("break quest 2");
                } else {
                    i += 1;
                    while i < input.len() && !input.is_char_boundary(i) {
                        i += 1;
                    }
                }

                if i > input.len() {
                    return None;
                }
            },
            PatternAtom::BackreferenceGroup(sub_pattern) => {
                if let Some(matched_string) =
                    match_string_exactly(&input[i..], sub_pattern, &local_backreference_group)
                {
                    // eprintln!("brg matched s: {}, i: {}", matched_string, i);
                    i += matched_string.len();
                    local_backreference_group.push(matched_string)
                }else{
                    return None;
                }
            }
            PatternAtom::BackreferenceInt(group_num) => {
                if (*group_num - 1) < local_backreference_group.len() {
                    let leftover_pattern = if index < pattern.len() - 1 {
                        &[
                            &split_pattern(&local_backreference_group[group_num - 1]),
                            &pattern[index + 1..],
                        ]
                        .concat()
                    } else {
                        &split_pattern(&local_backreference_group[group_num - 1])
                    };
                    if let Some(matched_string) = match_string_exactly(
                        &input[i..],
                        leftover_pattern,
                        &local_backreference_group,
                    ) {
                        return Some(input[0..i].to_string() + &matched_string);
                    }else{
                        return None;
                    }
                } else {
                    return None;
                }
            }
            PatternAtom::Alternation(vec_of_vec_pattern) => {
                return vec_of_vec_pattern.iter().find_map(|vp| {
                    if let Some(matched_string) = match_string_exactly(
                        &input[i..],
                        &[&vec![ PatternAtom::BackreferenceGroup(vp[..].to_vec() ) ], &pattern[index + 1..]].concat(),
                        &local_backreference_group,
                    ) {
                        // eprintln!("match found: {}", matched_string);
                        return Some(input[0..i].to_string() + &matched_string);
                    }
                    return None;
                });
            }
            PatternAtom::End =>{
                // eprintln!("Checking end, {}, {}, {}, {}", i, input.len(), input.len()==0, i < input.len()-1);
                if input.len() ==0 ||  i == input.len(){
                }else{
                    return None;
                }

            }
            _ => {
                if find_pattern_atom_at_start(&input[i..], pattern_atom) {
                    i += 1;
                    while i < input.len() && !input.is_char_boundary(i) {
                        i += 1;
                    }
                } else {
                    return None;
                }
            }
        }
    }
    Some(input[0..i].to_string())
}

// }
fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let pattern_vec = split_pattern(pattern);
    if pattern_vec.len() == 1
        && let PatternAtom::Start(subpattern) = &pattern_vec[0]
    {
        if let Some(_) = match_string_exactly(&input_line, &subpattern, &vec![]) {
            return true;
        }
        return false;
    } else {
        for (position, _) in input_line.char_indices() {
            if let Some(_) = match_string_exactly(
                &input_line[position..input_line.len()],
                &pattern_vec,
                &vec![],
            ) {
                return true;
            }
            if let PatternAtom::Start(_) = pattern_vec[0] {
                return false;
            }
        }
        return false;
    }
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
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

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(test)]
    mod parse_pattern_tests {
        // Note this useful idiom: importing names from outer (for mod tests) scope.
        use super::*;

        #[test]
        fn test_start_anchor() {
            assert_eq!(
                split_pattern("^\\d"),
                vec![PatternAtom::Start(vec![PatternAtom::Digit])]
            );
        }

        #[test]
        fn test_end_anchor() {
            assert_eq!(
                split_pattern("^\\d$"),
                vec![PatternAtom::Start(vec![
                    PatternAtom::Digit,
                    PatternAtom::End
                ])]
            );
            assert_eq!(
                split_pattern("\\d$"),
                vec![PatternAtom::Digit, PatternAtom::End]
            );
        }

        #[test]
        fn test_single_digit() {
            assert_eq!(split_pattern("\\d"), vec![PatternAtom::Digit]);
        }

        #[test]
        fn test_two_digits() {
            assert_eq!(
                split_pattern("\\d\\d"),
                vec![PatternAtom::Digit, PatternAtom::Digit]
            );
        }

        #[test]
        fn test_two_digits_and_char() {
            assert_eq!(
                split_pattern("\\d\\dd"),
                vec![
                    PatternAtom::Digit,
                    PatternAtom::Digit,
                    PatternAtom::Char('d')
                ]
            );
        }

        #[test]
        fn test_two_words() {
            assert_eq!(
                split_pattern("\\w\\w"),
                vec![PatternAtom::Word, PatternAtom::Word]
            );
        }

        #[test]
        fn test_two_words_and_char() {
            assert_eq!(
                split_pattern("\\w\\ww"),
                vec![PatternAtom::Word, PatternAtom::Word, PatternAtom::Char('w')]
            );
        }

        #[test]
        fn test_positive_group() {
            assert_eq!(
                split_pattern("[abc]"),
                vec![PatternAtom::PositiveGroup("abc".to_string())]
            );
        }

        #[test]
        fn test_alternation() {
            assert_eq!(
                split_pattern("(abc|ekg\\d)"),
                vec![PatternAtom::Alternation(vec![
                    vec![
                        PatternAtom::Char('a'),
                        PatternAtom::Char('b'),
                        PatternAtom::Char('c')
                    ],
                    vec![
                        PatternAtom::Char('e'),
                        PatternAtom::Char('k'),
                        PatternAtom::Char('g'),
                        PatternAtom::Digit
                    ]
                ])]
            );
        }

        #[test]
        fn test_backreference_group() {
            assert_eq!(
                split_pattern("(abc)"),
                vec![PatternAtom::BackreferenceGroup(vec![
                    PatternAtom::Char('a'),
                    PatternAtom::Char('b'),
                    PatternAtom::Char('c')
                ],)]
            );

            assert_eq!(
                split_pattern("(abc)\\d\\w"),
                vec![
                    PatternAtom::BackreferenceGroup(vec![
                        PatternAtom::Char('a'),
                        PatternAtom::Char('b'),
                        PatternAtom::Char('c')
                    ],),
                    PatternAtom::Digit,
                    PatternAtom::Word,
                ]
            );

            assert_eq!(
                split_pattern("\\d(abc)\\w"),
                vec![
                    PatternAtom::Digit,
                    PatternAtom::BackreferenceGroup(vec![
                        PatternAtom::Char('a'),
                        PatternAtom::Char('b'),
                        PatternAtom::Char('c')
                    ],),
                    PatternAtom::Word,
                ]
            );
        }

        #[test]
        fn test_backreference_int() {
            assert_eq!(
                split_pattern("\\1\\2\\10003"),
                vec![
                    PatternAtom::BackreferenceInt(1),
                    PatternAtom::BackreferenceInt(2),
                    PatternAtom::BackreferenceInt(10003),
                ]
            );

            assert_eq!(
                split_pattern("\\w\\1\\2\\d"),
                vec![
                    PatternAtom::Word,
                    PatternAtom::BackreferenceInt(1),
                    PatternAtom::BackreferenceInt(2),
                    PatternAtom::Digit,
                ]
            );
        }

        #[test]
        fn test_positive_group_and_char() {
            assert_eq!(
                split_pattern("[abc]a"),
                vec![
                    PatternAtom::PositiveGroup("abc".to_string()),
                    PatternAtom::Char('a')
                ]
            );
        }
        #[test]
        fn test_plus() {
            assert_eq!(
                split_pattern("a+"),
                vec![
                    PatternAtom::Char('a'),
                    PatternAtom::Star(Box::new(PatternAtom::Char('a')))
                ]
            );

            assert_eq!(
                split_pattern("ba+"),
                vec![
                    PatternAtom::Char('b'),
                    PatternAtom::Char('a'),
                    PatternAtom::Star(Box::new(PatternAtom::Char('a')))
                ]
            );
        }

        #[test]
        fn test_star() {
            assert_eq!(
                split_pattern("a*"),
                vec![PatternAtom::Star(Box::new(PatternAtom::Char('a')))]
            );

            assert_eq!(
                split_pattern("ba*"),
                vec![
                    PatternAtom::Char('b'),
                    PatternAtom::Star(Box::new(PatternAtom::Char('a')))
                ]
            );

            assert_eq!(
                split_pattern("ca?g"),
                vec![
                    PatternAtom::Char('c'),
                    PatternAtom::Question(Box::new(PatternAtom::Char('a'))),
                    PatternAtom::Char('g'),
                ]
            );
        }

        #[test]
        fn test_wild() {
            assert_eq!(split_pattern("."), vec![PatternAtom::Wild]);

            assert_eq!(
                split_pattern("g.+g"),
                vec![
                    PatternAtom::Char('g'),
                    PatternAtom::Wild,
                    PatternAtom::Star(Box::new(PatternAtom::Wild)),
                    PatternAtom::Char('g'),
                ]
            );
        }
    }
    #[cfg(test)]
    mod recognize_pattern_atom {
        use super::*;
        #[test]
        fn test_recognizes_digit_at_start() {
            assert_eq!(find_pattern_atom_at_start("1", &PatternAtom::Digit), true);
            assert_eq!(find_pattern_atom_at_start("a", &PatternAtom::Digit), false);

            assert_eq!(
                find_pattern_atom_at_start("1 dsf", &PatternAtom::Digit),
                true
            );
            assert_eq!(
                find_pattern_atom_at_start("a dsf", &PatternAtom::Digit),
                false
            );
        }

        #[test]
        fn test_recognizes_word_at_start() {
            assert_eq!(find_pattern_atom_at_start("&", &PatternAtom::Word), false);
            assert_eq!(find_pattern_atom_at_start("a", &PatternAtom::Word), true);

            assert_eq!(
                find_pattern_atom_at_start("& dsf", &PatternAtom::Word),
                false
            );
            assert_eq!(
                find_pattern_atom_at_start("a dsf", &PatternAtom::Word),
                true
            );
        }

        #[test]
        fn test_recognizes_positive_group_at_start() {
            assert_eq!(
                find_pattern_atom_at_start("1", &PatternAtom::PositiveGroup("abc".to_string())),
                false
            );
            assert_eq!(
                find_pattern_atom_at_start("a", &PatternAtom::PositiveGroup("abc".to_string())),
                true
            );
            assert_eq!(
                find_pattern_atom_at_start("aft", &PatternAtom::PositiveGroup("abc".to_string())),
                true
            );
            assert_eq!(
                find_pattern_atom_at_start("123", &PatternAtom::PositiveGroup("abc".to_string())),
                false
            );
        }

        #[test]
        fn test_recognizes_negative_group_at_start() {
            assert_eq!(
                find_pattern_atom_at_start("1", &PatternAtom::NegativeGroup("abc".to_string())),
                true
            );
            assert_eq!(
                find_pattern_atom_at_start("a", &PatternAtom::NegativeGroup("abc".to_string())),
                false
            );
            assert_eq!(
                find_pattern_atom_at_start("aft", &PatternAtom::NegativeGroup("abc".to_string())),
                false
            );
            assert_eq!(
                find_pattern_atom_at_start("123", &PatternAtom::NegativeGroup("abc".to_string())),
                true
            );
        }
        #[test]
        fn test_recognizes_char_at_start() {
            assert_eq!(
                find_pattern_atom_at_start("ffe", &PatternAtom::Char('c')),
                false
            );
            assert_eq!(
                find_pattern_atom_at_start("ffe", &PatternAtom::Char('f')),
                true
            );
        }

        #[test]
        fn test_recognizes_wild_at_start() {
            assert_eq!(find_pattern_atom_at_start("ffe", &PatternAtom::Wild), true);
            assert_eq!(find_pattern_atom_at_start("", &PatternAtom::Wild), false);
        }

        #[test]
        fn test_recognizes_start_anchor() {
            assert_eq!(
                match_pattern("ffe", "^\\d"),
                false
            );
            assert_eq!(find_pattern_atom_at_start("1", &PatternAtom::Digit), true);

            assert_eq!(
                match_pattern("1", "^\\d"),
               true
            );
        }

        #[test]
        fn test_recognizes_end_anchor() {
            assert_eq!(find_pattern_atom_at_start("", &PatternAtom::End), true);
        }


        #[test]
        fn test_recognizes_star_with_word() {
            assert_eq!(
                match_string_exactly(
                    "abc",
                    &vec![PatternAtom::Star(Box::new(PatternAtom::Word))],
                    &vec![]
                ),
                Some("abc".to_string())
            );

            assert_eq!(match_string_exactly("12 12", &split_pattern( "(\\w+)" ),
                    &vec![]
            ), Some( "12".to_string() ));
            assert_eq!(
                match_string_exactly(
                    "abc ",
                    &vec![ PatternAtom::BackreferenceGroup( vec![PatternAtom::Star(Box::new(PatternAtom::Word))] ) ],
                    &vec![]
                ),
                Some("abc".to_string())
            );

        }

        #[test]
        fn test_recognizes_star() {
            assert_eq!(
                match_string_exactly(
                    "",
                    &vec![PatternAtom::Star(Box::new(PatternAtom::Digit))],
                    &vec![]
                ),
                Some("".to_string())
            );
            assert_eq!(
                match_string_exactly(
                    "1",
                    &vec![PatternAtom::Star(Box::new(PatternAtom::Digit))],
                    &vec![]
                ),
                Some("1".to_string())
            );
            assert_eq!(
                match_string_exactly(
                    "1234",
                    &vec![PatternAtom::Star(Box::new(PatternAtom::Digit))],
                    &vec![]
                ),
                Some("1234".to_string())
            );
            assert_eq!(
                match_string_exactly(
                    "abc",
                    &vec![
                        PatternAtom::Digit,
                        PatternAtom::Star(Box::new(PatternAtom::Digit))
                    ],
                    &vec![]
                ),
                None
            );
        }
    }

    #[cfg(test)]
    mod integration_pattern_atom {
        use super::*;
        #[test]
        fn test_regression_chars() {
            assert_eq!(match_pattern("ffe", "ffe"), true);
            assert_eq!(match_pattern("1e", "\\d\\w"), true);
            assert_eq!(match_pattern("e1", "\\d\\w"), false);
            assert_eq!(match_pattern("1e", "^\\d\\w"), true);
            assert_eq!(match_pattern("e1e", "^\\d\\w"), false);
        }

        #[test]
        fn test_regression_star() {
            assert_eq!(match_pattern("ffe", "ffe+"), true);
            assert_eq!(match_pattern("ff", "ffe+"), false);
            assert_eq!(match_pattern("caaats", "ca+at"), true);
            assert_eq!(match_pattern("caaats", "ca+bt"), false);
            assert_eq!(match_pattern("cag", "ca?t"), false);
            assert_eq!(match_pattern("gol", "g.l"), true);
            assert_eq!(match_pattern("gol", "g.+l"), true);
            assert_eq!(match_pattern("gol", "g.+gl"), false);
            assert_eq!(match_pattern("gol", "golll"), false);
        }

        #[test]
        fn test_regression_alternation() {
            assert_eq!(match_pattern("abc", "(abc|bdf|hello)"), true);
            assert_eq!(match_pattern("abt", "(abc|bdf|hello)"), false);
            assert_eq!(match_pattern("ab1", "(ab|bdf|hello)\\d"), true);
            assert_eq!(match_pattern("a1", "(ab*|bdf|hello)\\d"), true);
        }

        #[test]
        fn test_regression_backreference() {
            assert_eq!(match_pattern("cat and cat", "(cat) and \\1"), true);
            assert_eq!(match_pattern("cat and dog", "(cat) and \\1"), false);
            assert_eq!(match_pattern("cat and cat and cat", "(cat) and \\1 and \\1"), true);
            assert_eq!(match_pattern("cat and cat and cat", "(cat) (and) \\1 \\2 \\1"), true);
            assert_eq!(match_pattern("1 and 1 and 1", "(\\d) (and) \\1 \\2 \\1"), true);
            assert_eq!(match_pattern("1 and 1 and 1", "(\\w) (and) \\1 \\2 \\1"), true);
            assert_eq!(match_pattern("12 12", "(\\w+) \\1"), true);
            assert_eq!(match_pattern("12 and 12 and 12", "(\\w+) (and) \\1 \\2 \\1"), true);
        }

        #[test]
        fn test_regression_backreference2() {
            // assert_eq!(match_pattern("cat is cat, not dog", "^([act]+) is \\1, not [^xyz]+$"), true);
            // assert_eq!(match_pattern("I see 1 cat", "^I see \\d+ (cat|dog)s?$"), true);
            assert_eq!(match_pattern("I see 2 dog3", "^I see \\d+ (cat|dog)s?$"), false);
        }
        #[test]
        fn test_regression_multiple_backrefernece(){
            assert_eq!(match_pattern("3 red squares and 3 red circles", "(\\d+) (\\w+) squares and \\1 \\2 circles"), true);
            assert_eq!(match_pattern("abc-def is abc-def, not efg", "([abc]+)-([def]+) is \\1-\\2, not [^xyz]+"), true);
            assert_eq!(match_pattern("howwdy heeey there, howwdy heeey", "(how+dy) (he?y) there, \\1 \\2"), false);
            assert_eq!(match_pattern("cat and fish, cat with fish", "(c.t|d.g) and (f..h|b..d), \\1 with \\2"), true);
            assert_eq!(match_pattern("a cog", "a (cat|dog)"), false);
            assert_eq!(match_pattern("apple pie, apple and pie", "^(\\w+) (\\w+), \\1 and \\2$"), true);
        }
    }
}
