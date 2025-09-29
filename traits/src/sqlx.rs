//! Implement [`sqlx`] traits around the [`crate::AssocId<T, RawId>`] wrapper.

impl<T, RawId, DB: sqlx::Database> sqlx::Type<DB> for crate::AssocId<T, RawId>
where
    RawId: sqlx::Type<DB>,
{
    fn type_info() -> <DB as sqlx::Database>::TypeInfo {
        RawId::type_info()
    }
}

impl<'a, T, RawId, DB: sqlx::Database> sqlx::Encode<'a, DB> for crate::AssocId<T, RawId>
where
    RawId: sqlx::Encode<'a, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as sqlx::Database>::ArgumentBuffer<'a>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        RawId::encode_by_ref(&self.inner, buf)
    }
}

impl<'a, T, RawId, DB: sqlx::Database> sqlx::Decode<'a, DB> for crate::AssocId<T, RawId>
where
    RawId: sqlx::Decode<'a, DB>,
{
    fn decode(
        value: <DB as sqlx::Database>::ValueRef<'a>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        Ok(Self::new(RawId::decode(value)?))
    }
}

#[cfg(feature = "sqlx_pg_array")]
impl<T, RawId> sqlx::postgres::PgHasArrayType for crate::AssocId<T, RawId>
where
    RawId: sqlx::postgres::PgHasArrayType,
{
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        RawId::array_type_info()
    }
}
