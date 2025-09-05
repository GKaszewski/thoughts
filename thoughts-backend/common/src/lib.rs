use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::TryGetError;
use sea_orm::{sea_query::ColumnType, sea_query::Value, sea_query::ValueType, TryGetable};
use sea_query::ValueTypeErr;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema, Debug)]
#[schema(example = "2025-09-05T12:34:56Z")]
pub struct DateTimeWithTimeZoneWrapper(String);

impl From<DateTimeWithTimeZone> for DateTimeWithTimeZoneWrapper {
    fn from(value: DateTimeWithTimeZone) -> Self {
        DateTimeWithTimeZoneWrapper(value.to_rfc3339())
    }
}

impl TryGetable for DateTimeWithTimeZoneWrapper {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        index: I,
    ) -> Result<Self, TryGetError> {
        let value: String = res.try_get_by(index)?;
        Ok(DateTimeWithTimeZoneWrapper(value))
    }

    fn try_get(res: &sea_orm::QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let value: String = res.try_get(pre, col)?;
        Ok(DateTimeWithTimeZoneWrapper(value))
    }
}

impl ValueType for DateTimeWithTimeZoneWrapper {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        if let Value::String(Some(string)) = v {
            Ok(DateTimeWithTimeZoneWrapper(*string))
        } else {
            Err(ValueTypeErr)
        }
    }

    fn array_type() -> sea_query::ArrayType {
        sea_query::ArrayType::String
    }

    fn column_type() -> ColumnType {
        ColumnType::String(sea_query::StringLen::Max)
    }

    fn type_name() -> String {
        "DateTimeWithTimeZoneWrapper".to_string()
    }
}
