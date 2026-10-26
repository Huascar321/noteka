use noteka_providers::{KeyringStore, ProviderError, api, google, openai, proxy};
use serde_json::{Value, json};

fn fail(error: ProviderError) -> ! {
    let code = match error.code {
        "validation_error" => 2,
        "auth_required" | "auth_timeout" | "access_denied" | "state_mismatch" => 4,
        _ => 5,
    };
    println!(
        "{}",
        json!({"ok":false,"error":{"code":error.code,"message":error.message}})
    );
    std::process::exit(code)
}
fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .cloned()
}
fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        println!(
            "{}",
            json!({"ok":false,"error":{"code":"validation_error","message":"Uso: noteka-auth openai|google <acción>"}})
        );
        std::process::exit(2);
    }
    let store = KeyringStore;
    let result: Result<Value, ProviderError> = match (args[0].as_str(), args.get(1).map(String::as_str)) {
        ("openai", Some("login")) => openai::login(&store).map(|s| {
            json!({"provider":"chatgpt","state":"conectado","subject":s.oauth.subject,"email":s.oauth.email,"scopes":s.oauth.scopes})
        }),
        ("openai", Some("status")) => openai::status(&store),
        ("openai", Some("logout")) => openai::logout(&store).map(|_| json!({"provider":"chatgpt","state":"sin_sesion"})),
        ("openai", Some("chat")) => {
            let model = arg(&args, "--model").ok_or_else(|| ProviderError::validation("--model requerido"));
            match model {
                Ok(model) => {
                    let prompt = arg(&args, "--prompt").ok_or_else(|| ProviderError::validation("--prompt requerido"));
                    match prompt {
                        Ok(prompt) => {
                            let max = arg(&args, "--max-tokens").and_then(|v| v.parse::<u32>().ok()).unwrap_or(512);
                            openai::chat(&store, &model, &prompt, max)
                                .map(|text| json!({"provider":"chatgpt","text":text}))
                        }
                        Err(e) => Err(e),
                    }
                }
                Err(e) => Err(e),
            }
        }
        ("google", Some("login")) => google::login(&store).map(|s| {
            json!({"provider":"google","state":"conectado","subject":s.oauth.subject,"email":s.oauth.email,"scopes":s.oauth.scopes})
        }),
        ("google", Some("status")) => google::status(&store),
        ("google", Some("logout")) => google::logout(&store).map(|_| json!({"provider":"google","state":"sin_sesion"})),
        ("google", Some("pick")) => google::pick(&store),
        ("google", Some("selected")) => google::selected(&store),
        ("google", Some("preview")) => {
            let id = arg(&args, "--id").ok_or_else(|| ProviderError::validation("--id requerido"));
            match id {
                Ok(id) => google::preview(&store, &id),
                Err(e) => Err(e),
            }
        }
        ("google", Some("files")) => google::files(&store).map(|files| json!({"files":files})),
        ("key", Some("set")) => {
            let provider = args.get(2).cloned().unwrap_or_default();
            let mut key = String::new();
            match std::io::Read::read_to_string(&mut std::io::stdin(), &mut key) {
                Ok(_) => api::set_key(&store, &provider, &key).map(|_| json!({"provider":provider,"api_key_configured":true})),
                Err(e) => Err(ProviderError::new("storage_error", format!("No se pudo leer la key: {e}"))),
            }
        }
        ("key", Some("status")) => {
            let provider = args.get(2).cloned().unwrap_or_default();
            api::key_status(&store, &provider)
        }
        ("key", Some("delete")) => {
            let provider = args.get(2).cloned().unwrap_or_default();
            api::delete_key(&store, &provider).map(|_| json!({"provider":provider,"api_key_configured":false}))
        }
        ("serve", Some("chatgpt")) => {
            let port = arg(&args, "--port").and_then(|v| v.parse::<u16>().ok()).unwrap_or(11434);
            if port == 0 { Err(ProviderError::validation("--port inválido")) }
            else { proxy::serve(&store, port).map(|_| json!({"state":"detenido"})) }
        }
        ("provider", Some("chat")) => {
            let provider = arg(&args, "--provider").ok_or_else(|| ProviderError::validation("--provider requerido"));
            match provider {
                Ok(provider) => {
                    let model = arg(&args, "--model").ok_or_else(|| ProviderError::validation("--model requerido"));
                    match model {
                        Ok(model) => {
                            let prompt = arg(&args, "--prompt").ok_or_else(|| ProviderError::validation("--prompt requerido"));
                            match prompt {
                                Ok(prompt) => match provider.as_str() {
                                    "chatgpt" => {
                                        let max = arg(&args, "--max-tokens").and_then(|v| v.parse::<u32>().ok()).unwrap_or(512);
                                        openai::chat(&store, &model, &prompt, max).map(|text| {
                                            json!({"provider":"chatgpt","text":text})
                                        })
                                    }
                                    "openai" => {
                                        let max = arg(&args, "--max-tokens").and_then(|v| v.parse::<u32>().ok()).unwrap_or(512);
                                        api::openai_chat(&store, &model, &prompt, max).map(|text| {
                                            json!({"provider":"openai","text":text})
                                        })
                                    }
                                    "gemini" => api::gemini_chat(&store, &model, &prompt).map(|value| {
                                        json!({"provider":"gemini","text":api::gemini_text(&value),"raw":value})
                                    }),
                                    _ => Err(ProviderError::validation("Proveedor desconocido: usá chatgpt|openai|gemini")),
                                },
                                Err(e) => Err(e),
                            }
                        }
                        Err(e) => Err(e),
                    }
                }
                Err(e) => Err(e),
            }
        }
        _ => Err(ProviderError::validation("Uso: noteka-auth openai|google|key|provider <acción>")),
    };
    match result {
        Ok(data) => println!("{}", json!({"ok":true,"data":data})),
        Err(error) => fail(error),
    }
}
