use deno_core::futures::FutureExt;
use deno_core::PollEventLoopOptions;
use deno_core::{JsRuntime, RuntimeOptions};
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

pub fn execute_ast(
    ast: Module,
) -> impl std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send>>> + Send {
    let source_code = ast_to_source_code(ast);

    tokio::task::spawn_blocking(move || {
        // Create a new JsRuntime with the custom extension.
        let mut js_runtime = JsRuntime::new(RuntimeOptions {
            ..Default::default()
        });

        // Execute the JavaScript code.
        js_runtime.execute_script("<usage>", source_code).unwrap();

        // Run the event loop to completion.
        #[allow(clippy::let_underscore_future)]
        let _ = js_runtime.run_event_loop(PollEventLoopOptions::default());

        Ok(())
    })
    .map(|res| {
        res.unwrap_or_else(|join_error| {
            Err(Box::new(join_error) as Box<dyn std::error::Error + Send>)
        })
    })
}
