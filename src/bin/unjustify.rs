use clap::Parser;
use std::io;
use std::io::{BufRead, Write};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long="delimiters", short='d', help="ignored", default_value="")]
    delimeters: String,
}

fn update_spaces(mut spaces: Vec<bool>, string: &String) -> Vec<bool> {
    for (i, c) in string.chars().enumerate() {
        match spaces.get_mut(i) {
            Some(space) => *space = *space && c.is_whitespace(),
            None => spaces.push(c.is_whitespace())
        }
    }
    spaces
}

fn columns(spaces: &Vec<bool>) -> Vec<(usize, usize)> {
    let mut ret = Vec::new();
    let mut start: Option<usize> = None;
    for (i, space) in spaces.iter().enumerate() {
        match (start, space) {
            (None,false) => {
                start = Some(i);
            },
            (Some(pos), true) => {
                ret.push((pos, i));
                start = None;
            }
            (_, _) => (),
        }
    }
    ret
}

fn main() -> io::Result<()> {
    let _args = Cli::parse();
    
    let stdin = io::stdin();
    let in_handle = stdin.lock();

    let lines = in_handle.lines().collect::<io::Result<Vec<String>>>()?;
    let spaces = lines.iter().fold(Vec::new(), update_spaces);
    let columns = columns(&spaces);

    let mut outln: Vec<String> = Vec::new();
    for string in lines {
        let line: Vec<char> = string.chars().collect();
        for (s, e) in &columns {
            outln.push(line[*s..*e].iter().collect::<String>().trim().to_string());
        }
        println!("{}", &outln.join(","));
        outln.clear();
    }

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn update_spaces_first_line() {
        let mut line = String::new();
        line.push_str("  a bb  ccc ");
        let spaces = update_spaces(Vec::new(), &line);
        assert_eq!(spaces, vec![true, true, false, true, false, false, true, true, false, false, false, true])
    }

    #[test]
    fn update_spaces_two_lines() {
        let mut line = String::new();
        line.push_str("  a bb  ccc ");
        let spaces = update_spaces(Vec::new(), &line);
        line.clear();
        line.push_str(" aa  b ccc  ");
        let spaces = update_spaces(spaces, &line);
        assert_eq!(spaces, vec![true, false, false, true, false, false, true, false, false, false, false, true])
    }

    #[test]
    fn columns_first_line() {
        let runs = columns(&vec![true, true, false, true, false, false, true, true, false, false, false, true]);
        assert_eq!(runs, vec![(2, 3), (4, 6), (8, 11)]);
    }
    
    #[test]
    fn columns_two_lines() {
        let runs = columns(&vec![true, false, false, true, false, false, true, false, false, false, false, true]);
        assert_eq!(runs, vec![(1, 3), (4, 6), (7, 11)]);
    }
}
