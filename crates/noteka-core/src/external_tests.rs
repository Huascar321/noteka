use crate::{
    contracts::*,
    workspace::{self, Workspace},
};
use serde_json::json;

fn setup() -> (tempfile::TempDir, Workspace) {
    let directory = tempfile::tempdir().unwrap();
    let workspace = Workspace::create(directory.path(), Some("UTC".into())).unwrap();
    (directory, workspace)
}
fn note(workspace: &mut Workspace, title: &str) -> Entity {
    workspace
        .entity_create(
            serde_json::from_value(json!({"kind":"note","title":title})).expect("CreateEntity"),
            "test",
            "request",
        )
        .unwrap()
}
fn link(entity: &Entity, metadata: serde_json::Value) -> LinkExternalResource {
    LinkExternalResource {
        entity_id: entity.id.clone(),
        expected_revision: entity.revision,
        provider: "google_drive".into(),
        account_id: "account-1".into(),
        resource_type: "drive_file".into(),
        resource_id: "file-1".into(),
        metadata,
    }
}

#[test]
fn link_is_idempotent_updates_metadata_and_shares_resource() {
    let (_directory, mut workspace) = setup();
    let first = note(&mut workspace, "Primera");
    let linked = workspace
        .external_link(
            link(&first, json!({"name":"Plan","mimeType":"application/pdf"})),
            "test",
            "link-1",
        )
        .unwrap();
    assert!(linked.changed);
    assert_eq!(linked.entity.revision, first.revision + 1);
    assert_eq!(linked.resource.resource_id, "file-1");
    assert_eq!(linked.resource.metadata["name"], "Plan");
    assert_eq!(
        workspace
            .external_list(ListExternalResources {
                entity_id: first.id.clone(),
                cursor: None,
                limit: None
            })
            .unwrap()
            .items
            .len(),
        1
    );

    let repeated = workspace
        .external_link(
            link(
                &linked.entity,
                json!({"name":"Plan","mimeType":"application/pdf"}),
            ),
            "test",
            "link-2",
        )
        .unwrap();
    assert!(!repeated.changed);
    assert_eq!(repeated.entity.revision, linked.entity.revision);

    let changed = workspace
        .external_link(
            link(
                &repeated.entity,
                json!({"name":"Plan v2","mimeType":"application/pdf"}),
            ),
            "test",
            "link-3",
        )
        .unwrap();
    assert!(changed.changed);
    assert_eq!(changed.entity.revision, repeated.entity.revision + 1);
    assert_eq!(changed.resource.id, linked.resource.id);
    assert_eq!(changed.resource.metadata["name"], "Plan v2");

    let second = note(&mut workspace, "Segunda");
    let shared = workspace
        .external_link(
            LinkExternalResource {
                entity_id: second.id.clone(),
                expected_revision: second.revision,
                provider: "google_drive".into(),
                account_id: "account-1".into(),
                resource_type: "drive_file".into(),
                resource_id: "file-1".into(),
                metadata: json!({"name":"Plan v2"}),
            },
            "test",
            "link-4",
        )
        .unwrap();
    assert!(shared.changed);
    assert_eq!(shared.resource.id, linked.resource.id);
    assert_eq!(
        workspace
            .external_list(ListExternalResources {
                entity_id: second.id.clone(),
                cursor: None,
                limit: None
            })
            .unwrap()
            .items
            .len(),
        1
    );

    let unlinked = workspace
        .external_unlink(
            UnlinkExternalResource {
                entity_id: first.id.clone(),
                expected_revision: changed.entity.revision,
                id: linked.resource.id.clone(),
            },
            "test",
            "unlink-1",
        )
        .unwrap();
    assert!(unlinked.changed);
    assert_eq!(unlinked.entity.revision, changed.entity.revision + 1);
    assert!(
        workspace
            .external_list(ListExternalResources {
                entity_id: first.id,
                cursor: None,
                limit: None
            })
            .unwrap()
            .items
            .is_empty()
    );
    assert_eq!(
        workspace
            .external_list(ListExternalResources {
                entity_id: second.id,
                cursor: None,
                limit: None
            })
            .unwrap()
            .items
            .len(),
        1
    );
    let resource_count: u32 = workspace
        .conn
        .query_row("SELECT count(*) FROM external_resources", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(resource_count, 1, "unlink no borra el recurso externo");
}

#[test]
fn rejects_invalid_fields_metadata_and_missing_association() {
    let (_directory, mut workspace) = setup();
    let entity = note(&mut workspace, "Entidad");
    let mut input = link(&entity, json!({}));
    input.provider = " ".into();
    assert_eq!(
        format!(
            "{:?}",
            workspace
                .external_link(input, "test", "bad-provider")
                .unwrap_err()
                .code
        ),
        "ValidationError"
    );
    let mut input = link(&entity, json!({"blob":"x".repeat(65_536)}));
    input.resource_id = "file-2".into();
    assert_eq!(
        format!(
            "{:?}",
            workspace
                .external_link(input, "test", "bad-metadata")
                .unwrap_err()
                .code
        ),
        "ValidationError"
    );
    let missing = workspace.external_unlink(
        UnlinkExternalResource {
            entity_id: entity.id,
            expected_revision: entity.revision,
            id: "missing".into(),
        },
        "test",
        "unlink-missing",
    );
    assert_eq!(format!("{:?}", missing.unwrap_err().code), "NotFound");
}

#[test]
fn migrates_v19_to_external_tables() {
    let (directory, workspace) = setup();
    let path = workspace.path.clone();
    workspace
        .conn
        .execute_batch(
            "DROP TABLE entity_external_resources; DROP TABLE external_resources; UPDATE workspace_meta SET schema_version=19; PRAGMA user_version=19;",
        )
        .unwrap();
    drop(workspace);
    let workspace = Workspace::open(&path).unwrap();
    assert_eq!(workspace::version(&workspace.conn).unwrap(), 20);
    assert_eq!(
        workspace
            .conn
            .query_row("SELECT count(*) FROM external_resources", [], |row| row
                .get::<_, u32>(0))
            .unwrap(),
        0
    );
    drop(directory);
}

#[test]
fn external_list_paginates_by_resource_id() {
    let (_directory, mut workspace) = setup();
    let entity = note(&mut workspace, "Paginada");
    let mut revision = entity.revision;
    for index in 0..3 {
        let linked = workspace
            .external_link(
                LinkExternalResource {
                    entity_id: entity.id.clone(),
                    expected_revision: revision,
                    provider: "google_drive".into(),
                    account_id: "account-1".into(),
                    resource_type: "drive_file".into(),
                    resource_id: format!("file-{index}"),
                    metadata: json!({"index":index}),
                },
                "test",
                "page-link",
            )
            .unwrap();
        revision = linked.entity.revision;
    }
    let first = workspace
        .external_list(ListExternalResources {
            entity_id: entity.id.clone(),
            cursor: None,
            limit: Some(1),
        })
        .unwrap();
    assert_eq!(first.items.len(), 1);
    let cursor = first.next_cursor.clone().expect("cursor");
    let second = workspace
        .external_list(ListExternalResources {
            entity_id: entity.id,
            cursor: Some(cursor),
            limit: Some(1),
        })
        .unwrap();
    assert_eq!(second.items.len(), 1);
    assert_ne!(first.items[0].id, second.items[0].id);
}
