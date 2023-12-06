use clap::Parser;
use std::io;
use std::collections::HashSet;
use std::io::BufRead;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short, help="additional column delimiters", default_value="")]
    delimiters: String,
}

fn update_spaces(delimiters: &HashSet<char>, mut spaces: Vec<bool>, string: &String) -> Vec<bool> {
    for (i, c) in string.chars().enumerate() {
        match spaces.get_mut(i) {
            Some(space) => *space = *space && (c.is_whitespace() || delimiters.contains(&c)),
            None => spaces.push(c.is_whitespace() || delimiters.contains(&c))
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
    let args = Cli::parse();
    let delimiter_set: HashSet<char> = args.delimiters.chars().collect();
    
    let stdin = io::stdin();
    let in_handle = stdin.lock();

    let lines = in_handle.lines().collect::<io::Result<Vec<String>>>()?;
    let spaces = lines.iter().fold(Vec::new(), |spaces, string| { update_spaces(&delimiter_set, spaces, string) });
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
        let spaces = update_spaces(&HashSet::new(), Vec::new(), &line);
        assert_eq!(spaces, vec![true, true, false, true, false, false, true, true, false, false, false, true])
    }

    #[test]
    fn update_spaces_two_lines() {
        let mut line = String::new();
        line.push_str("  a bb  ccc ");
        let spaces = update_spaces(&HashSet::new(), Vec::new(), &line);
        line.clear();
        line.push_str(" aa  b ccc  ");
        let spaces = update_spaces(&HashSet::new(), spaces, &line);
        assert_eq!(spaces, vec![true, false, false, true, false, false, true, false, false, false, false, true])
    }

    #[test]
    fn update_spaces_comma() {
        let mut line = String::new();
        line.push_str(",,a,bb,,ccc,");
        let mut delims = HashSet::new();
        delims.insert(',');
        let spaces = update_spaces(&delims, Vec::new(), &line);
        assert_eq!(spaces, vec![true, true, false, true, false, false, true, true, false, false, false, true])
    }

    #[test]
    fn update_spaces_mixed_delims() {
        let mut line = String::new();
        line.push_str(", a,bb, ccc,");
        let mut delims = HashSet::new();
        delims.insert(',');
        let spaces = update_spaces(&delims, Vec::new(), &line);
        assert_eq!(spaces, vec![true, true, false, true, false, false, true, true, false, false, false, true])
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
