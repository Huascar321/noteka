use super::*;
use crate::{
    attachment_search as lexical, attachment_search_contracts::AttachmentIndexInput,
    attachment_search_tests as fixtures, contracts::*, workspace::Workspace,
};
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

struct Fixture {
    dimension: u32,
    fail: bool,
}
impl EmbeddingRuntime for Fixture {
    fn embed(
        &self,
        _purpose: EmbeddingPurpose,
        texts: &[String],
        _timeout: Duration,
    ) -> Result<Vec<Vec<f32>>> {
        if self.fail {
            return Err(unavailable("fixture_failure"));
        }
        Ok(texts
            .iter()
            .map(|text| fixture_vector(text, self.dimension))
            .collect())
    }
}
fn fixture_vector(text: &str, dimension: u32) -> Vec<f32> {
    let mut vector = vec![0.0f32; dimension as usize];
    for word in text
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
    {
        let mut hash = 0u64;
        for byte in word.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(u64::from(byte));
        }
        vector[(hash as usize) % dimension as usize] += 1.0;
    }
    if vector.iter().all(|v| *v == 0.0) {
        vector[0] = 1.0;
    }
    let norm = vector.iter().map(|v| v * v).sum::<f32>().sqrt();
    for value in &mut vector {
        *value /= norm;
    }
    vector
}
fn profile() -> EmbeddingProfile {
    EmbeddingProfile {
        version: 1,
        provider: "local_http".into(),
        runtime: "openai_embeddings_v1".into(),
        model: "fixture-embedding".into(),
        revision: "test-v1".into(),
        dimension: 8,
        normalization: EmbeddingNormalization::L2,
        task: EmbeddingTask::Retrieval,
        query_prefix: "Query: ".into(),
        document_prefix: "Document: ".into(),
        tokenizer: "fixture".into(),
        chunking: EmbeddingChunking::ExtractionFragmentV1,
        quantization: EmbeddingQuantization::F32,
    }
}
fn setup() -> (tempfile::TempDir, Workspace) {
    let t = tempfile::tempdir().unwrap();
    let w = Workspace::create(&t.path().join("w"), Some("UTC".into())).unwrap();
    (t, w)
}
fn build(
    w: &Workspace,
    profile: EmbeddingProfile,
    filter: IndexFilter,
    runtime: &dyn EmbeddingRuntime,
) -> Result<SemanticBuildResult> {
    build_with_runtime(
        &w.path,
        profile,
        "http://127.0.0.1:1/v1/embeddings".into(),
        filter,
        None,
        runtime,
    )
}
fn all() -> IndexFilter {
    IndexFilter {
        job_id: None,
        attachment_id: None,
        extractor: None,
    }
}
fn lexical_index(w: &Workspace, job: &ExtractionJob) {
    let revision = index::control(&w.conn).unwrap().revision;
    lexical::index(
        &w.path,
        AttachmentIndexInput::Put {
            job_id: job.id.clone(),
            expected_generation_id: job.generation.as_ref().unwrap().id.clone(),
            expected_revision: revision,
        },
    )
    .unwrap();
}
fn retrieval(query: &str, mode: RetrievalMode) -> RetrievalInput {
    RetrievalInput {
        query: query.into(),
        mode: Some(mode),
        job_id: None,
        attachment_id: None,
        extractor: None,
        limit: None,
        budget_bytes: None,
        include_stale: false,
    }
}
fn status(w: &Workspace) -> EmbeddingStatus {
    match index(&w.path, EmbeddingIndexInput::Status {}, None).unwrap() {
        EmbeddingIndexResult::Status(status) => status,
        _ => unreachable!(),
    }
}

#[test]
fn profile_fingerprint_is_stable_and_space_sensitive() {
    let base = profile();
    base.validate().unwrap();
    assert_eq!(base.fingerprint(), base.fingerprint());
    let variants = [
        EmbeddingProfile {
            revision: "test-v2".into(),
            ..base.clone()
        },
        EmbeddingProfile {
            dimension: 16,
            ..base.clone()
        },
        EmbeddingProfile {
            query_prefix: "Q: ".into(),
            ..base.clone()
        },
        EmbeddingProfile {
            document_prefix: "D: ".into(),
            ..base.clone()
        },
        EmbeddingProfile {
            tokenizer: "other".into(),
            ..base.clone()
        },
    ];
    for variant in variants {
        assert_ne!(base.fingerprint(), variant.fingerprint());
    }
    let invalid = EmbeddingProfile {
        dimension: 0,
        ..base.clone()
    };
    assert!(invalid.validate().is_err());
    let invalid = EmbeddingProfile {
        revision: "latest".into(),
        ..base.clone()
    };
    assert!(invalid.validate().is_err());
    let invalid = EmbeddingProfile {
        provider: "remote".into(),
        ..base.clone()
    };
    assert!(invalid.validate().is_err());
}

#[test]
fn build_activates_reuses_and_returns_sources() {
    let (_t, mut w) = setup();
    let a = fixtures::add(&mut w, b"alpha beta", "utf8_text");
    let b = fixtures::add(&mut w, b"gamma delta", "utf8_text");
    let runtime = Fixture {
        dimension: 8,
        fail: false,
    };
    let first = build(&w, profile(), all(), &runtime).unwrap();
    assert!(!first.reused);
    assert_eq!(first.generation.state, SemanticGenerationState::Active);
    assert_eq!(first.generation.chunk_count, 2);
    assert_eq!(first.generation.source_count, 2);
    let state = status(&w);
    assert_eq!(
        state.active.as_ref().unwrap().generation_id,
        first.generation.generation_id
    );
    assert_eq!(state.coverage.active_chunks, 2);
    let page = search_inner(
        &w.path,
        retrieval("alpha", RetrievalMode::Semantic),
        None,
        Some(&runtime),
    )
    .unwrap();
    assert_eq!(page.mode_used, RetrievalMode::Semantic);
    assert!(!page.items.is_empty());
    assert_eq!(page.items[0].job_id, a.id);
    assert_ne!(page.items[0].job_id, b.id);
    assert!(page.items[0].semantic_score.unwrap() > 0.0);
    let second = build(&w, profile(), all(), &runtime).unwrap();
    assert!(second.reused);
    assert_eq!(second.embedded_chunks, 0);
    assert_eq!(
        second.generation.generation_id,
        first.generation.generation_id
    );
}

#[test]
fn failed_build_preserves_active_generation() {
    let (_t, mut w) = setup();
    fixtures::add(&mut w, b"alpha beta", "utf8_text");
    let runtime = Fixture {
        dimension: 8,
        fail: false,
    };
    let first = build(&w, profile(), all(), &runtime).unwrap();
    fixtures::add(&mut w, b"fail me", "utf8_text");
    let error = build(
        &w,
        profile(),
        all(),
        &Fixture {
            dimension: 8,
            fail: true,
        },
    )
    .unwrap_err();
    assert!(matches!(error.code, ErrorCode::StorageError));
    let state = status(&w);
    assert_eq!(
        state.active.as_ref().unwrap().generation_id,
        first.generation.generation_id
    );
    assert!(state.building.is_none());
    assert_eq!(
        w.conn
            .query_row(
                "SELECT count(*) FROM semantic_chunks WHERE generation_id=?1",
                [&first.generation.generation_id],
                |r| r.get::<_, u32>(0)
            )
            .unwrap(),
        1
    );
}

#[test]
fn stale_sources_are_excluded_unless_requested() {
    let (_t, mut w) = setup();
    let job = fixtures::add(&mut w, b"alpha beta", "utf8_text");
    let runtime = Fixture {
        dimension: 8,
        fail: false,
    };
    build(&w, profile(), all(), &runtime).unwrap();
    w.conn
        .execute(
            "UPDATE attachments SET sha256=?1 WHERE id=?2",
            rusqlite::params!["0".repeat(64), job.attachment_id],
        )
        .unwrap();
    let page = search_inner(
        &w.path,
        retrieval("alpha", RetrievalMode::Semantic),
        None,
        Some(&runtime),
    )
    .unwrap();
    assert!(page.items.is_empty());
    assert_eq!(page.coverage.stale_excluded, 1);
    let mut input = retrieval("alpha", RetrievalMode::Semantic);
    input.include_stale = true;
    let page = search_inner(&w.path, input, None, Some(&runtime)).unwrap();
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].source_state, RetrievalSourceState::Stale);
}

#[test]
fn hybrid_falls_back_to_lexical_when_provider_is_unreachable() {
    let (_t, mut w) = setup();
    let job = fixtures::add(&mut w, b"alpha beta", "utf8_text");
    fixtures::add(&mut w, b"gamma delta", "utf8_text");
    lexical_index(&w, &job);
    let runtime = Fixture {
        dimension: 8,
        fail: false,
    };
    build(&w, profile(), all(), &runtime).unwrap();
    let page = search(&w.path, retrieval("alpha", RetrievalMode::Hybrid), None).unwrap();
    assert_eq!(page.mode_used, RetrievalMode::Lexical);
    assert!(page.degraded.iter().any(|r| r == "query_embedding_failed"));
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].job_id, job.id);
    assert!(page.items[0].lexical_rank.is_some());
}

#[test]
fn hybrid_uses_rrf_and_respects_budget() {
    let (_t, mut w) = setup();
    let job = fixtures::add(&mut w, b"alpha beta", "utf8_text");
    lexical_index(&w, &job);
    let runtime = Fixture {
        dimension: 8,
        fail: false,
    };
    build(&w, profile(), all(), &runtime).unwrap();
    let mut input = retrieval("alpha", RetrievalMode::Hybrid);
    input.budget_bytes = Some(1);
    let page = search_inner(&w.path, input, None, Some(&runtime)).unwrap();
    assert!(page.items.is_empty());
    assert!(page.budget_exhausted);
    let page = search_inner(
        &w.path,
        retrieval("alpha", RetrievalMode::Hybrid),
        None,
        Some(&runtime),
    )
    .unwrap();
    assert_eq!(page.mode_used, RetrievalMode::Hybrid);
    assert_eq!(page.items.len(), 1);
    assert!(page.items[0].semantic_rank.is_some());
    assert!(page.items[0].lexical_rank.is_some());
}

#[test]
fn lexical_only_when_no_semantic_generation_exists() {
    let (_t, mut w) = setup();
    let job = fixtures::add(&mut w, b"alpha beta", "utf8_text");
    lexical_index(&w, &job);
    for mode in [RetrievalMode::Hybrid, RetrievalMode::Semantic] {
        let page = search(&w.path, retrieval("alpha", mode), None).unwrap();
        assert_eq!(page.mode_used, RetrievalMode::Lexical, "{mode:?}");
        assert!(page.degraded.iter().any(|r| r == "no_active_generation"));
        assert_eq!(page.items.len(), 1);
    }
}

#[test]
fn migration_017_is_additive_and_backup_preserves_active_generation() {
    let (t, mut w) = setup();
    fixtures::add(&mut w, b"alpha beta", "utf8_text");
    let runtime = Fixture {
        dimension: 8,
        fail: false,
    };
    let built = build(&w, profile(), all(), &runtime).unwrap();
    let backup = t.path().join("backup");
    w.backup(&backup).unwrap();
    w.conn
        .execute_batch(
            "DROP TABLE agent_runs;DROP TABLE semantic_chunks;DROP TABLE semantic_control;DROP TABLE semantic_generations;DROP TABLE semantic_profiles;DROP TABLE IF EXISTS extraction_pdf_structures;
UPDATE workspace_meta SET schema_version=16;PRAGMA user_version=16",
        )
        .unwrap();
    w.migrate(false).unwrap();
    assert_eq!(crate::workspace::version(&w.conn).unwrap(), 20);
    let state = status(&w);
    assert!(state.active.is_none());
    assert_eq!(state.revision, 1);
    let destination = t.path().join("restored");
    Workspace::restore(&backup, &destination).unwrap();
    let restored = Workspace::open(&destination).unwrap();
    let state = status(&restored);
    assert_eq!(
        state.active.as_ref().unwrap().generation_id,
        built.generation.generation_id
    );
    assert_eq!(state.coverage.active_chunks, 1);
    assert_eq!(
        restored
            .conn
            .query_row("SELECT count(*) FROM attachments", [], |r| r
                .get::<_, u32>(0))
            .unwrap(),
        1
    );
}

#[test]
fn reset_requires_current_revision_and_clears_only_derived_data() {
    let (_t, mut w) = setup();
    fixtures::add(&mut w, b"alpha beta", "utf8_text");
    let runtime = Fixture {
        dimension: 8,
        fail: false,
    };
    build(&w, profile(), all(), &runtime).unwrap();
    let state = status(&w);
    let error = index(
        &w.path,
        EmbeddingIndexInput::Reset {
            expected_revision: state.revision + 1,
        },
        None,
    )
    .unwrap_err();
    assert!(matches!(error.code, ErrorCode::RevisionConflict));
    let result = index(
        &w.path,
        EmbeddingIndexInput::Reset {
            expected_revision: state.revision,
        },
        None,
    )
    .unwrap();
    assert!(matches!(
        result,
        EmbeddingIndexResult::Reset {
            removed_chunks: 1,
            ..
        }
    ));
    let state = status(&w);
    assert!(state.active.is_none());
    assert_eq!(
        w.conn
            .query_row("SELECT count(*) FROM extraction_jobs", [], |r| r
                .get::<_, u32>(0))
            .unwrap(),
        1
    );
}

fn mock_server(body: String) -> (String, mpsc::Receiver<()>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (ready_tx, ready_rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        ready_tx.send(()).unwrap();
        let mut buffer = [0u8; 16 * 1024];
        let _ = stream.read(&mut buffer).unwrap();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();
    });
    (format!("http://{address}/v1/embeddings"), ready_rx, handle)
}

#[test]
fn http_adapter_parses_and_normalizes_vectors() {
    let body =
        serde_json::json!({"data":[{"index":0,"embedding":[3.0,4.0,0.0,0.0,0.0,0.0,0.0,0.0]}]})
            .to_string();
    let (url, ready, handle) = mock_server(body);
    let runtime = HttpEmbeddingRuntime::new(profile(), url).unwrap();
    let vectors = runtime
        .embed(
            EmbeddingPurpose::Query,
            &["alpha".into()],
            Duration::from_secs(2),
        )
        .unwrap();
    assert_eq!(vectors.len(), 1);
    assert!((vectors[0][0] - 0.6).abs() < 1e-6);
    assert!((vectors[0][1] - 0.8).abs() < 1e-6);
    ready.recv().unwrap();
    handle.join().unwrap();
}

#[test]
fn http_adapter_rejects_wrong_dimension() {
    let body = serde_json::json!({"data":[{"index":0,"embedding":[1.0]}]}).to_string();
    let (url, ready, handle) = mock_server(body);
    let runtime = HttpEmbeddingRuntime::new(profile(), url).unwrap();
    let error = runtime
        .embed(
            EmbeddingPurpose::Document,
            &["alpha".into()],
            Duration::from_secs(2),
        )
        .unwrap_err();
    assert!(matches!(error.code, ErrorCode::StorageError));
    ready.recv().unwrap();
    handle.join().unwrap();
}

#[test]
fn remote_endpoint_is_rejected_before_any_request() {
    let error = HttpEmbeddingRuntime::new(profile(), "https://example.com/v1/embeddings".into())
        .unwrap_err();
    assert!(matches!(error.code, ErrorCode::StorageError));
}

#[test]
#[ignore = "benchmark explícito del backend exacto; no forma parte del gate por defecto"]
fn semantic_backend_benchmark() {
    use std::time::Instant;
    fn vector(seed: u64, dimension: usize) -> Vec<f32> {
        let mut state = seed | 1;
        let mut out = Vec::with_capacity(dimension);
        for _ in 0..dimension {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            out.push(((state >> 11) as f64 / (1u64 << 53) as f64) as f32 - 0.5);
        }
        let norm = out.iter().map(|v| v * v).sum::<f32>().sqrt();
        for value in &mut out {
            *value /= norm;
        }
        out
    }
    let dimension = 1024usize;
    let query = vector(0x51ed270b, dimension);
    for size in [1usize, 4096, 8192] {
        let chunks = (0..size)
            .map(|i| {
                vector(
                    0x9e3779b97f4a7c15u64.wrapping_add(i as u64 * 0x100000001b3),
                    dimension,
                )
            })
            .collect::<Vec<_>>();
        let scan = || {
            let mut scored = Vec::with_capacity(chunks.len());
            for (i, chunk) in chunks.iter().enumerate() {
                scored.push((semantic::dot(&query, chunk), i));
            }
            scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap().then(a.1.cmp(&b.1)));
            scored.truncate(50);
            scored
        };
        for _ in 0..5 {
            scan();
        }
        let mut samples = Vec::with_capacity(20);
        for _ in 0..20 {
            let started = Instant::now();
            let scored = scan();
            assert_eq!(scored.len(), 50.min(size));
            samples.push(started.elapsed().as_secs_f64() * 1000.0);
        }
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p50 = samples[samples.len() / 2];
        let p95 = samples[(samples.len() * 95 / 100).min(samples.len() - 1)];
        println!(
            "{{\"chunks\":{},\"dimension\":{},\"vectors_bytes\":{},\"p50_ms\":{:.6},\"p95_ms\":{:.6},\"samples\":20}}",
            size,
            dimension,
            size * dimension * 4,
            p50,
            p95
        );
    }
}
