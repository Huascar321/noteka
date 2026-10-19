//! Instrumentación exclusiva del feature de pruebas: nunca en el binario normal.
use serde_json::{Value, json};
use std::sync::Mutex;
use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};
use tauri::Manager;
static HOME_READ: Mutex<(String, u64, bool)> = Mutex::new((String::new(), 0, false));
pub fn read_instrumentation(request: &Value) -> Option<noteka_core::contracts::Response> {
    if request["command"]["operation"] != "home.query" {
        return None;
    }
    let (view, delay, failure) = HOME_READ.lock().unwrap().clone();
    if request["command"]["input"]["view"] != view {
        return None;
    }
    std::thread::sleep(std::time::Duration::from_millis(delay));
    failure.then(|| {
        noteka_core::error_response(noteka_core::error::CoreError::new(
            noteka_core::error::ErrorCode::StorageError,
            "Lectura Home fallida inyectada sólo en native-smoke.",
        ))
    })
}

fn root() -> PathBuf {
    PathBuf::from(std::env::var("NOTEKA_SMOKE_ROOT").expect("NOTEKA_SMOKE_ROOT requerido"))
}
fn workspace_path() -> PathBuf {
    root().join(
        if std::env::var("NOTEKA_SMOKE_PHASE").as_deref() == Ok("restored") {
            "restored"
        } else {
            "workspace"
        },
    )
}
fn cli(group: &str, action: &str, input: Value) -> Value {
    let mut child = Command::new(std::env::var("NOTEKA_SMOKE_CLI").unwrap())
        .args([
            "--json",
            "--workspace",
            workspace_path().to_str().unwrap(),
            group,
            action,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let r: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(out.status.success(), "{r}");
    r["data"].clone()
}
#[tauri::command]
pub fn smoke_control(app: tauri::AppHandle, step: String, input: Value) -> Value {
    let path = workspace_path();
    match step.as_str() {
        "init" => {
            let file = root().join("synthetic.txt");
            std::fs::write(&file, b"original offline").unwrap();
            let image = root().join("synthetic.png");
            std::fs::write(&image, include_bytes!("../icons/256x256.png")).unwrap();
            json!({"path":path,"phase":std::env::var("NOTEKA_SMOKE_PHASE").unwrap_or_default(),"theme":std::env::var("NOTEKA_SMOKE_THEME").unwrap_or_else(|_|"dark".into()),"root":root(),"restored":root().join("restored"),"file":file,"image":image,"export":root().join("ui-export")})
        }
        "get" => cli("entity", "get", input),
        "home" => cli("home", "query", input),
        "viewport" => {
            app.get_webview_window("main")
                .unwrap()
                .set_size(tauri::LogicalSize::new(
                    input["width"].as_f64().unwrap(),
                    input["height"].as_f64().unwrap(),
                ))
                .unwrap();
            json!(true)
        }
        "capture" => {
            let name = input["name"].as_str().unwrap();
            assert!(name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'));
            let destination = root().join(format!("{name}.png"));
            let helper = std::env::var("NOTEKA_CAPTURE_HELPER")
                .expect("NOTEKA_CAPTURE_HELPER requerido para capturas nativas");
            let out = Command::new("python3")
                .args([
                    helper.as_str(),
                    "--app",
                    "noteka-desktop",
                    "--path",
                    destination.to_str().unwrap(),
                ])
                .output()
                .unwrap();
            assert!(
                out.status.success(),
                "{}",
                String::from_utf8_lossy(&out.stderr)
            );
            json!(String::from_utf8_lossy(&out.stdout).trim())
        }
        "home_read" => {
            *HOME_READ.lock().unwrap() = (
                input["view"].as_str().unwrap_or_default().into(),
                input["delay"].as_u64().unwrap_or(0),
                input["failure"] == true,
            );
            json!(true)
        }
        "relation_add" => cli("relation", "add", input),
        "relation_remove" => cli("relation", "remove", input),
        "entity_archive" => cli("entity", "archive", input),
        "entity_restore" => cli("entity", "restore", input),
        "entity_raw" => {
            rusqlite::Connection::open(path.join("noteka.sqlite"))
                .unwrap()
                .execute(
                    "UPDATE entities SET payload=?2 WHERE id=?1",
                    rusqlite::params![input["id"].as_str().unwrap(), input["payload"].to_string()],
                )
                .unwrap();
            json!(true)
        }
        "home_seed" => {
            // Lotes sintéticos grandes sólo por instrumentación; escrituras funcionales se prueban por CLI.
            let mut db = rusqlite::Connection::open(path.join("noteka.sqlite")).unwrap();
            let tx = db.transaction().unwrap();
            for (k, kind) in ["task", "note", "project", "tag", "person"]
                .into_iter()
                .enumerate()
            {
                let payload = match kind {
                    "task" => json!({"due_date":input["date"],"state":"todo","inbox":true}),
                    "project" => json!({"state":"doing"}),
                    "tag" => json!({"classification":"topic"}),
                    _ => json!({}),
                };
                for i in 0..if matches!(kind, "task" | "project") {
                    102
                } else {
                    51
                } {
                    let id = format!("00000000-0000-4000-8000-{:012}", k * 1000 + i);
                    let p = if i >= 51 && kind == "task" {
                        json!({"state":"done"})
                    } else if i >= 51 && kind == "project" {
                        json!({"state":"planned"})
                    } else {
                        payload.clone()
                    };
                    tx.execute("INSERT INTO entities VALUES (?1,?2,?3,1,NULL,'2026-10-01T12:00:00.000000Z','2026-10-01T12:00:00.000000Z',?4)",rusqlite::params![id,kind,format!("Home synthetic {kind} {i:02} título largo para verificar ajuste y controles visibles"),p.to_string()]).unwrap();
                }
            }
            tx.commit().unwrap();
            json!(true)
        }
        "focus_host" => {
            let window = app.get_webview_window("main").unwrap();
            let webview: tauri::Webview = window.as_ref().clone();
            std::thread::spawn(move || {
                webview
                    .window()
                    .set_focus()
                    .expect("Foco de ventana de prueba");
                webview.set_focus().expect("Foco de webview de prueba");
            });
            json!(true)
        }
        "list" => cli("entity", "list", input),
        "entity_create" => cli("entity", "create", input),
        "import_attachment" => cli("attachment", "import", input),
        "templates" => cli("template", "list", input),
        "template_get" => cli("template", "get", input),
        "template_save" => cli("template", "save", input),
        "template_instantiate" => cli("template", "instantiate", input),
        "entity_count" => json!(
            rusqlite::Connection::open(path.join("noteka.sqlite"))
                .unwrap()
                .query_row("SELECT count(*) FROM entities", [], |r| r.get::<_, i64>(0))
                .unwrap()
        ),
        "template_failure_on" | "template_failure_off" => {
            let conn = rusqlite::Connection::open(path.join("noteka.sqlite")).unwrap();
            conn.execute_batch(if step == "template_failure_on" { "CREATE TRIGGER smoke_template_fail BEFORE UPDATE ON templates BEGIN SELECT RAISE(ABORT,'test'); END;" } else { "DROP TRIGGER smoke_template_fail;" }).unwrap();
            json!(true)
        }
        "template_raw" => {
            rusqlite::Connection::open(path.join("noteka.sqlite"))
                .unwrap()
                .execute(
                    "UPDATE templates SET definition=?2 WHERE id=?1",
                    rusqlite::params![
                        input["id"].as_str().unwrap(),
                        input["definition"].to_string()
                    ],
                )
                .unwrap();
            json!(true)
        }
        "view_raw" => {
            rusqlite::Connection::open(path.join("noteka.sqlite"))
                .unwrap()
                .execute(
                    "UPDATE views SET json=?2 WHERE id=?1",
                    rusqlite::params![input["id"].as_str().unwrap(), input["view"].to_string()],
                )
                .unwrap();
            json!(true)
        }
        "workflow" => cli("workflow", "query", input),
        "search" => cli("search", "query", input),
        "collections" => cli("collection", "list", input),
        "collection_get" => cli("collection", "get", input),
        "property_define" => cli("property", "define", input),
        "formula_set" => cli("formula", "set", input),
        "record_get" => cli("record", "get", input),
        "record_create" => cli("record", "create", input),
        "record_update" => cli("record", "update", input),
        "database_query" => cli("database", "query", input),
        "view_save" => cli("view", "save", input),
        "query_value" => {
            let conn = rusqlite::Connection::open(path.join("noteka.sqlite")).unwrap();
            conn.execute(
                "UPDATE property_values SET json=?1 WHERE entity_id=?2 AND property_id=?3",
                rusqlite::params![
                    input["value"].to_string(),
                    input["entity_id"].as_str().unwrap(),
                    input["property_id"].as_str().unwrap()
                ],
            )
            .unwrap();
            json!(true)
        }
        "cli_update" => cli("entity", "update", input),
        "rich" => cli(
            "entity",
            "create",
            json!({"kind":"note","title":"Bloques enriquecidos","document":serde_json::from_str::<Value>(include_str!("../../tests/fixtures/rich-document.json")).unwrap()}),
        ),
        "read_attachment" => cli("attachment", "read", input),
        "check_export" => serde_json::from_slice::<Value>(
            &std::fs::read(root().join("ui-export").join("manifest.json")).unwrap(),
        )
        .unwrap(),
        "failure_on" | "failure_off" => {
            let conn = rusqlite::Connection::open(path.join("noteka.sqlite")).unwrap();
            conn.execute_batch(if step=="failure_on" {"CREATE TRIGGER smoke_fail BEFORE INSERT ON changes BEGIN SELECT RAISE(ABORT,'injected'); END;"} else {"DROP TRIGGER smoke_fail;"}).unwrap();
            json!(true)
        }
        "unknown" => {
            let note = cli(
                "entity",
                "create",
                json!({"kind":"note","title":"Formato futuro"}),
            );
            let document = json!({"schema_version":7,"blocks":[{"id":"future","type":"widget","keep":{"value":42}}]});
            let conn = rusqlite::Connection::open(path.join("noteka.sqlite")).unwrap();
            conn.execute(
                "UPDATE documents SET json=?2 WHERE entity_id=?1",
                rusqlite::params![note["id"].as_str().unwrap(), document.to_string()],
            )
            .unwrap();
            note
        }
        "archived_project" => {
            let project = cli(
                "entity",
                "create",
                json!({"kind":"project","title":"Proyecto relacionado"}),
            );
            let relation = cli(
                "relation",
                "add",
                json!({"source_id":input["id"],"target_id":project["id"],"kind":"project","expected_revision":input["revision"]}),
            );
            cli(
                "entity",
                "archive",
                json!({"id":project["id"],"expected_revision":1}),
            );
            relation
        }
        "close_request" => {
            app.get_webview_window("main").unwrap().close().unwrap();
            json!(true)
        }
        "pending" => json!(
            app.state::<super::Pending>()
                .0
                .load(std::sync::atomic::Ordering::SeqCst)
        ),
        "roundtrip" => {
            cli(
                "workspace",
                "backup",
                json!({"destination":root().join("backup")}),
            );
            cli(
                "workspace",
                "restore",
                json!({"backup":root().join("backup"),"destination":root().join("restored")}),
            )
        }
        "finish" => {
            println!("NOTEKA_SMOKE_RESULT {}", input);
            app.state::<super::Pending>()
                .0
                .store(false, std::sync::atomic::Ordering::SeqCst);
            app.exit(if input["ok"] == true { 0 } else { 1 });
            json!(true)
        }
        "record" => {
            println!("NOTEKA_SMOKE_RESULT {}", input);
            json!(true)
        }
        _ => panic!("Paso de prueba no soportado"),
    }
}
