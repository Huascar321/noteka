use noteka_core::contracts::Response;
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{Emitter, Manager};

struct Pending(AtomicBool);
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

pub fn run() {
    let builder = tauri::Builder::default()
        .manage(Pending(AtomicBool::new(false)))
        .invoke_handler(tauri::generate_handler![
            operate,
            pending_edits,
            close_confirmed
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
