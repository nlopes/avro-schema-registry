use std::fmt;
use std::str;

use actix::Message;
use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::api::errors::{ApiError, ApiErrorCode};

use super::schema::*;
use super::Subject;

#[derive(Debug, Identifiable, Queryable, Associations, Serialize)]
#[table_name = "configs"]
#[belongs_to(Subject)]
pub struct Config {
    pub id: i64,
    pub compatibility: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub subject_id: Option<i64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CompatibilityLevel {
    Backward,
    BackwardTransitive,
    Forward,
    ForwardTransitive,
    Full,
    FullTransitive,
    #[serde(rename = "NONE")]
    CompatNone,
    #[serde(other)]
    Unknown,
}

impl fmt::Display for CompatibilityLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let screaming_snake_case = match self {
            CompatibilityLevel::Backward => Ok("BACKWARD"),
            CompatibilityLevel::BackwardTransitive => Ok("BACKWARD_TRANSITIVE"),
            CompatibilityLevel::Forward => Ok("FORWARD"),
            CompatibilityLevel::ForwardTransitive => Ok("FORWARD_TRANSITIVE"),
            CompatibilityLevel::Full => Ok("FULL"),
            CompatibilityLevel::FullTransitive => Ok("FULL_TRANSITIVE"),
            CompatibilityLevel::CompatNone => Ok("NONE"),
            // This won't ever be parsed, so we're fine by leaving this empty
            _ => Ok(""),
        }?;
        write!(f, "{}", screaming_snake_case)
    }
}

impl str::FromStr for CompatibilityLevel {
    type Err = ();

    fn from_str(s: &str) -> Result<CompatibilityLevel, ()> {
        match s {
            "BACKWARD" => Ok(CompatibilityLevel::Backward),
            "BACKWARD_TRANSITIVE" => Ok(CompatibilityLevel::BackwardTransitive),
            "FORWARD" => Ok(CompatibilityLevel::Forward),
            "FORWARD_TRANSITIVE" => Ok(CompatibilityLevel::ForwardTransitive),
            "FULL" => Ok(CompatibilityLevel::Full),
            "FULL_TRANSITIVE" => Ok(CompatibilityLevel::FullTransitive),
            "NONE" => Ok(CompatibilityLevel::CompatNone),
            _ => Err(()),
        }
    }
}

impl CompatibilityLevel {
    /// Returns [`Ok`] value of `self` if the `CompatibilityLevel is valid, otherwise
    /// returns the [`Err`] of `InvalidCompatibilitylevel`
    ///
    /// [`Ok`]: enum.Result.html#variant.Ok
    /// [`Err`]: enum.Result.html#variant.Err
    pub fn valid(&self) -> Result<CompatibilityLevel, ApiError> {
        ConfigCompatibility::new(self.to_string()).and(Ok(*self))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigCompatibility {
    pub compatibility: CompatibilityLevel,
}

impl ConfigCompatibility {
    pub fn new(level: String) -> Result<ConfigCompatibility, ApiError> {
        match level.parse::<CompatibilityLevel>() {
            Ok(l) => Ok(Self { compatibility: l }),
            Err(_) => Err(ApiError::new(ApiErrorCode::InvalidCompatibilityLevel)),
        }
    }
}

pub struct GetConfig;
impl Message for GetConfig {
    type Result = Result<ConfigCompatibility, ApiError>;
}

// Just to be clearer when we're implementing the Handler
pub type SetConfig = ConfigCompatibility;

impl Message for SetConfig {
    type Result = Result<ConfigCompatibility, ApiError>;
}

pub struct GetSubjectConfig {
    pub subject: String,
}

impl Message for GetSubjectConfig {
    type Result = Result<ConfigCompatibility, ApiError>;
}

pub struct SetSubjectConfig {
    pub subject: String,
    pub compatibility: CompatibilityLevel,
}

impl Message for SetSubjectConfig {
    type Result = Result<ConfigCompatibility, ApiError>;
}

impl Config {
    pub const DEFAULT_COMPATIBILITY: CompatibilityLevel = CompatibilityLevel::Backward;

    /// Retrieves the global compatibility level
    ///
    /// *NOTE*: if there is no global compatibility level, it sets it to the default
    /// compatibility level
    pub fn get_global_compatibility(conn: &PgConnection) -> Result<String, ApiError> {
        use super::schema::configs::dsl::*;
        match configs.filter(id.eq(0)).get_result::<Config>(conn) {
            // This should always return ok. If it doesn't, that means someone manually
            // edited the configs entry with id 0. Not only that, but they set the column
            // compatibility to NULL. Because of that, we don't try to fix it (although we
            // probably could) and instead return an internal server error.
            Ok(config) => config
                .compatibility
                .ok_or(ApiError::new(ApiErrorCode::BackendDatastoreError)),
            Err(diesel::result::Error::NotFound) => {
                // If we didn't find an entry with id 0, then this is either:
                //
                // a) first time we try to get a config, so we should set a default
                // b) database was manually modified and we should set a default again
                Config::insert(&Self::DEFAULT_COMPATIBILITY.to_string(), conn)?;
                Ok(Self::DEFAULT_COMPATIBILITY.to_string())
            }
            _ => Err(ApiError::new(ApiErrorCode::BackendDatastoreError)),
        }
    }

    pub fn get_with_subject_name(
        conn: &PgConnection,
        subject_name: String,
    ) -> Result<String, ApiError> {
        let subject = Subject::get_by_name(conn, subject_name)?;
        match Config::belonging_to(&subject).get_result::<Config>(conn) {
            // This should always return ok. If it doesn't, that means someone manually
            // edited the configs entry with id 0. Not only that, but they set the column
            // compatibility to NULL. Because of that, we don't try to fix it (although we
            // probably could) and instead return an internal server error.
            Ok(config) => config
                .compatibility
                .ok_or(ApiError::new(ApiErrorCode::BackendDatastoreError)),
            _ => Err(ApiError::new(ApiErrorCode::BackendDatastoreError)),
        }
    }

    pub fn set_with_subject_name(
        conn: &PgConnection,
        subject_name: String,
        compat: String,
    ) -> Result<String, ApiError> {
        use super::schema::configs::dsl::*;

        let subject = Subject::get_by_name(conn, subject_name)?;
        match Config::belonging_to(&subject).get_result::<Config>(conn) {
            Ok(config) => {
                match diesel::update(&config)
                    .set(compatibility.eq(&compat))
                    .get_result::<Config>(conn)
                {
                    Ok(conf) => conf
                        .compatibility
                        .ok_or(ApiError::new(ApiErrorCode::BackendDatastoreError)),
                    _ => Err(ApiError::new(ApiErrorCode::BackendDatastoreError)),
                }
            }
            Err(diesel::result::Error::NotFound) => {
                diesel::insert_into(configs)
                    .values((
                        compatibility.eq(&compat),
                        created_at.eq(diesel::dsl::now),
                        updated_at.eq(diesel::dsl::now),
                        subject_id.eq(subject.id),
                    ))
                    .execute(conn)
                    .or(Err(ApiError::new(ApiErrorCode::BackendDatastoreError)))?;
                Ok(compat)
            }
            _ => Err(ApiError::new(ApiErrorCode::BackendDatastoreError)),
        }
    }

    /// Updates the global compatibility level
    ///
    /// *NOTE*: if there is no global compatibility level, it sets it to the level passed
    pub fn set_global_compatibility(conn: &PgConnection, compat: &str) -> Result<String, ApiError> {
        use super::schema::configs::dsl::*;

        match diesel::update(configs.find(0))
            .set(compatibility.eq(compat))
            .get_result::<Config>(conn)
        {
            Ok(config) => config
                .compatibility
                .ok_or(ApiError::new(ApiErrorCode::BackendDatastoreError)),
            Err(diesel::result::Error::NotFound) => {
                // If we didn't find an entry with id 0, then this is either:
                //
                // a) first time we try to get a config, so we should set a default
                // b) database was manually modified and we should set a default again
                Config::insert(compat, conn)?;
                Ok(compat.to_string())
            }
            _ => Err(ApiError::new(ApiErrorCode::BackendDatastoreError)),
        }
    }

    fn insert(compat: &str, conn: &PgConnection) -> Result<usize, ApiError> {
        use super::schema::configs::dsl::*;

        diesel::insert_into(configs)
            .values((
                id.eq(0),
                compatibility.eq(&compat),
                created_at.eq(diesel::dsl::now),
                updated_at.eq(diesel::dsl::now),
            ))
            .execute(conn)
            .or(Err(ApiError::new(ApiErrorCode::BackendDatastoreError)))
    }
}
