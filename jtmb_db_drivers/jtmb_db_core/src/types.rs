use crate::Decode;


macro_rules! t_newtype {
    ( $($t:ident,$ty:ty),*) => {
        $(
            #[derive(PartialEq,Eq,Clone,Debug)]
            #[repr(transparent)]
            pub struct $t($ty);
            impl std::ops::Deref for $t {
                type Target = $ty;
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }
            impl std::convert::From<$ty> for $t {
                fn from(value: $ty) -> $t {
                    Self(value)
                }
            }
            impl std::ops::DerefMut  for $t {
                // type Target=$ty;
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.0
                }
            }
        )*
    }
} 
t_newtype!(
    Int8,i8,
    Int16,i16,
    Int32,i32,
    Int64,i64,
    Int128,i128,

    Uint8,u8,
    Uint16,u16,
    Uint32,u32,
    Uint64,u64,
    Uint128,u128
);

