use crate::{
    batch_storage::Budget,
    contracts::*,
    error::*,
    extraction::job_error,
    extraction_storage as store, ocr_storage, ocr_supervisor, pdf_storage, pdf_supervisor,
    text_extractor,
    workspace::{self, Workspace},
    xlsx_storage, xlsx_supervisor,
};
use chrono::{Duration as ChronoDuration, Utc};
use rusqlite::{Connection, params};
use serde_json::json;
use std::time::Duration;

struct Claim {
    job: ExtractionJob,
    owner: store::Owner,
    recovered: u32,
    incompatible: u32,
}
enum Claimed {
    Work(Claim),
    Done(ExtractionRun),
    Recovered(ExtractionJob),
}
fn idle(reason: &str, recovered: u32, incompatible: u32) -> ExtractionRun {
    ExtractionRun {
        outcome: ExtractionOutcome::Idle,
        reason: Some(reason.into()),
        job: None,
        recovered_count: recovered,
        incompatible_count: incompatible,
    }
}
fn fence(conn: &Connection, root: &std::path::Path, c: &Claim, b: &Budget<'_>) -> Result<()> {
    b.check()?;
    let ctl = store::control(conn)?;
    let o = store::owner(conn)?;
    let j = store::job(conn, root, &c.job.id)?;
    if j.state != ExtractionState::Running
        || j.revision != c.job.revision
        || !j.supported
        || j.config_hash != c.job.config_hash
        || j.config != c.job.config
        || j.extractor != c.job.extractor
        || j.extractor_version != c.job.extractor_version
        || j.attachment_id != c.job.attachment_id
        || j.source_sha256 != c.job.source_sha256
        || j.source_bytes != c.job.source_bytes
        || ctl.paused
        || ctl.revision != c.owner.control_revision
        || o.as_ref().is_none_or(|o| {
            o.token != c.owner.token
                || o.job_id != c.job.id
                || o.scope != c.owner.scope
                || o.control_revision != c.owner.control_revision
                || o.lease_until != c.owner.lease_until
        })
    {
        return Err(store::error(ErrorCode::WorkspaceBusy, "ownership_lost"));
    }
    if !c.owner.live(&store::scope(conn, root)?)? {
        return Err(store::error(ErrorCode::WorkspaceBusy, "worker_interrupted"));
    }
    Ok(())
}
impl Workspace {
    fn extraction_claim(
        &self,
        input: &RunExtraction,
        #[cfg(test)] hook: &mut Option<&mut dyn FnMut(&str)>,
    ) -> Result<Claimed> {
        if let Some(id) = &input.id {
            store::uuid(id)?;
        }
        let tx = store::immediate(&self.conn)?;
        #[cfg(test)]
        if let Some(h) = hook.as_mut() {
            h("claim_locked");
        }
        store::queue_integrity(&tx)?;
        pdf_storage::shape(&tx)?;
        xlsx_storage::shape(&tx)?;
        ocr_storage::shape(&tx)?;
        let ctl = store::control(&tx)?;
        let recovered = store::recover(&tx, &self.path)?;
        if input
            .id
            .as_ref()
            .is_some_and(|id| Some(id) == recovered.as_ref())
        {
            let j = store::job(&tx, &self.path, input.id.as_ref().unwrap())?;
            store::response(&j)?;
            tx.commit()?;
            return Ok(Claimed::Recovered(j));
        }
        let recovered = u32::from(recovered.is_some());
        let requested = input
            .id
            .as_ref()
            .map(|id| store::job(&tx, &self.path, id))
            .transpose()?;
        if requested.as_ref().is_some_and(|j| !j.supported) {
            return Err(store::invalid("unsupported_extractor_config"));
        }
        if ctl.paused {
            if requested
                .as_ref()
                .is_some_and(|j| j.state != ExtractionState::Queued)
            {
                return Err(store::invalid("invalid_state"));
            }
            let out = idle("queue_paused", recovered, 0);
            store::response(&out)?;
            tx.commit()?;
            return Ok(Claimed::Done(out));
        }
        if store::owner(&tx)?.is_some() {
            return Err(store::error(ErrorCode::WorkspaceBusy, "worker_active"));
        }
        let mut incompatible = 0;
        let selected = if let Some(j) = requested {
            if j.state != ExtractionState::Queued {
                return Err(store::invalid("invalid_state"));
            }
            Some(j)
        } else {
            let ids:Vec<String>=tx.prepare("SELECT id FROM extraction_jobs WHERE state='queued' ORDER BY created_at COLLATE BINARY,id COLLATE BINARY")?.query_map([],|r|r.get(0))?.collect::<std::result::Result<_,_>>()?;
            let mut selected = None;
            for id in ids {
                let j = store::job(&tx, &self.path, &id)?;
                if !j.supported {
                    incompatible += 1;
                } else if selected.is_none() {
                    selected = Some(j);
                }
            }
            selected
        };
        let Some(j) = selected else {
            let out = idle(
                if incompatible > 0 {
                    "no_compatible_jobs"
                } else {
                    "queue_empty"
                },
                recovered,
                incompatible,
            );
            store::response(&out)?;
            tx.commit()?;
            return Ok(Claimed::Done(out));
        };
        let owner = store::Owner {
            job_id: j.id.clone(),
            token: workspace::id(),
            scope: store::scope(&tx, &self.path)?,
            control_revision: ctl.revision,
            lease_until: (Utc::now() + ChronoDuration::seconds(EXTRACTION_LEASE_SECONDS))
                .to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
        };
        // Reservar capacidad de revisión para terminar/revertir este claim.
        store::bump(store::bump(j.revision)?)?;
        tx.execute("UPDATE extraction_jobs SET state='running',revision=?2,attempts=?3,updated_at=?4,started_at=?4,finished_at=NULL,error=NULL WHERE id=?1",params![j.id,store::bump(j.revision)?,store::bump(j.attempts)?,workspace::now()])?;
        tx.execute("UPDATE extraction_runtime SET job_id=?1,token=?2,scope=?3,control_revision=?4,lease_until=?5 WHERE id=1",params![owner.job_id,owner.token,owner.scope,owner.control_revision,owner.lease_until])?;
        let job = store::job(&tx, &self.path, &j.id)?;
        store::response(&job)?;
        tx.commit()?;
        #[cfg(test)]
        if let Some(h) = hook.as_mut() {
            h("claim_committed");
        }
        Ok(Claimed::Work(Claim {
            job,
            owner,
            recovered,
            incompatible,
        }))
    }
    pub fn extraction_run(&self, input: RunExtraction) -> Result<ExtractionRun> {
        self.extraction_run_impl(
            input,
            #[cfg(test)]
            None,
            #[cfg(test)]
            None,
        )
    }
    pub(crate) fn extraction_run_impl(
        &self,
        input: RunExtraction,
        #[cfg(test)] mut hook: Option<&mut dyn FnMut(&str)>,
        #[cfg(test)] test_duration: Option<Duration>,
    ) -> Result<ExtractionRun> {
        // La reconstrucción/hash de todos los derivados no debe retener el lock
        // de escritor. Claim vuelve a comprobar estado, ownership y forma actual;
        // publicación revalida fence/fuente/budget y sólo confirma datos validados.
        {
            let snapshot = self.conn.unchecked_transaction()?;
            pdf_storage::integrity(&snapshot)?;
            xlsx_storage::integrity(&snapshot)?;
            ocr_storage::integrity(&snapshot)?;
        }
        #[cfg(test)]
        if let Some(h) = hook.as_mut() {
            h("before_claim");
        }
        let c = match self.extraction_claim(
            &input,
            #[cfg(test)]
            &mut hook,
        )? {
            Claimed::Done(out) => return Ok(out),
            Claimed::Recovered(j) => return Err(job_error(j.error.clone().unwrap(), &j)),
            Claimed::Work(c) => c,
        };
        let duration = Duration::from_millis(EXTRACTION_EXECUTION_MS);
        #[cfg(test)]
        let duration = test_duration.unwrap_or(duration);
        let result = (|| {
            let b = Budget::with_duration(&self.conn, duration)?;
            let result = (|| {
                #[cfg(test)]
                if let Some(h) = hook.as_mut() {
                    h("claimed");
                }
                let check = || {
                    let tx = self.conn.unchecked_transaction()?;
                    fence(&tx, &self.path, &c, &b)
                };
                check()?;
                let a = store::job_source(&self.conn, &c.job)?;
                store::eligible_for(&a, &c.job.extractor)?;
                let bytes = store::source(&self.path, &a, check)?;
                #[cfg(test)]
                if let Some(h) = hook.as_mut() {
                    h("source_read");
                }
                let (parts, pdf, xlsx, ocr) = match c.job.extractor.as_str() {
                    "utf8_text" => (text_extractor::extract(&bytes, check)?, None, None, None),
                    "pdf_text" => (
                        Vec::new(),
                        Some(pdf_storage::prepare(pdf_supervisor::run(
                            &bytes, &b, check,
                        )?)?),
                        None,
                        None,
                    ),
                    "xlsx_cells" => (
                        Vec::new(),
                        None,
                        Some(xlsx_storage::prepare(xlsx_supervisor::run(
                            &bytes, &b, check,
                        )?)?),
                        None,
                    ),
                    "image_ocr" => (
                        Vec::new(),
                        None,
                        None,
                        Some(ocr_storage::prepare(ocr_supervisor::run(
                            &bytes, &b, check,
                        )?)?),
                    ),
                    _ => return Err(store::invalid("unsupported_extractor_config")),
                };
                if let Some(o) = &ocr {
                    ocr_storage::source_format(&o.manifest, &a)?;
                }
                let (fragment_count, extracted_bytes) = if let Some(o) = &ocr {
                    (o.chunks.len() as u32, o.manifest.representation_bytes)
                } else if let Some(x) = &xlsx {
                    (x.chunks.len() as u32, x.manifest.representation_bytes)
                } else if let Some(p) = &pdf {
                    (p.fragments.len() as u32, p.manifest.extracted_bytes)
                } else {
                    (parts.len() as u32, c.job.source_bytes)
                };
                #[cfg(test)]
                if let Some(h) = hook.as_mut() {
                    h("extracted");
                }
                #[cfg(test)]
                if let Some(h) = hook.as_mut() {
                    h("before_publish");
                }
                b.check()?;
                let tx = store::immediate(&self.conn)?;
                #[cfg(test)]
                if let Some(h) = hook.as_mut() {
                    h("publish_locked");
                }
                fence(&tx, &self.path, &c, &b)?;
                let a = store::job_source(&tx, &c.job)?;
                store::eligible_for(&a, &c.job.extractor)?;
                if let Some(o) = &ocr {
                    ocr_storage::source_format(&o.manifest, &a)?;
                }
                store::source(&self.path, &a, || fence(&tx, &self.path, &c, &b))?;
                if !store::supported(&c.job.extractor, c.job.extractor_version, &c.job.config) {
                    return Err(store::invalid("unsupported_extractor_config"));
                }
                let retained = ocr_storage::retained(&tx, &c.job.id)?;
                if retained
                    .checked_add(u64::from(extracted_bytes))
                    .is_none_or(|n| n > MAX_DERIVED_BYTES)
                {
                    return Err(store::invalid("derived_size_limit"));
                }
                tx.execute(
                    "UPDATE extraction_jobs SET generation_id=NULL WHERE id=?1",
                    [&c.job.id],
                )?;
                tx.execute(
                    "DELETE FROM extraction_generations WHERE job_id=?1",
                    [&c.job.id],
                )?;
                let gen_id = workspace::id();
                let now = workspace::now();
                tx.execute("INSERT INTO extraction_generations(id,job_id,attachment_id,source_sha256,extractor,extractor_version,config,config_hash,source_bytes,covered_bytes,fragment_count,published_at,attempt) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?9,?10,?11,?12)",params![gen_id,c.job.id,c.job.attachment_id,c.job.source_sha256,c.job.extractor,c.job.extractor_version,serde_json::to_string(&c.job.config)?,c.job.config_hash,c.job.source_bytes,fragment_count,now,c.job.attempts])?;
                if let Some(pdf) = &pdf {
                    pdf_storage::publish(&tx, &gen_id, &c.job.id, pdf)?;
                }
                if let Some(xlsx) = &xlsx {
                    xlsx_storage::publish(&tx, &gen_id, &c.job.id, xlsx)?;
                }
                if let Some(o) = &ocr {
                    ocr_storage::publish(&tx, &gen_id, &c.job.id, o)?;
                }
                {
                    let mut stmt = tx.prepare(
                        "INSERT INTO extraction_fragments VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                    )?;
                    for p in &parts {
                        b.check()?;
                        stmt.execute(params![
                            gen_id,
                            c.job.id,
                            p.ordinal,
                            p.text,
                            p.byte_start,
                            p.byte_end,
                            p.line_start,
                            p.line_end
                        ])?;
                    }
                }
                // Comprobar fencing mientras el job/reserva todavía están running.
                fence(&tx, &self.path, &c, &b)?;
                tx.execute("UPDATE extraction_jobs SET state='succeeded',revision=?2,generation_id=?3,updated_at=?4,finished_at=?4,error=NULL WHERE id=?1",params![c.job.id,store::bump(c.job.revision)?,gen_id,now])?;
                store::release(&tx)?;
                let out = ExtractionRun {
                    outcome: ExtractionOutcome::Succeeded,
                    reason: None,
                    job: Some(store::job(&tx, &self.path, &c.job.id)?),
                    recovered_count: c.recovered,
                    incompatible_count: c.incompatible,
                };
                store::response(&out)?;
                #[cfg(test)]
                if let Some(h) = hook.as_mut() {
                    h("before_commit");
                }
                b.check()?;
                if !c.owner.live(&store::scope(&tx, &self.path)?)? {
                    return Err(store::error(ErrorCode::WorkspaceBusy, "worker_interrupted"));
                }
                tx.commit()?;
                #[cfg(test)]
                if let Some(h) = hook.as_mut() {
                    h("committed");
                }
                Ok(out)
            })();
            result.map_err(|e| b.map(e))
        })();
        match result {
            Ok(out) => Ok(out),
            Err(e) => {
                #[cfg(test)]
                if let Some(h) = hook.as_mut() {
                    h("before_cleanup");
                }
                self.extraction_cleanup(&c, e)
            }
        }
    }
    fn extraction_cleanup(&self, c: &Claim, mut e: CoreError) -> Result<ExtractionRun> {
        if e.details["reason"] == "execution_deadline" {
            e.message = "Se agotó el presupuesto cooperativo de extracción local.".into();
        }
        if e.details.get("reason").is_none() {
            e.details["reason"] = json!(if matches!(e.code, ErrorCode::WorkspaceBusy) {
                "storage_contention"
            } else {
                "storage_failure"
            });
        }
        let cleanup: Result<(ExtractionJob, bool)> = (|| {
            let tx = store::immediate(&self.conn)?;
            let j = store::job(&tx, &self.path, &c.job.id)?;
            let o = store::owner(&tx)?;
            if j.state == ExtractionState::Running
                && j.revision == c.job.revision
                && o.as_ref().is_some_and(|o| {
                    o.token == c.owner.token
                        && o.scope == c.owner.scope
                        && o.control_revision == c.owner.control_revision
                })
            {
                store::fail(
                    &tx,
                    &j.id,
                    if e.details["reason"] == "input_stale" {
                        ExtractionState::Stale
                    } else {
                        ExtractionState::Failed
                    },
                    &e,
                )?;
                let j = store::job(&tx, &self.path, &j.id)?;
                store::response(&j)?;
                tx.commit()?;
                Ok((j, true))
            } else {
                Ok((j, false))
            }
        })();
        match cleanup {
            Ok((j, false)) => {
                let out = ExtractionRun {
                    outcome: ExtractionOutcome::Interrupted,
                    reason: Some("ownership_lost".into()),
                    job: Some(j),
                    recovered_count: c.recovered,
                    incompatible_count: c.incompatible,
                };
                store::response(&out)?;
                Ok(out)
            }
            Ok((j, true)) => Err(job_error(e, &j)),
            Err(_) => {
                e.details["cleanup_pending"] = json!(true);
                Err(job_error(e, &c.job))
            }
        }
    }
}
