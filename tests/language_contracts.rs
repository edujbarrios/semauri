use semauri::Compiler;

#[test]
fn lexer_comments_escapes_and_invalid_escapes_match_contract() {
    let compiler = Compiler::new();
    let tokens = compiler
        .tokenize("# ignored\nLet message be \"line\\nnext\".")
        .unwrap();
    assert_eq!(tokens[0].lexeme.to_lowercase(), "let");

    let ast = compiler
        .parse("Let message be \"tab\\tquote\\\"slash\\\\\". Create a web.")
        .unwrap()
        .to_json();
    assert_eq!(
        ast["statements"][0]["value"]["value"],
        "tab\tquote\"slash\\"
    );

    let error = compiler.parse("Let message be \"bad\\q\".").unwrap_err();
    assert_eq!(error.code, "S104");
}

#[test]
fn make_artifact_synonym_and_explicit_reference_work() {
    let compiler = Compiler::new();
    let output = compiler
        .compile(
            "Make a web called Shop. Add a button called Buy. Add an image called Logo. Set the color of the button called Buy to blue.",
            None,
        )
        .unwrap()
        .output
        .unwrap();
    assert!(output.contains("<title>Shop</title>"));
    assert!(output.contains("color: blue"));
}

#[test]
fn reference_errors_keep_diagnostic_codes() {
    let compiler = Compiler::new();

    let no_referent = compiler.analyze("Create a web. Make it red.").unwrap_err();
    assert_eq!(no_referent.code, "S304");

    let missing = compiler
        .analyze("Create a web. Add a button called Buy. Set the color of the button called Missing to red.")
        .unwrap_err();
    assert_eq!(missing.code, "S309");
}

#[test]
fn arithmetic_precedence_parentheses_and_otherwise_are_deterministic() {
    let compiler = Compiler::new();

    let first = compiler
        .compile(
            "Create a web. If 2 plus 3 times 4 is equal to 14: Add a button called Yes. Otherwise: Add a button called No. End.",
            None,
        )
        .unwrap()
        .output
        .unwrap();
    assert!(first.contains(">Yes</button>"));

    let second = compiler
        .compile(
            "Create a web. If (2 plus 3) times 4 is equal to 20: Add a button called Grouped. Otherwise: Add a button called No. End.",
            None,
        )
        .unwrap()
        .output
        .unwrap();
    assert!(second.contains(">Grouped</button>"));
}

#[test]
fn scope_and_type_errors_remain_stable() {
    let compiler = Compiler::new();
    let cases = [
        ("Create a web. If 1: Add a button. End.", "S316"),
        ("Let x be red plus 1. Create a web.", "S314"),
        ("Let x be 1 is equal to true. Create a web.", "S315"),
        ("Let x be 1 divided by 0. Create a web.", "S317"),
        ("Let x be 1. Let x be 2. Create a web.", "S311"),
        (
            "Create a web. If true: Let local be 1. End. Let x be local.",
            "S312",
        ),
    ];
    for (source, code) in cases {
        let error = compiler.analyze(source).unwrap_err();
        assert_eq!(error.code, code, "{source}");
    }
}

#[test]
fn inclusive_and_logical_operators_keep_precedence() {
    let compiler = Compiler::new();
    let output = compiler
        .compile(
            "Create a web. If 5 is greater than or equal to 5 and not false or false: Add a button called Boundary. End.",
            None,
        )
        .unwrap()
        .output
        .unwrap();
    assert!(output.contains(">Boundary</button>"));

    let error = compiler
        .analyze("Create a web. If true or missing is equal to true: Add a button. End.")
        .unwrap_err();
    assert_eq!(error.code, "S312");
}

#[test]
fn ast_keeps_explicit_unary_binary_and_block_nodes() {
    let ast = Compiler::new()
        .parse("Create a web. If not (1 plus 1 is equal to 3): Add a button called Safe. End.")
        .unwrap()
        .to_json()
        .to_string();
    assert!(ast.contains("unary_expression"));
    assert!(ast.contains("binary_expression"));
    assert!(ast.contains("\"type\":\"block\""));
}

#[test]
fn not_equal_comparisons_are_typed_and_deterministic() {
    let compiler = Compiler::new();
    let output = compiler
        .compile(
            "Create a web. If 2 plus 2 is not equal to 5: Add a button called Correct. Otherwise: Add a button called Wrong. End.",
            None,
        )
        .unwrap()
        .output
        .unwrap();
    assert!(output.contains(">Correct</button>"));

    let error = compiler
        .analyze("Create a web. If 1 is not equal to true: Add a button. End.")
        .unwrap_err();
    assert_eq!(error.code, "S315");
}

#[test]
fn unary_minus_is_numeric_and_binds_tighter_than_multiplication() {
    let compiler = Compiler::new();

    let output = compiler
        .compile(
            "Create a web. If 2 times minus 3 plus 11 is equal to 5: Add a button called Negative. End.",
            None,
        )
        .unwrap()
        .output
        .unwrap();
    assert!(output.contains(">Negative</button>"));

    let ast = compiler.parse("Let debt be minus 5. Create a web.").unwrap().to_json();
    assert_eq!(ast["statements"][0]["value"]["operator"], "negate");

    let error = compiler
        .analyze("Let invalid be minus true. Create a web.")
        .unwrap_err();
    assert_eq!(error.code, "S314");
}

#[test]
fn modulo_is_a_typed_multiplicative_operator() {
    let compiler = Compiler::new();

    let output = compiler
        .compile(
            "Create a web. If 10 modulo 3 is equal to 1: Add a button called Remainder. End.",
            None,
        )
        .unwrap()
        .output
        .unwrap();
    assert!(output.contains(">Remainder</button>"));

    let output = compiler
        .compile(
            "Create a web. If 2 plus 10 modulo 4 times 3 is equal to 8: Add a button called Precedence. End.",
            None,
        )
        .unwrap()
        .output
        .unwrap();
    assert!(output.contains(">Precedence</button>"));

    let error = compiler
        .analyze("Let invalid be 10 modulo 0. Create a web.")
        .unwrap_err();
    assert_eq!(error.code, "S317");
}

#[test]
fn plus_concatenates_strings_without_cross_type_coercion() {
    let compiler = Compiler::new();

    let output = compiler
        .compile(
            "Create a web. If \"Hello, \" plus \"Semauri\" is equal to \"Hello, Semauri\": Add a button called Greeting. End.",
            None,
        )
        .unwrap()
        .output
        .unwrap();
    assert!(output.contains(">Greeting</button>"));

    let optimized = compiler
        .optimized_hir("Let message be \"Hello\" plus \" world\". Print message.")
        .unwrap()
        .to_json()
        .to_string();
    assert!(optimized.contains("Hello world"));

    let error = compiler
        .analyze("Let invalid be 1 plus \"item\". Create a web.")
        .unwrap_err();
    assert_eq!(error.code, "S314");
}

#[test]
fn hash_slash_and_nested_block_comments_are_ignored() {
    let compiler = Compiler::new();
    let source = "// file heading\nLet count be 2. /* outer\n /* inner */ still outer */\n# another comment\nPrint count.";
    let tokens = compiler.tokenize(source).unwrap();
    assert_eq!(tokens[0].lexeme.to_lowercase(), "let");
    assert_eq!(tokens[0].to_json()["line"], 2);
    assert!(tokens.iter().any(|token| token.lexeme.to_lowercase() == "print"));

    compiler
        .analyze("Let text be \"// not a comment; /* also text */\". Print text.")
        .unwrap();

    let error = compiler.tokenize("Let count be 2. /* unclosed").unwrap_err();
    assert_eq!(error.code, "S105");
}
