use crate::fdw::qdrant_fdw::qdrant_client::{QdrantClient, QdrantClientError};
use pgrx::pg_sys;
use pgrx::pg_sys::panic::ErrorReport;
use std::collections::HashMap;
use supabase_wrappers::interface::{Column, Limit, Qual, Row, Sort};
use supabase_wrappers::prelude::*;
use supabase_wrappers::wrappers_fdw;
use thiserror::Error;

#[wrappers_fdw(
    version = "0.1.0",
    author = "Supabase",
    website = "https://github.com/supabase/wrappers/tree/main/wrappers/src/fdw/qdrant_fdw",
    error_type = "QdrantFdwError"
)]
pub(crate) struct QdrantFdw {
    qdrant_client: QdrantClient,
}

#[derive(Error, Debug)]
enum QdrantFdwError {
    #[error("{0}")]
    OptionsError(#[from] OptionsError),

    #[error("{0}")]
    QdrantClientError(#[from] QdrantClientError),
}

impl From<QdrantFdwError> for ErrorReport {
    fn from(value: QdrantFdwError) -> Self {
        match value {
            QdrantFdwError::OptionsError(e) => e.into(),
            QdrantFdwError::QdrantClientError(e) => e.into(),
        }
    }
}

impl ForeignDataWrapper<QdrantFdwError> for QdrantFdw {
    fn new(options: &HashMap<String, String>) -> Result<Self, QdrantFdwError>
    where
        Self: Sized,
    {
        let api_url = require_option("api_url", options)?;
        let api_key = require_option("api_key", options)?;

        Ok(Self {
            qdrant_client: QdrantClient::new(api_url, api_key)?,
        })
    }

    fn begin_scan(
        &mut self,
        _quals: &[Qual],
        _columns: &[Column],
        _sorts: &[Sort],
        _limit: &Option<Limit>,
        options: &HashMap<String, String>,
    ) -> Result<(), QdrantFdwError> {
        let collection_name = require_option("collection_name", options)?;
        let _response = self.qdrant_client.fetch_collection(collection_name)?;
        Ok(())
    }

    fn iter_scan(&mut self, _row: &mut Row) -> Result<Option<()>, QdrantFdwError> {
        Ok(None)
    }

    fn end_scan(&mut self) -> Result<(), QdrantFdwError> {
        Ok(())
    }

    fn validator(
        options: Vec<Option<String>>,
        catalog: Option<pg_sys::Oid>,
    ) -> Result<(), QdrantFdwError> {
        if let Some(oid) = catalog {
            if oid == FOREIGN_SERVER_RELATION_ID {
                check_options_contain(&options, "api_url")?;
                check_options_contain(&options, "api_key")?;
            } else if oid == FOREIGN_TABLE_RELATION_ID {
                check_options_contain(&options, "collection_name")?;
            }
        }

        Ok(())
    }
}
