use deno_core::futures::FutureExt;
use deno_core::JsRuntime;
use deno_core::ModuleSpecifier;
use deno_core::RuntimeOptions;
use deno_core::{FsModuleLoader, PollEventLoopOptions};
use std::str::FromStr;
use std::sync::Arc;
use swc_ecma_ast::Module;
use swc_ecma_codegen::{text_writer::JsWriter, Emitter};

fn ast_to_source_code(ast: Module) -> String {
    let cm = Arc::new(swc_common::SourceMap::default());
    let mut buf = Vec::new();
    {
        let writer = Box::new(JsWriter::new(cm.clone(), "\n", &mut buf, None));
        let mut emitter = Emitter {
            cfg: swc_ecma_codegen::Config::default(),
            cm: cm.clone(),
            comments: None,
            wr: writer,
        };
        emitter.emit_module(&ast).unwrap();
    }
    String::from_utf8(buf).unwrap()
}

pub async fn execute_ast(ast: Module) -> Result<(), Box<dyn std::error::Error>> {
    // Convert AST to source code
    let source_code = ast_to_source_code(ast);

    // Create a runtime
    let module_loader = Arc::new(FsModuleLoader);
    let options = RuntimeOptions {
        module_loader: Some(module_loader.into()),
        ..Default::default()
    };

    // Load the source code as a module
    let mut runtime = JsRuntime::new(options);
    let module_specifier = ModuleSpecifier::from_str("file:///main.js").unwrap();

    let main_module_id = runtime
        .load_main_es_module_from_code(&module_specifier, source_code.clone())
        .await
        .unwrap();

    runtime.execute_script("main.js", source_code).unwrap();

    // Evaluate the main module
    let result = runtime.mod_evaluate(main_module_id).boxed_local();

    runtime
        .run_event_loop(PollEventLoopOptions::default())
        .await
        .unwrap();

    result.await.unwrap();

    Ok(())
}
