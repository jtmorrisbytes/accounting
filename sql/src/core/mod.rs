use std::{marker::PhantomData, ops::Deref};

use sqlx::prelude::FromRow;
use uuid::Uuid;

pub mod models;


// where T: ParialEq
/// the primary ID type for the entire library.
#[derive(FromRow, PartialEq, Eq, Hash, Debug,Clone)]
#[sqlx(transparent)]
pub struct Id {
    pub value: uuid::Uuid,
    // #[sqlx(skip)]
    // _marker: std::marker::PhantomData<T>,
}
impl Id {
    pub fn new_v7() -> Self {
        Self {
            value: uuid::Uuid::now_v7(),
            // _marker: PhantomData,
        }
    }
}

// enables us to use the id type in multiple databases

impl<DB: sqlx::Database> sqlx::Type<DB> for Id
where
    uuid::Uuid: sqlx::Type<DB>,
{
    fn type_info() -> <DB as sqlx::Database>::TypeInfo {
        <uuid::Uuid as sqlx::Type<DB>>::type_info()
    }
}

impl<'q, DB: sqlx::Database> sqlx::Encode<'q, DB> for Id
where
    uuid::Uuid: sqlx::Encode<'q, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        <uuid::Uuid as sqlx::Encode<'q, DB>>::encode(self.value, buf)
    }
}
impl<'r, DB: sqlx::Database> sqlx::Decode<'r, DB> for Id
where
    uuid::Uuid: sqlx::Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let uuid = <uuid::Uuid as sqlx::Decode<'r, DB>>::decode(value)?;
        Ok(Self {
            value: uuid,
            // _marker: PhantomData,
        })
    }
}

impl std::ops::Deref for Id {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl std::convert::From<Vec<u8>> for Id {
    fn from(value: Vec<u8>) -> Self {
        let value = value.as_array().unwrap();
        Self {
            value: uuid::Uuid::from_bytes(*value),
        }
    }
}

impl Default for Id {
    fn default() -> Self {
        Self {
            value: uuid::Uuid::now_v7(),
            
        }
    }
}



pub trait GetId {
    fn id(&self) -> &Id
    where
        Self: Sized;
}

pub trait Entity: GetId {
    fn primary_key(&self) -> &Id
    where
        Self: Sized,
    {
        self.id()
    }
    fn entity_eq(&self, other: &Self) -> bool
    where
        Self: Sized,
    {
        self.id().eq(other.id())
    }
}

pub trait Table {
    const TABLE_NAME: &'static str;
}


impl<T> Entity for T
where
    T: GetId,
{
    fn primary_key(&self) -> &Id
    where
        Self: Sized,
    {
        self.id()
    }
}
