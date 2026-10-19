//! Infraestructura del proceso descartable; no se enlaza al núcleo ni al host.
mod codecs;
pub mod containment;
mod diagnostics;
mod framing;
pub mod parser;
mod predictors;
pub mod runtime;
mod streams;
mod text;
mod xref;
