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

    let f1 : Vec<_> = "function add(a, b) {\n \"use strict\";\n \
                       return a + b; // fôo\n}".lines().collect();
    let f2 : Vec<_> = "function multiply(a, b) {\n \"use strict\";\n \
                       return a * b;\n}\nfunction divide(a, b) {\n \
                       \"use strict\";\n try {\n  return multiply(add(a, \
                       b), a, b) / c;\n } catch (e) {\n  \
                       Raven.captureException(e);\n }\n}".lines().collect();

    let mut files = HashMap::new();
    files.insert("file1.js", f1);
    files.insert("file2.js", f2);

    let ism = SourceMapIndex::from_reader(input).unwrap();
    let flat_map = ism.flatten().unwrap();

    for token in flat_map.tokens() {
        let src = files.get(token.get_source().unwrap()).unwrap();
        if let Some(name) = token.get_name() {
            let line = src[token.get_src_line() as usize];
            let idx = token.get_src_col() as usize;
            let span = &line[idx..idx + name.len()];
            assert_eq!(span, name);
        }
    }
}
