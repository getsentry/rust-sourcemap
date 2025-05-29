use sourcemap::{SourceMap, SourceView};

#[test]
fn test_basic_name_mapping() {
    let input: &[_] = br#"{"version":3,"file":"test.min.js","sources":["test.js"],"names":["makeAFailure","testingStuff","Error","onSuccess","data","onFailure","invoke","cb","failed","test","value"],"mappings":"AAAA,GAAIA,cAAe,WACjB,QAASC,KACP,GAAIA,GAAe,EACnB,MAAM,IAAIC,OAAMD,GAGlB,QAASE,GAAUC,GACjBH,IAGF,QAASI,GAAUD,GACjB,KAAM,IAAIF,OAAM,WAGlB,QAASI,GAAOF,GACd,GAAIG,GAAK,IACT,IAAIH,EAAKI,OAAQ,CACfD,EAAKF,MACA,CACLE,EAAKJ,EAEPI,EAAGH,GAGL,QAASK,KACP,GAAIL,IAAQI,OAAQ,KAAME,MAAO,GACjCJ,GAAOF,GAGT,MAAOK","sourcesContent":["var makeAFailure = (function() {\n  function testingStuff() {\n    var testingStuff = 42;\n    throw new Error(testingStuff);\n  }\n\n  function onSuccess(data) {\n    testingStuff();\n  }\n\n  function onFailure(data) {\n    throw new Error('failed!');\n  }\n\n  function invoke(data) {\n    var cb = null;\n    if (data.failed) {\n      cb = onFailure;\n    } else {\n      cb = onSuccess;\n    }\n    cb(data);\n  }\n\n  function test() {\n    var data = {failed: true, value: 42};\n    invoke(data);\n  }\n\n  return test;\n})();\n"]}"#;
    let minified_file = r#"var makeAFailure=function(){function n(){var n=42;throw new Error(n)}function r(r){n()}function e(n){throw new Error("failed!")}function i(n){var i=null;if(n.failed){i=e}else{i=r}i(n)}function u(){var n={failed:true,value:42};i(n)}return u}();"#;
    let sv = SourceView::new(minified_file.into());
    let sm = SourceMap::from_reader(input).unwrap();

    let name = sm.get_original_function_name(0, 107, "e", &sv);
    assert_eq!(name, Some("onFailure"));

    // a stacktrace
    let locs = &[
        (0, 107, "e", "onFailure"),
        (0, 179, "i", "invoke"),
        (0, 226, "u", "test"),
    ];

    for &(line, col, minified_name, original_name) in locs {
        let name = sm.get_original_function_name(line, col, minified_name, &sv);
        assert_eq!(name, Some(original_name));
    }
}

#[test]
fn test_unicode_mapping() {
    let input = r#"{"version":3,"file":"test.min.js","sources":["test.js"],"names":["makeAFailure","onSuccess","data","onFailure","Error","invoke","cb","failed","㮏","value"],"mappings":"AAAA,GAAIA,cAAe,WACjB,QAASC,GAAUC,IAEnB,QAASC,GAAUD,IACjB,WACE,KAAM,IAAIE,OAAM,eAIpB,QAASC,GAAOH,GACd,GAAII,GAAK,IACT,IAAIJ,EAAKK,OAAQ,CACfD,EAAKH,MACA,CACLG,EAAKL,EAEPK,EAAGJ,GAGI,QAASM,KAChB,GAAIN,IAAQK,OAAQ,KAAME,MAAO,GACjCJ,GAAOH,GAGT,MAAOM","sourcesContent":["var makeAFailure = (function() {\n  function onSuccess(data) {}\n\n  function onFailure(data) {\n    (function() {\n      throw new Error('failed!');\n    })();\n  }\n\n  function invoke(data) {\n    var cb = null;\n    if (data.failed) {\n      cb = onFailure;\n    } else {\n      cb = onSuccess;\n    }\n    cb(data);\n  }\n\n  /* 😍 */ function 㮏() {\n    var data = {failed: true, value: 42};\n    invoke(data);\n  }\n\n  return 㮏;\n})();\n"]}"#.as_bytes();
    let minified_file = r#"var makeAFailure=function(){function n(n){}function e(n){(function(){throw new Error("failed!")})()}function i(i){var r=null;if(i.failed){r=e}else{r=n}r(i)}function r(){var n={failed:true,value:42};i(n)}return r}();"#;
    let sv = SourceView::new(minified_file.into());
    let sm = SourceMap::from_reader(input).unwrap();

    // stacktrace
    let locs = &[
        (0, 75, "<unknown>", None),
        (0, 97, "e", Some("onFailure")),
        (0, 151, "i", Some("invoke")),
        (0, 198, "r", Some("㮏")),
    ];

    for &(line, col, minified_name, original_name_match) in locs {
        let name = sm.get_original_function_name(line, col, minified_name, &sv);
        assert_eq!(name, original_name_match);
    }
}

#[test]
fn test_lambda_function_name_mapping() {
    let input: &[_] = br#"{"version":3,"sources":["source.js"],"sourcesContent":["const SOME_CONST = 3;\n\nfunction outer() {\n    const aFunctionAsConst = function() {\n        console.log(\"A function as const\");\n        aFunctionAsConst();\n    };\n\n    const aLambdaAsConst = () => {\n        console.log(\"A lambda as const\");\n        aLambdaAsConst();\n    };\n\n    function aRegularFunction() {\n        console.log(\"A regular function\");\n        aRegularFunction();\n    }\n\n    aFunctionAsConst();\n    aLambdaAsConst();\n    aRegularFunction();\n}\n\nouter();\n"],"names":["SOME_CONST","outer","aFunctionAsConst","console","log","aLambdaAsConst","aRegularFunction"],"mappings":"AAAA,IAAMA,WAAa,EAEnB,SAASC,QACoB,SAAnBC,IACFC,QAAQC,IAAI,qBAAqB,EACjCF,EAAiB,CACrB,CAHA,IAKMG,EAAiB,KACnBF,QAAQC,IAAI,mBAAmB,EAC/BC,EAAe,CACnB,EAOAH,EAAiB,EACjBG,EAAe,EANf,SAASC,IACLH,QAAQC,IAAI,oBAAoB,EAChCE,EAAiB,CACrB,EAIiB,CACrB,CAEAL,MAAM"}"#;
    let minified_file = r#"let SOME_CONST=3;function outer(){function o(){console.log("A function as const"),o()}let n=()=>{console.log("A lambda as const"),n()};o(),n(),function o(){console.log("A regular function"),o()}()}outer();"#;
    let sv = SourceView::new(minified_file.into());
    let sm = SourceMap::from_reader(input).unwrap();

    // a stacktrace
    let locs = &[
        (0, 56, "o", "aFunctionAsConst"),
        (0, 106, "n", "aLambdaAsConst"),
        (0, 165, "o", "aRegularFunction"),
    ];

    for &(line, col, minified_name, original_name) in locs {
        let name = sm.get_original_function_name(line, col, minified_name, &sv);
        assert_eq!(name, Some(original_name));
    }
}
