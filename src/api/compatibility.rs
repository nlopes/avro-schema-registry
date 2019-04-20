use actix_web::{
    web::{Data, Json, Path},
    HttpResponse,
};

use crate::api::SchemaBody;
use crate::db::DbPool;

pub fn check_compatibility(
    info: Path<(String, u32)>,
    body: Json<SchemaBody>,
    _data: Data<DbPool>,
) -> HttpResponse {
    let (subject, version) = info.into_inner();
    let _schema = body.into_inner().schema;
    info!("method=post,subject={},version={}", subject, version);

    HttpResponse::NotImplemented().finish()
}
