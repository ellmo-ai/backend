use swc_common::input::StringInput;
use swc_common::BytePos;
use swc_ecma_parser::{Parser, Syntax};

pub fn parse_js_to_ast(js_code: &str) -> swc_ecma_ast::Module {
    let parse_options = swc_ecma_parser::TsConfig {
        tsx: false,
        decorators: true,
        ..Default::default()
    };

    // Parse the input JavaScript code into an AST
    let lexer = swc_ecma_parser::lexer::Lexer::new(
        Syntax::Typescript(parse_options),
        Default::default(),
        StringInput::new(js_code, BytePos(0), BytePos(0)),
        None,
    );
    let mut parser = Parser::new_from(lexer);

    parser.parse_module().expect("Failed to parse module")
}
