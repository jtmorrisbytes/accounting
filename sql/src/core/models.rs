#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub struct PasskeyId(crate::core::Id);


impl std::ops::Deref for PasskeyId {
    type Target = crate::core::Id;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


#[derive(Debug,Clone,PartialEq,Eq)]
#[cfg_attr(feature = "serialize",derive(serde::Serialize))]
#[cfg_attr(feature = "deserialize",derive(serde::Deserialize))]

pub struct Passkey {
    pub id: PasskeyId,
    pub credential_id: Vec<u8>,
    pub public_key: Vec<u8>,
    pub sign_count: i32
}



// the external user structure
#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub struct UserId(crate::core::Id);

impl std::ops::Deref for UserId {
    type Target = crate::core::Id;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
