#[allow(unused)]
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use deno_core::ByteString;
use deno_core::Extension;
use deno_core::JsRuntime;
use deno_core::ModuleLoadResponse;
use deno_core::ModuleLoader;
use deno_core::ModuleSource;
use deno_core::ModuleSpecifier;
use deno_core::ModuleType;
use deno_core::OpState;
use deno_core::PollEventLoopOptions;
use deno_core::RuntimeOptions;
use deno_core::anyhow::anyhow;
use deno_core::error::AnyError;
use deno_core::error::ModuleLoaderError;
use deno_core::op2;
use deno_core::url::Url;
use futures::FutureExt;
use deno_core::RequestedModuleType;
use deno_core::ModuleSourceCode;

#[op2(fast)]
// I had to declare it as core error because somehow
// anyerror is not in the traitbound
fn op_log(#[string] msg: String) -> Result<(), deno_core::error::CoreError> {
    println!("[MODULE.log] {msg}");
    Ok(())
}

#[op2(async)]
async fn op_sleep(
    #[bigint] duration_in_ms: u64,
    _state: Rc<RefCell<OpState>>,
) -> Result<(), deno_core::error::CoreError> {
    println!("[ZINNIA] going to sleep for {duration_in_ms}ms");
    tokio::time::sleep(Duration::from_millis(duration_in_ms)).await;
    println!("[ZINNIA] waking up");
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    println!("== Running the JavaScript demo ==");
    run_module(&Url::parse("zinnia://demo-module.js").unwrap())
        .await
        .unwrap();

    println!("== Running the Rust/WASM demo ==");

    run_module(&Url::parse("zinnia://demo-module-rs.js").unwrap())
        .await
        .unwrap();

    println!("== DONE ==");
}

async fn run_module(specifier: &ModuleSpecifier) -> Result<(), deno_core::anyhow::Error> {

    let ext2 = Extension::default();
    let ops = vec![]
    // Build a deno_core::Extension providing custom ops
    let ext = Extension::builder("zinnia")
        .ops(vec![
            // An op for summing an array of numbers
            // The op-layer automatically deserializes inputs
            // and serializes the returned Result & value
            op_log::decl(),
            op_sleep::decl(),
            op_base64_atob::decl(),
        ])
        .build();

    // Initialize a runtime instance
    let mut runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![ext],
        inspector: false,
        module_loader: Some(Rc::new(ZinniaModuleLoader)),
        ..Default::default()
    });

    // Running a script that's not an ES module
    //
    // runtime
    //   .execute_script("demo-module.js", include_str!("../mod-js/demo-module.js"))
    //   .unwrap();
    // runtime.run_event_loop(false).await.unwrap();

    // Load TextEncoder and TextDecoder APIs
    runtime
        .execute_script(
            "zinnia://text-encoding.js",
            include_str!("../src/text-encoding.js"),
        )
        .unwrap();

    // Create global `atob` function
    runtime
        .execute_script(
            "zinnia://atob.js",
            r#"
  globalThis.atob = function(str) { return Deno.core.ops.op_base64_atob(str); };
  "#,
        )
        .unwrap();

    // Enable Async Ops
    runtime
        .execute_script(
            "zinnia://enable-async-ops.js",
            "Deno.core.initializeAsyncOps()",
        )
        .unwrap();

    let main_module_id = runtime.load_main_es_module(specifier).await?;

    // println!("evaluating the demo module");
    let res = runtime.mod_evaluate(main_module_id);

    // println!("running the event loop");
    let polloptions = PollEventLoopOptions::default();
    runtime.run_event_loop(polloptions).await?;
    // ** to check if poll options should be default or should be set to both false ? ** 
    // println!("awaiting module evaluation result");
    res.await;

    Ok(())
}

// #[op2(fast)]
// fn op_base64_atob(mut s: ByteString) -> Result<ByteString, AnyError> {
//   let decoded_len = forgiving_base64_decode_inplace(&mut s)?;
//   s.truncate(decoded_len);
//   Ok(s)
// }


#[inline]
fn forgiving_base64_decode_inplace(input: &mut [u8]) -> Result<usize, AnyError> {
    let error: _ = || anyhow!("Failed to decode base64");
    let decoded = base64_simd::forgiving_decode_inplace(input).map_err(|_| error())?;
    Ok(decoded.len())
}
/// Our custom module loader.
pub struct ZinniaModuleLoader;

// Error Resolving
impl ModuleLoader for ZinniaModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        _referrer: &str,
        _kind: deno_core::ResolutionKind,
    ) -> Result<deno_core::ModuleSpecifier, ModuleLoaderError> {
        match specifier {
            "zinnia://demo-module.js" => Ok(Url::parse(specifier).unwrap()),
            "zinnia://demo-module-rs.js" => Ok(Url::parse(specifier).unwrap()),
            "zinnia://demo-module-rs.loader.js" => Ok(Url::parse(specifier).unwrap()),
            _ => Err(ModuleLoaderError::NotFound),
        }
    }

    fn load(
        &self,
        module_specifier: &deno_core::ModuleSpecifier,
        _maybe_referrer: Option<&ModuleSpecifier>,
        is_dyn_import: bool,
        requested_module_type: RequestedModuleType,
    ) -> ModuleLoadResponse {
        let specifier = String::from(module_specifier.as_str());
        async move {
            if is_dyn_import {
                return Err(anyhow!(
                    "Zinnia does not support dynamic imports. (URL: {})",
                    specifier
                ));
            }

            let code = {
                match specifier.as_str() {
                    "zinnia://demo-module.js" => include_str!("../mod-js/demo-module.js"),
                    "zinnia://demo-module-rs.js" => {
                        r#"
import {instantiate} from 'zinnia://demo-module-rs.loader.js';

// Zinnia SDK
globalThis.Zinnia = {
  log(msg) {
    console.trace('log')
    Deno.core.ops.op_log(msg)
  },

  async sleep(durationInMs) {
    console.trace('sleep')
    return Deno.core.ops.op_sleep(durationInMs);
  },
};

// Run the WASM module
const {run} = await instantiate();
await run();
"#
                    }
                    "zinnia://demo-module-rs.loader.js" => {
                        include_str!("../target/deno/mod_rs.generated.js")
                    }
                    _ => Err(ModuleLoaderError::NotFound),
                }
            };

            let module = ModuleSource::new(ModuleType::JavaScript,ModuleSourceCode::Bytes(code),module_specifier,None) ;
            Ok(module)
        }
        .boxed_local();
    }
}
