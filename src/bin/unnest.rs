use std::io;
use std::collections::HashMap;
use serde_json;
use serde_json::{Map, Value};

type Columns = Vec<String>; // consider a parallel Set to make contains cheaper
type Row = HashMap<String, String>;

fn ensure(columns: &mut Columns, name: &str) {
    let name = name.to_string();
    if !columns.contains(&name) {
        columns.push(name);
    }
}

fn singleton(path: &str, value: String) -> Vec<Row> {
    let mut row = HashMap::new();
    row.insert(path.to_string(), value);
    let mut ret = Vec::new();
    ret.push(row);
    ret
}

// a single row with no columns, as base case for fold
fn empty() -> Vec<Row> {
    let row = HashMap::new();
    let mut ret = Vec::new();
    ret.push(row);
    ret
}

fn recurse_array(columns: &mut Columns, path: &str, array: Vec<Value>) -> Vec<Row> {
    array.iter().fold(empty(), |mut acc, val| {
        acc.append(&mut recurse_value(columns, path, val.clone()));
        acc
    })
}

fn recurse_map(columns: &mut Columns, path: &str, map: Map<String, Value>) -> Vec<Row> {
    map.iter().fold(empty(), |acc, (key, value)| {
        let path_key = if path == "" { key.clone() } else { format!("{}.{}", path, key) };
        let rhs = recurse_value(columns, &path_key, value.clone());
        let mut ret = Vec::new();
        // merge each possible pair of partial rows
        for a in acc {
            for b in &rhs {
                let mut r = a.clone();
                r.extend(b.clone());
                ret.push(r);
            }
        }
        ret
    })
}

fn recurse_value(columns: &mut Columns, path: &str, value: Value) -> Vec<Row> {
    match value {
        Value::Null => Vec::new(),
        Value::String(s) => {
            ensure(columns, path);
            singleton(path, s)
        },
        Value::Bool(b) => {
            ensure(columns, path);
            singleton(path, format!("{}", b))
        },
        Value::Number(n) => {
            ensure(columns, path);
            singleton(path, format!("{}", n))
        },
        Value::Array(arr) => recurse_array(columns, path, arr),
        Value::Object(map) => recurse_map(columns, path, map),
    }
}

fn main() -> io::Result<()> {
    // parse args
    // parse stdin as json
    let json: Value = serde_json::from_reader(io::stdin())?;

    // recurse into json, building columns & rows as we go
    let mut columns: Columns = Vec::new();
    // TODO validate that outer is an array?
    let rows = recurse_value(&mut columns, "", json);

    // output
    println!("{}", columns.join(" "));
    for r in rows.iter() {
        let mut out = Vec::new();
        for c in columns.iter() {
            if let Some(s) = r.get(c) {
                out.push(s.clone())
            }
        }
        println!("{}", out.join(" "));
    }
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use serde_json::json;

    fn leafs() -> Value {
        json!({
            "n": 123,
            "b": true,
        })
    }

    fn assert_columns_eq(actual: &Columns, expected: &str) {
        let words: Vec<&str> = expected.split_whitespace().collect();
        assert!(actual.len() == words.len(), "{:?} has length {} expected {}", actual, actual.len(), words.len());
        for (a, e) in std::iter::zip(actual, words) {
            assert_eq!(a, e);
        }
    }

    fn assert_row_eq(actual: &Row, expected: &str) {
        let pairs: Vec<(&str, &str)> = expected.split_whitespace().map(|w| {
            let mut kv = w.split(":");
            (kv.next().unwrap(), kv.next().unwrap())
        }).collect();
        assert!(actual.len() == pairs.len(), "{:?} has length {} expected {}", actual, actual.len(), pairs.len());
        for (k, v) in pairs {
            let o_v = actual.get(k);
            match o_v {
                None => panic!("expected row to include key {}", k),
                Some(a) => assert!(a == v, "row has {} at key {} expected {}", a, k, v),
            }
        }
    }

    #[test]
    fn columns_leafs() {
        let mut columns = Vec::new();
        recurse_value(&mut columns, "", leafs());
        assert_columns_eq(&columns, "b n");
    }

    #[test]
    fn row_leafs() {
        let mut columns = Vec::new();
        let rows = recurse_value(&mut columns, "", leafs());
        assert_eq!(rows.len(), 1);
        assert_row_eq(&rows[0], "n:123 b:true");
    }
}
