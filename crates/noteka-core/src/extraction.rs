use crate::{
    attachments,
    contracts::*,
    error::*,
    extraction_storage as store,
    workspace::{self, Workspace},
};
use rusqlite::{OptionalExtension, params};
use serde_json::json;

impl Workspace {
    pub fn extraction_enqueue(&self, input: EnqueueExtraction) -> Result<EnqueuedExtraction> {
        store::uuid(&input.attachment_id)?;
        let config = serde_json::to_value(input.config.unwrap_or_default())?;
        if !store::supported(&input.extractor, 1, &config) {
            return Err(store::invalid("unsupported_extractor_config"));
        }
        let hash = store::config_hash(&config)?;
        let tx = store::immediate(&self.conn)?;
        let a = attachments::metadata(&tx, &input.attachment_id)?;
        store::eligible_for(&a, &input.extractor)?;
        let existing:Option<String>=tx.query_row("SELECT id FROM extraction_jobs WHERE attachment_id=?1 AND source_sha256=?2 AND extractor=?3 AND extractor_version=1 AND config_hash=?4",params![a.id,a.sha256,input.extractor,hash],|r|r.get(0)).optional()?;
        let id = if let Some(id) = existing.as_ref() {
            id.clone()
        } else {
            store::admit(&tx, true)?;
            store::source(&self.path, &a, || Ok(()))?;
            let id = workspace::id();
            let now = workspace::now();
            tx.execute("INSERT INTO extraction_jobs(id,attachment_id,source_sha256,source_bytes,extractor,extractor_version,config,config_hash,state,revision,attempts,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,1,?6,?7,'queued',1,0,?8,?8)",params![id,a.id,a.sha256,a.bytes as u32,input.extractor,serde_json::to_string(&config)?,hash,now])?;
            id
        };
        let out = EnqueuedExtraction {
            job: store::job(&tx, &self.path, &id)?,
            deduplicated: existing.is_some(),
        };
        store::response(&out)?;
        tx.commit()?;
        Ok(out)
    }
    pub fn extraction_get(&self, input: IdInput) -> Result<ExtractionJob> {
        store::uuid(&input.id)?;
        let tx = self.conn.unchecked_transaction()?;
        let out = store::job(&tx, &self.path, &input.id)?;
        store::response(&out)?;
        Ok(out)
    }
    pub fn extraction_list(&self, input: ListExtraction) -> Result<Page<ExtractionJob>> {
        if let Some(id) = &input.attachment_id {
            store::uuid(id)?;
        }
        let limit = store::page_limit(input.limit, 50, 100)?;
        let tx = self.conn.unchecked_transaction()?;
        let mut shape = store::Cursor::new(self, "extraction.list", limit)?;
        shape.attachment_id = input.attachment_id.clone();
        shape.state = input.state.clone();
        let cursor = if let Some(raw) = input.cursor {
            let c = store::Cursor::read(&raw)?;
            let mut blank = c.clone();
            blank.after_created = None;
            blank.after_id = None;
            if blank != shape
                || c.after_created
                    .as_ref()
                    .is_none_or(|d| chrono::DateTime::parse_from_rfc3339(d).is_err())
                || c.after_id
                    .as_ref()
                    .is_none_or(|id| store::uuid(id).is_err())
            {
                return Err(store::invalid("cursor_scope_mismatch"));
            }
            c
        } else {
            shape.clone()
        };
        let state = input.state.as_ref().map(ExtractionState::as_str);
        let ids:Vec<String>=tx.prepare("SELECT id FROM extraction_jobs WHERE (?1 IS NULL OR attachment_id=?1) AND (?2 IS NULL OR state=?2) AND (?3 IS NULL OR created_at>?3 OR (created_at=?3 AND id>?4)) ORDER BY created_at COLLATE BINARY,id COLLATE BINARY LIMIT ?5")?
            .query_map(params![input.attachment_id,state,cursor.after_created,cursor.after_id,limit+1],|r|r.get(0))?.collect::<std::result::Result<_,_>>()?;
        let more = ids.len() > limit as usize;
        let items = ids
            .iter()
            .take(limit as usize)
            .map(|id| store::job(&tx, &self.path, id))
            .collect::<Result<Vec<_>>>()?;
        let next_cursor = if more {
            let last = items.last().unwrap();
            shape.after_created = Some(last.created_at.clone());
            shape.after_id = Some(last.id.clone());
            Some(shape.encode()?)
        } else {
            None
        };
        let out = Page { items, next_cursor };
        store::response(&out)?;
        Ok(out)
    }
    pub fn extraction_control(&self, input: ExtractionControlInput) -> Result<ExtractionControl> {
        let ExtractionControlInput::Set {
            paused,
            expected_revision,
        } = input
        else {
            let tx = self.conn.unchecked_transaction()?;
            let out = store::control(&tx)?;
            store::response(&out)?;
            return Ok(out);
        };
        let tx = store::immediate(&self.conn)?;
        let old = store::control(&tx)?;
        store::expected(old.revision, expected_revision)?;
        if old.paused == paused {
            store::response(&old)?;
            tx.commit()?;
            return Ok(old);
        }
        // Recuperar con la revisión/control anteriores: nunca confundir cambio con lease expirado.
        store::recover(&tx, &self.path)?;
        if paused && let Some(o) = store::owner(&tx)? {
            let j = store::job(&tx, &self.path, &o.job_id)?;
            tx.execute("UPDATE extraction_jobs SET state='queued',revision=?2,updated_at=?3,finished_at=NULL,error=NULL WHERE id=?1",params![j.id,store::bump(j.revision)?,workspace::now()])?;
            store::release(&tx)?;
        }
        tx.execute(
            "UPDATE extraction_control SET paused=?1,revision=?2,updated_at=?3 WHERE id=1",
            params![paused, store::bump(old.revision)?, workspace::now()],
        )?;
        let out = store::control(&tx)?;
        store::response(&out)?;
        tx.commit()?;
        Ok(out)
    }
    pub fn extraction_cancel(&self, input: ExtractionRevision) -> Result<ExtractionJob> {
        store::uuid(&input.id)?;
        let tx = store::immediate(&self.conn)?;
        let j = store::job(&tx, &self.path, &input.id)?;
        store::expected(j.revision, input.expected_revision)?;
        if j.state != ExtractionState::Cancelled {
            if !matches!(j.state, ExtractionState::Queued | ExtractionState::Running) {
                return Err(store::invalid("invalid_state"));
            }
            tx.execute("UPDATE extraction_jobs SET state='cancelled',revision=?2,updated_at=?3,finished_at=?3,error=NULL WHERE id=?1",params![j.id,store::bump(j.revision)?,workspace::now()])?;
            if j.state == ExtractionState::Running {
                store::release(&tx)?;
            }
        }
        let out = store::job(&tx, &self.path, &input.id)?;
        store::response(&out)?;
        tx.commit()?;
        Ok(out)
    }
    pub fn extraction_retry(&self, input: ExtractionRevision) -> Result<ExtractionJob> {
        store::uuid(&input.id)?;
        let tx = store::immediate(&self.conn)?;
        let j = store::job(&tx, &self.path, &input.id)?;
        store::expected(j.revision, input.expected_revision)?;
        if !j.supported {
            return Err(store::invalid("unsupported_extractor_config"));
        }
        if !matches!(
            j.state,
            ExtractionState::Failed
                | ExtractionState::Cancelled
                | ExtractionState::Stale
                | ExtractionState::Succeeded
        ) {
            return Err(store::invalid("invalid_state"));
        }
        store::admit(&tx, false)?;
        tx.execute("UPDATE extraction_jobs SET state='queued',revision=?2,updated_at=?3,started_at=NULL,finished_at=NULL,error=NULL WHERE id=?1",params![j.id,store::bump(j.revision)?,workspace::now()])?;
        let out = store::job(&tx, &self.path, &j.id)?;
        store::response(&out)?;
        tx.commit()?;
        Ok(out)
    }
    pub fn extraction_fragments(&self, input: ReadFragments) -> Result<FragmentPage> {
        store::uuid(&input.id)?;
        let limit = store::page_limit(input.limit, 32, 64)?;
        let tx = self.conn.unchecked_transaction()?;
        let j = store::job(&tx, &self.path, &input.id)?;
        let g = j
            .generation
            .ok_or_else(|| store::error(ErrorCode::NotFound, "generation_not_found"))?;
        if g.extractor != "utf8_text"
            || !store::supported(&g.extractor, g.extractor_version, &g.config)
        {
            return Err(store::invalid("unsupported_generation_config"));
        }
        let mut shape = store::Cursor::new(self, "extraction.fragments", limit)?;
        shape.job_id = Some(input.id.clone());
        shape.generation_id = Some(g.id.clone());
        let from = if let Some(raw) = &input.cursor {
            let c = store::Cursor::read(raw)?;
            let mut blank = c.clone();
            blank.ordinal = None;
            if blank != shape || c.ordinal.is_none_or(|n| n == 0 || n >= g.fragment_count) {
                return Err(store::invalid("cursor_scope_mismatch"));
            }
            c.ordinal.unwrap()
        } else {
            0
        };
        let a = attachments::metadata(&tx, &g.attachment_id)?;
        store::eligible(&a)?;
        if a.sha256 != g.source_sha256 || a.bytes != u64::from(g.source_bytes) {
            return Err(store::storage("input_stale"));
        }
        let bytes = store::source(&self.path, &a, || Ok(()))?;
        let items:Vec<ExtractionFragment>=tx.prepare("SELECT ordinal,text,byte_start,byte_end,line_start,line_end FROM extraction_fragments WHERE generation_id=?1 AND ordinal>=?2 ORDER BY ordinal LIMIT ?3")?
            .query_map(params![g.id,from,limit],|r|Ok(ExtractionFragment {ordinal:r.get(0)?,text:r.get(1)?,byte_start:r.get(2)?,byte_end:r.get(3)?,line_start:r.get(4)?,line_end:r.get(5)?}))?.collect::<std::result::Result<_,_>>()?;
        // No certificar un slice corrupto aunque SQLite supere sus CHECKs.
        for (index, p) in items.iter().enumerate() {
            if p.ordinal != from + index as u32
                || p.byte_end <= p.byte_start
                || bytes.get(p.byte_start as usize..p.byte_end as usize) != Some(p.text.as_bytes())
                || p.text.len() > 4096
                || p.line_start
                    != 1 + bytes[..p.byte_start as usize]
                        .iter()
                        .filter(|&&b| b == b'\n')
                        .count() as u32
                || p.line_end
                    != 1 + bytes[..p.byte_end as usize - 1]
                        .iter()
                        .filter(|&&b| b == b'\n')
                        .count() as u32
            {
                return Err(store::storage("invalid_fragment"));
            }
        }
        let after = from + items.len() as u32;
        if items.len() as u32 != (g.fragment_count - from).min(limit) {
            return Err(store::storage("invalid_generation"));
        }
        let next_cursor = if after < g.fragment_count {
            shape.ordinal = Some(after);
            Some(shape.encode()?)
        } else {
            None
        };
        let out = FragmentPage {
            job_id: input.id,
            generation: g,
            result_from_previous_attempt: j.result_from_previous_attempt,
            covers_generation: input.cursor.is_none() && next_cursor.is_none(),
            items,
            next_cursor,
        };
        store::response(&out)?;
        Ok(out)
    }
}

pub(crate) fn job_error(mut e: CoreError, j: &ExtractionJob) -> CoreError {
    e.details["job_id"] = json!(j.id);
    e.details["state"] = serde_json::to_value(&j.state).unwrap();
    e.details["revision"] = json!(j.revision);
    e.details["attempt"] = json!(j.attempts);
    e
}
