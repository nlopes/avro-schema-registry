use std::str::FromStr;

use actix_web::{
    web::{Data, Json, Path},
    HttpResponse, Responder,
};
use log::info;
use serde::Serialize;

use crate::api::errors::{ApiAvroErrorCode, ApiError};
use crate::api::SchemaBody;
use crate::db::models::{CompatibilityLevel, Config, Schema};
use crate::db::{DbManage, DbPool};

pub async fn check_compatibility(
    info: Path<(String, u32)>,
    body: Json<SchemaBody>,
    db: Data<DbPool>,
) -> impl Responder {
    let (subject, version) = info.into_inner();
    let schema = body.into_inner().schema;
    info!("method=post,subject={},version={}", subject, version);

    let mut conn = db.connection()?;
    let sv_response = crate::api::subjects::get_subject_version_from_db(
        &mut conn,
        subject.clone(),
        Some(version),
    )?;
    let compatibility = Config::get_with_subject_name(&mut conn, subject)?;
    if let Ok(compat) = CompatibilityLevel::from_str(&compatibility) {
        if let Ok(is_compatible) =
            SchemaCompatibility::is_compatible(&sv_response.schema, &schema, compat)
        {
            Ok(HttpResponse::Ok().json(SchemaCompatibility { is_compatible }))
        } else {
            Err(ApiError::new(ApiAvroErrorCode::InvalidAvroSchema))
        }
    } else {
        Err(ApiError::new(ApiAvroErrorCode::InvalidAvroSchema))
    }
}

#[derive(Debug, Serialize)]
struct SchemaCompatibility {
    is_compatible: bool,
}

impl SchemaCompatibility {
    fn is_compatible(
        old: &str,
        new: &str,
        compatibility: CompatibilityLevel,
    ) -> Result<bool, ApiError> {
        match compatibility {
            CompatibilityLevel::CompatNone => Ok(true),
            CompatibilityLevel::Backward => Schema::is_compatible(new, old),
            CompatibilityLevel::Forward => Schema::is_compatible(old, new),
            CompatibilityLevel::Full => {
                Ok(Schema::is_compatible(old, new)? && Schema::is_compatible(new, old)?)
            }
            _ => unimplemented!(),
        }
    }
}
