extern crate sourcemap;

use sourcemap::SourceMap;


#[test]
fn test_basic_name_mapping() {
    let input = r#"{"version":3,"file":"test.min.js","sources":["test.js"],"names":["makeAFailure","testingStuff","Error","onSuccess","data","onFailure","invoke","cb","failed","test","value"],"mappings":"AAAA,GAAIA,cAAe,WACjB,QAASC,KACP,GAAIA,GAAe,EACnB,MAAM,IAAIC,OAAMD,GAGlB,QAASE,GAAUC,GACjBH,IAGF,QAASI,GAAUD,GACjB,KAAM,IAAIF,OAAM,WAGlB,QAASI,GAAOF,GACd,GAAIG,GAAK,IACT,IAAIH,EAAKI,OAAQ,CACfD,EAAKF,MACA,CACLE,EAAKJ,EAEPI,EAAGH,GAGL,QAASK,KACP,GAAIL,IAAQI,OAAQ,KAAME,MAAO,GACjCJ,GAAOF,GAGT,MAAOK","sourcesContent":["var makeAFailure = (function() {\n  function testingStuff() {\n    var testingStuff = 42;\n    throw new Error(testingStuff);\n  }\n\n  function onSuccess(data) {\n    testingStuff();\n  }\n\n  function onFailure(data) {\n    throw new Error('failed!');\n  }\n\n  function invoke(data) {\n    var cb = null;\n    if (data.failed) {\n      cb = onFailure;\n    } else {\n      cb = onSuccess;\n    }\n    cb(data);\n  }\n\n  function test() {\n    var data = {failed: true, value: 42};\n    invoke(data);\n  }\n\n  return test;\n})();\n"]}"#.as_bytes();
    let minified_file = r#"var makeAFailure=function(){function n(){var n=42;throw new Error(n)}function r(r){n()}function e(n){throw new Error("failed!")}function i(n){var i=null;if(n.failed){i=e}else{i=r}i(n)}function u(){var n={failed:true,value:42};i(n)}return u}();"#;
    let sm = SourceMap::from_reader(input).unwrap();

    let tok = sm.lookup_token(0, 45).unwrap();
    assert_eq!(tok.get_name(), Some("testingStuff"));
    assert_eq!(tok.get_minified_name(minified_file), Some("n"));

    let tok = sm.lookup_token(0, 66).unwrap();
    assert_eq!(tok.get_name(), Some("testingStuff"));
    assert_eq!(tok.get_minified_name(minified_file), Some("n"));

    let tok = sm.lookup_token(0, 96).unwrap();
    assert_eq!(tok.get_name(), Some("onFailure"));
    assert_eq!(tok.get_minified_name(minified_file), Some("e"));

    let name = sm.get_original_function_name(0, 107, "e", minified_file);
    assert_eq!(name, Some("onFailure"));


    // a stacktrae
    let locs = &[
        (0, 107, "e", "onFailure"),
        (0, 179, "i", "invoke"),
        (0, 226, "u", "test"),
    ];

    for &(line, col, minified_name, original_name) in locs {
        let name = sm.get_original_function_name(line, col, minified_name, minified_file);
        assert_eq!(name, Some(original_name));
    }
}
