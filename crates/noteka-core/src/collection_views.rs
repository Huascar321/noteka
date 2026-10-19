use crate::{contracts::*, error::*};
use chrono::{Datelike, NaiveDate};
use serde_json::{Value, json};

fn property<'a>(props: &'a [Property], id: &str) -> Result<&'a Property> {
    props
        .iter()
        .find(|p| p.id == id)
        .ok_or_else(|| CoreError::validation("Presentación refiere propiedad ajena/desconocida."))
}
pub(crate) fn month(s: &str) -> Result<(NaiveDate, NaiveDate)> {
    if s.len() != 7
        || s.as_bytes()[4] != b'-'
        || !s
            .bytes()
            .enumerate()
            .all(|(i, b)| i == 4 || b.is_ascii_digit())
    {
        return Err(CoreError::validation(
            "Mes debe ser YYYY-MM, años 0001–9999.",
        ));
    }
    let year = s[..4]
        .parse::<i32>()
        .map_err(|_| CoreError::validation("Año inválido."))?;
    let m = s[5..]
        .parse::<u32>()
        .map_err(|_| CoreError::validation("Mes inválido."))?;
    let start = NaiveDate::from_ymd_opt(year, m, 1)
        .filter(|_| year >= 1)
        .ok_or_else(|| CoreError::validation("Mes inválido."))?;
    let end = (28..=31)
        .rev()
        .find_map(|day| NaiveDate::from_ymd_opt(year, m, day))
        .unwrap();
    Ok((start, end))
}
pub(crate) fn validate(layout: &ViewLayout, props: &[Property], cfg: &ViewConfig) -> Result<()> {
    match layout {
        ViewLayout::Board => {
            if cfg.calendar.is_some() {
                return Err(CoreError::validation("Board no admite calendar."));
            }
            let p = property(
                props,
                cfg.group_by
                    .as_deref()
                    .ok_or_else(|| CoreError::validation("Board requiere group_by."))?,
            )?;
            if !matches!(p.spec.kind, PropertyKind::Select | PropertyKind::Checkbox) {
                return Err(CoreError::validation("Board agrupa sólo select/checkbox."));
            }
        }
        ViewLayout::Calendar => {
            if cfg.group_by.is_some() {
                return Err(CoreError::validation("Calendar no admite group_by."));
            }
            let c = cfg
                .calendar
                .as_ref()
                .ok_or_else(|| CoreError::validation("Calendar requiere property_id/month."))?;
            let p = property(props, &c.property_id)?;
            if !matches!(p.spec.kind, PropertyKind::Date | PropertyKind::DateRange) {
                return Err(CoreError::validation("Calendar requiere date/date_range."));
            }
            month(&c.month)?;
        }
        _ => {
            if cfg.group_by.is_some() || cfg.calendar.is_some() {
                return Err(CoreError::validation(
                    "Este layout no admite agrupación/calendario.",
                ));
            }
        }
    }
    Ok(())
}
// Valores ya validados por el núcleo. YYYY-MM-DD conserva orden civil lexicográfico.
fn range<'a>(r: &'a RecordData, id: &str) -> Option<(&'a str, &'a str)> {
    let v = r.values.get(id)?;
    if let Some(date) = v.as_str() {
        Some((date, date))
    } else {
        Some((v.get("start")?.as_str()?, v.get("end")?.as_str()?))
    }
}
pub(crate) fn in_month(r: &RecordData, c: &CalendarConfig, start: &str, end: &str) -> bool {
    range(r, &c.property_id).is_none_or(|(a, b)| a <= end && b >= start)
}
pub(crate) fn groups(
    layout: &ViewLayout,
    props: &[Property],
    cfg: &ViewConfig,
    records: &[RecordData],
) -> Result<Vec<ViewGroup>> {
    let mut groups = vec![];
    match layout {
        ViewLayout::Board => {
            let id = cfg.group_by.as_ref().unwrap();
            let p = property(props, id)?;
            let keys = if p.spec.kind == PropertyKind::Checkbox {
                vec![json!(false), json!(true)]
            } else {
                p.spec.options.iter().map(|s| json!(s)).collect()
            };
            for key in keys.into_iter().chain([Value::Null]) {
                let label = if key.is_null() {
                    "Sin valor".into()
                } else if let Some(s) = key.as_str() {
                    s.into()
                } else {
                    key.to_string()
                };
                groups.push(ViewGroup {
                    entity_ids: records
                        .iter()
                        .filter(|r| r.values.get(id).unwrap_or(&Value::Null) == &key)
                        .map(|r| r.entity.id.clone())
                        .collect(),
                    key,
                    label,
                });
            }
        }
        ViewLayout::Calendar => {
            let c = cfg.calendar.as_ref().unwrap();
            let (start, end) = month(&c.month)?;
            for day in 1..=end.day() {
                let date = NaiveDate::from_ymd_opt(start.year(), start.month(), day)
                    .unwrap()
                    .to_string();
                let entity_ids = records
                    .iter()
                    .filter(|r| {
                        range(r, &c.property_id)
                            .is_some_and(|(a, b)| a <= date.as_str() && b >= date.as_str())
                    })
                    .map(|r| r.entity.id.clone())
                    .collect();
                groups.push(ViewGroup {
                    key: json!(date),
                    label: date,
                    entity_ids,
                });
            }
            groups.push(ViewGroup {
                key: Value::Null,
                label: "Sin fecha".into(),
                entity_ids: records
                    .iter()
                    .filter(|r| range(r, &c.property_id).is_none())
                    .map(|r| r.entity.id.clone())
                    .collect(),
            });
        }
        _ => {}
    }
    Ok(groups)
}
