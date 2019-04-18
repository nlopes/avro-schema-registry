use actix::Handler;

use crate::api::errors::ApiError;

use super::{
    Config, ConfigCompatibility, ConnectionPooler, GetConfig, GetSubjectConfig, SetConfig,
    SetSubjectConfig,
};

impl Handler<GetConfig> for ConnectionPooler {
    type Result = Result<ConfigCompatibility, ApiError>;

    fn handle(&mut self, _: GetConfig, _: &mut Self::Context) -> Self::Result {
        let conn = self.connection()?;
        Config::get_global_compatibility(&conn).and_then(ConfigCompatibility::new)
    }
}

impl Handler<SetConfig> for ConnectionPooler {
    type Result = Result<ConfigCompatibility, ApiError>;

    fn handle(&mut self, config: SetConfig, _: &mut Self::Context) -> Self::Result {
        let conn = self.connection()?;
        Config::set_global_compatibility(&conn, &config.compatibility.valid()?.to_string())
            .and_then(ConfigCompatibility::new)
    }
}

impl Handler<GetSubjectConfig> for ConnectionPooler {
    type Result = Result<ConfigCompatibility, ApiError>;

    fn handle(&mut self, config: GetSubjectConfig, _: &mut Self::Context) -> Self::Result {
        let conn = self.connection()?;
        Config::get_with_subject_name(&conn, config.subject).and_then(ConfigCompatibility::new)
    }
}

impl Handler<SetSubjectConfig> for ConnectionPooler {
    type Result = Result<ConfigCompatibility, ApiError>;

    fn handle(&mut self, config: SetSubjectConfig, _: &mut Self::Context) -> Self::Result {
        let conn = self.connection()?;
        Config::set_with_subject_name(
            &conn,
            config.subject,
            config.compatibility.valid()?.to_string(),
        )
        .and_then(ConfigCompatibility::new)
    }
}
