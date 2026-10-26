// Oráculo de prueba: cuerpo anterior a optimize-local-index-verification, sin cambios.
pub(crate) fn verify_reference<'a>(c: &'a Connection, b: &Budget<'_>) -> Result<Verified<'a>> {
    let revision = control(c)?.revision;
    structure(c)?;
    let result = (|| {
        let count: u32 =
            c.query_row("SELECT count(*) FROM extraction_search_entries", [], |r| {
                r.get(0)
            })?;
        let (fragments,bytes):(u32,u32) = c.query_row("SELECT count(*),coalesce(sum(length(CAST(text AS BLOB))),0) FROM extraction_search_fts",[],|r|Ok((r.get(0)?,r.get(1)?)))?;
        if count > MAX_INDEX_ENTRIES
            || fragments > MAX_INDEX_FRAGMENTS
            || u64::from(bytes) > MAX_INDEX_TEXT_BYTES
        {
            return Err(error("invalid_attachment_index"));
        }
        let raw:Vec<(u32,String,String,u32,u32,String,String)> = c.prepare("SELECT slot,job_id,generation_id,fragment_count,text_bytes,content_hash,indexed_at FROM extraction_search_entries ORDER BY job_id,generation_id,slot")?
            .query_map([],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?)))?.collect::<std::result::Result<_,_>>()?;
        c.execute_batch("DROP TABLE IF EXISTS temp.attachment_actual_vocab;DROP TABLE IF EXISTS temp.attachment_expected_vocab;DROP TABLE IF EXISTS temp.attachment_expected_fts;CREATE VIRTUAL TABLE temp.attachment_expected_fts USING fts5(job_id UNINDEXED,generation_id UNINDEXED,ordinal UNINDEXED,text,tokenize='unicode61',detail=full);")?;
        let mut verified = Verified {
            conn: c,
            entries: Vec::with_capacity(raw.len()),
        };
        let mut insert = c.prepare("INSERT INTO temp.attachment_expected_fts(rowid,job_id,generation_id,ordinal,text) VALUES (?1,?2,?3,?4,?5)")?;
        for (slot, job, generation, count, bytes, hash, at) in raw {
            b.check()?;
            if !(1..=MAX_INDEX_ENTRIES).contains(&slot)
                || DateTime::parse_from_rfc3339(&at).is_err()
            {
                return Err(error("invalid_attachment_index"));
            }
            let data = canonical(c, &job, b)?;
            if data.generation.id != generation
                || data.fragments.len() as u32 != count
                || data.text_bytes != bytes
                || data.hash != hash
            {
                return Err(error("invalid_attachment_index"));
            }
            for f in &data.fragments {
                b.check()?;
                insert.execute(params![
                    (slot - 1) * 512 + f.ordinal + 1,
                    job,
                    generation,
                    f.ordinal,
                    f.text
                ])?;
            }
            let entry = data.entry(at);
            verified.entries.push(Selected { slot, data, entry });
        }
        drop(insert);
        if !same_copies(c, b)? {
            return Err(error("invalid_attachment_index"));
        }
        c.execute_batch("CREATE VIRTUAL TABLE temp.attachment_actual_vocab USING fts5vocab('main','extraction_search_fts','instance');CREATE VIRTUAL TABLE temp.attachment_expected_vocab USING fts5vocab('temp','attachment_expected_fts','instance');")?;
        let fragments = verified
            .entries
            .iter()
            .flat_map(|s| {
                s.data
                    .fragments
                    .iter()
                    .map(move |f| ((s.slot - 1) * 512 + f.ordinal + 1, f.text.as_str()))
            })
            .collect::<Vec<_>>();
        if same_repeated_postings(c, b, &fragments)? != Some(true) && !same_postings(c, b)? {
            return Err(error("invalid_attachment_index"));
        }
        b.check()?;
        Ok(verified)
    })();
    result.map_err(|e| {
        let e = b.map(e);
        if matches!(e.code, ErrorCode::WorkspaceBusy) {
            return e;
        }
        let mut e = error("invalid_attachment_index");
        e.details["current_revision"] = json!(revision);
        e
    })
}

