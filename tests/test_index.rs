extern crate sourcemap;

use std::collections::HashMap;
use sourcemap::SourceMapIndex;

#[test]
fn test_basic_indexed_sourcemap() {
    let input: &[_] = br#"{
        "version": 3,
        "file": "min.js",
        "sections": [
            {
                "offset": {
                    "line": 0,
                    "column": 0
                },
                "map": {
                    "version":3,
                    "sources":["file1.js"],
                    "names":["add","a","b"],
                    "mappings":"AAAA,QAASA,KAAIC,EAAGC,GACf,YACA,OAAOD,GAAIC",
                    "file":"file1.min.js"
                }
            },
            {
                "offset": {
                    "line": 1,
                    "column": 0
                },
                "map": {
                    "version":3,
                    "sources":["file2.js"],
                    "names":["multiply","a","b","divide","add","c","e","Raven","captureException"],
                    "mappings":"AAAA,QAASA,UAASC,EAAGC,GACpB,YACA,OAAOD,GAAIC,EAEZ,QAASC,QAAOF,EAAGC,GAClB,YACA,KACC,MAAOF,UAASI,IAAIH,EAAGC,GAAID,EAAGC,GAAKG,EAClC,MAAOC,GACRC,MAAMC,iBAAiBF",
                    "file":"file2.min.js"
                }
            }
        ]
    }"#;
    let file1 : Vec<&'static str> = r#"function add(a, b) {
 "use strict";
 return a + b; // fôo
}
"#.lines().collect();
    let file2 : Vec<&'static str> = r#"function multiply(a, b) {
 "use strict";
 return a * b;
}
function divide(a, b) {
 "use strict";
 try {
  return multiply(add(a, b), a, b) / c;
 } catch (e) {
  Raven.captureException(e);
 }
}
"#.lines().collect();

    println!("{:?}", file1);
    let mut files = HashMap::new();
    files.insert("file1.js", file1);
    files.insert("file2.js", file2);

    let ism = SourceMapIndex::from_reader(input).unwrap();
    let flat_map = ism.flatten().unwrap();

    for token in flat_map.tokens() {
        let src = files.get(token.get_source().unwrap()).unwrap();
        println!("{:?}", token);
        if let Some(name) = token.get_name() {
            let line = src[token.get_src_line() as usize];
            let idx = token.get_src_col() as usize;
            let span = &line[idx..idx + name.len()];
            assert_eq!(span, name);
        }
    }
}
