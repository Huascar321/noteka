use noteka_core::contracts::Response;
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{Emitter, Manager};

struct Pending(AtomicBool);
#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn noteka_current_location(context: *mut std::ffi::c_void, callback: extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_char));
}

#[tauri::command]
async fn weather_current_location() -> Result<Value, String> {
    #[cfg(target_os = "macos")]
    {
        tauri::async_runtime::spawn_blocking(|| {
            extern "C" fn callback(context: *mut std::ffi::c_void, json: *const std::ffi::c_char) {
                let sender = unsafe { Box::from_raw(context.cast::<std::sync::mpsc::Sender<String>>()) };
                let result = if json.is_null() { String::new() } else { unsafe { std::ffi::CStr::from_ptr(json) }.to_string_lossy().into_owned() };
                let _ = sender.send(result);
            }
            let (sender, receiver) = std::sync::mpsc::channel::<String>();
            unsafe { noteka_current_location(Box::into_raw(Box::new(sender)).cast(), callback); }
            let raw = receiver.recv().map_err(|_| "Location unavailable.".to_owned())?;
            let value: Value = serde_json::from_str(&raw).map_err(|_| "Location unavailable.".to_owned())?;
            if let Some(error) = value.get("error").and_then(Value::as_str) { return Err(error.to_owned()); }
            Ok(value)
        }).await.map_err(|_| "Location unavailable.".to_owned())?
    }
    #[cfg(not(target_os = "macos"))]
    { Err("Current location is not available on this platform.".to_owned()) }
}
#[cfg(feature = "native-smoke")]
mod smoke;
#[tauri::command]
async fn operate(request: Value) -> Response {
    tauri::async_runtime::spawn_blocking(move || {
        #[cfg(feature = "native-smoke")]
        if let Some(response) = smoke::read_instrumentation(&request) {
            return response;
        }
        noteka_core::execute_json(request, "desktop")
    })
    .await
    .unwrap_or_else(|_| {
        noteka_core::error_response(noteka_core::error::CoreError::new(
            noteka_core::error::ErrorCode::StorageError,
            "La operación no pudo completarse.",
        ))
    })
}
#[tauri::command]
fn pending_edits(state: tauri::State<'_, Pending>, pending: bool) {
    state.0.store(pending, Ordering::SeqCst);
}
#[tauri::command]
fn close_confirmed(app: tauri::AppHandle) {
    app.state::<Pending>().0.store(false, Ordering::SeqCst);
    app.exit(0);
}

#[tauri::command]
async fn google_pick() -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(|| {
        noteka_providers::google::pick(&noteka_providers::KeyringStore)
            .map_err(|error| format!("{}: {}", error.code, error.message))
    })
    .await
    .map_err(|_| "El Picker no pudo completarse.".to_owned())?
}

#[tauri::command]
async fn google_preview(file_id: String) -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        noteka_providers::google::preview(&noteka_providers::KeyringStore, &file_id)
            .map_err(|error| format!("{}: {}", error.code, error.message))
    })
    .await
    .map_err(|_| "La previsualización no pudo completarse.".to_owned())?
}

fn external_url_allowed(url: &str) -> bool {
    if url == "https://open-meteo.com/" {
        return true;
    }
    [
        "https://docs.google.com/",
        "https://drive.google.com/",
        "https://sheets.google.com/",
    ]
    .iter()
    .any(|prefix| url.starts_with(prefix))
}

#[tauri::command]
fn open_external(url: String) -> Result<(), String> {
    if !external_url_allowed(&url) {
        return Err("URL externa no permitida.".into());
    }
    #[cfg(target_os = "macos")]
    let (program, args) = ("open", vec![url.as_str()]);
    #[cfg(target_os = "windows")]
    let (program, args) = ("cmd", vec!["/C", "start", "", url.as_str()]);
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let (program, args) = ("xdg-open", vec![url.as_str()]);
    std::process::Command::new(program)
        .args(args)
        .status()
        .map_err(|error| format!("No se pudo abrir el navegador: {error}"))?;
    Ok(())
}

pub fn run() {
    let builder = tauri::Builder::default()
        .manage(Pending(AtomicBool::new(false)))
        .invoke_handler(tauri::generate_handler![
            operate,
            pending_edits,
            close_confirmed,
            google_pick,
            google_preview,
            weather_current_location,
            open_external
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event
                && window.state::<Pending>().0.load(Ordering::SeqCst)
            {
                api.prevent_close();
                let _ = window.emit("noteka-close-requested", ());
            }
        });
    #[cfg(feature = "native-smoke")]
    let builder = builder
        .invoke_handler(tauri::generate_handler![
            operate,
            pending_edits,
            close_confirmed,
            google_pick,
            google_preview,
            weather_current_location,
            open_external,
            smoke::smoke_control
        ])
        .on_page_load(|webview, payload| {
            if payload.event() == tauri::webview::PageLoadEvent::Finished {
                // El foco del NSWindow y del WKWebView son precondiciones distintas.
                // Fuera del callback para no bloquear el hilo de eventos al enfocar.
                let webview = webview.clone();
                std::thread::spawn(move || {
                    webview
                        .window()
                        .set_focus()
                        .expect("Foco de ventana de prueba");
                    webview.set_focus().expect("Foco de webview de prueba");
                    webview
                        .eval(include_str!("../../tests/native/home.js"))
                        .expect("Helpers Home de prueba");
                    webview
                        .eval(include_str!("../../tests/native/smoke.js"))
                        .expect("Recorrido de prueba");
                });
            }
        });
    let app = builder
        .build(tauri::generate_context!())
        .expect("No se pudo iniciar Noteka");
    app.run(|app, event| {
        if let tauri::RunEvent::ExitRequested { api, .. } = event
            && app.state::<Pending>().0.load(Ordering::SeqCst)
        {
            api.prevent_exit();
            let _ = app.emit("noteka-close-requested", ());
        }
    });
}

#[cfg(test)]
mod tests {
    use super::external_url_allowed;

    #[test]
    fn external_urls_are_limited_to_allowed_destinations() {
        assert!(external_url_allowed(
            "https://docs.google.com/spreadsheets/d/x"
        ));
        assert!(external_url_allowed("https://drive.google.com/file/d/x"));
        assert!(external_url_allowed(
            "https://sheets.google.com/spreadsheets/d/x"
        ));
        assert!(!external_url_allowed("https://example.com/"));
        assert!(!external_url_allowed("http://docs.google.com/"));
        assert!(external_url_allowed("https://open-meteo.com/"));
        assert!(!external_url_allowed("https://open-meteo.com.evil.example/"));
    }
}
