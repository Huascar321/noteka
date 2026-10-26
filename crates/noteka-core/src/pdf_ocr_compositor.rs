//! Composición privada PDF→raster→Vision. Aún sin operación pública ni persistencia.
use crate::{
    batch_storage::Budget,
    error::*,
    extraction_storage as store,
    pdf_raster_protocol::{self as raster, RasterIdentity},
};
use noteka_ocr_protocol::v2::{OcrImageFormat, OcrRecognition};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub(crate) const PROFILE: &str = "apple_cgpdf_vision_r3_fast_es_en_v1";
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Page {
    pub raster: RasterIdentity,
    pub recognition: OcrRecognition,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Document {
    pub version: u16,
    pub profile: String,
    pub raster_profile: String,
    pub source_sha256: String,
    pub pages: Vec<Page>,
}
impl Document {
    pub(crate) fn validate(&self) -> Result<()> {
        let invalid = || store::invalid("pdf_ocr_document_invalid");
        if self.version != 1
            || self.profile != PROFILE
            || self.raster_profile != raster::PROFILE
            || !raster::valid_hash(&self.source_sha256)
            || self.pages.is_empty()
            || self.pages.len() > 20
        {
            return Err(invalid());
        }
        let first = &self.pages[0].raster;
        let mut pixels = 0u64;
        for (i, p) in self.pages.iter().enumerate() {
            if p.raster.clone().normalize().map_err(|_| invalid())? != p.raster
                || p.raster.page_number as usize != i + 1
                || p.raster.page_count as usize != self.pages.len()
                || p.raster.total_pixels != first.total_pixels
                || p.raster.os_version != first.os_version
                || p.raster.os_build != first.os_build
            {
                return Err(invalid());
            }
            p.recognition.validate().map_err(|_| invalid())?;
            let image = &p.recognition.image;
            if image.format != OcrImageFormat::Png
                || image.orientation != 1
                || (image.width, image.height) != (p.raster.width, p.raster.height)
                || p.recognition.engine.os_version != p.raster.os_version
                || p.recognition.engine.os_build != p.raster.os_build
            {
                return Err(invalid());
            }
            pixels += u64::from(image.width) * u64::from(image.height);
        }
        if pixels != first.total_pixels || pixels > raster::MAX_DOCUMENT_PIXELS {
            return Err(invalid());
        }
        if serde_json::to_vec(self)?.len() > raster::MAX_SOURCE {
            return Err(store::invalid("pdf_ocr_result_limit"));
        }
        Ok(())
    }
}
pub(crate) fn run(
    bytes: &[u8],
    budget: &Budget<'_>,
    mut check: impl FnMut() -> Result<()>,
) -> Result<Document> {
    compose(
        bytes,
        budget,
        &mut check,
        |page, b, guard| crate::pdf_raster_supervisor::run(bytes, page, b, guard),
        |png, b, guard| crate::ocr_supervisor::run_sandboxed(png, b, guard),
    )
}
type Guard<'a> = &'a mut dyn FnMut() -> Result<()>;
pub(crate) fn compose(
    bytes: &[u8],
    budget: &Budget<'_>,
    check: &mut dyn FnMut() -> Result<()>,
    mut render: impl FnMut(u32, &Budget<'_>, Guard<'_>) -> Result<raster::Raster>,
    mut recognize: impl FnMut(&[u8], &Budget<'_>, Guard<'_>) -> Result<OcrRecognition>,
) -> Result<Document> {
    if bytes.is_empty() || bytes.len() > raster::MAX_SOURCE {
        return Err(store::invalid("source_size_limit"));
    }
    let mut doc = Document {
        version: 1,
        profile: PROFILE.into(),
        raster_profile: raster::PROFILE.into(),
        source_sha256: format!("{:x}", Sha256::digest(bytes)),
        pages: Vec::new(),
    };
    let mut page = 1;
    let mut observed_pixels = 0u64;
    loop {
        crate::scoped::check_shutdown()?;
        budget.check()?;
        check()?;
        let raster = render(page, budget, check)?;
        let identity = raster
            .identity
            .clone()
            .normalize()
            .map_err(|_| store::storage("pdf_ocr_raster_worker_protocol"))?;
        if identity != raster.identity
            || identity.page_number != page
            || identity.png_bytes as usize != raster.png.len()
            || identity.png_sha256 != format!("{:x}", Sha256::digest(&raster.png))
            || doc.pages.first().is_some_and(|p| {
                p.raster.page_count != identity.page_count
                    || p.raster.total_pixels != identity.total_pixels
                    || p.raster.os_version != identity.os_version
                    || p.raster.os_build != identity.os_build
            })
        {
            return Err(store::storage("pdf_ocr_raster_worker_protocol"));
        }
        let recognition = recognize(&raster.png, budget, check)?
            .normalize()
            .map_err(|_| store::storage("pdf_ocr_recognition_protocol"))?;
        observed_pixels += u64::from(identity.width) * u64::from(identity.height);
        let count = identity.page_count;
        doc.pages.push(Page {
            raster: identity,
            recognition,
        });
        // Acotar crecimiento antes de empezar la página siguiente, sin publicar prefijos.
        if observed_pixels > raster::MAX_DOCUMENT_PIXELS
            || serde_json::to_vec(&doc)?.len() > raster::MAX_SOURCE
        {
            return Err(store::invalid("pdf_ocr_result_limit"));
        }
        crate::scoped::check_shutdown()?;
        budget.check()?;
        check()?;
        if page == count {
            break;
        }
        page += 1;
    }
    doc.validate()?;
    budget.check()?;
    check()?;
    Ok(doc)
}
