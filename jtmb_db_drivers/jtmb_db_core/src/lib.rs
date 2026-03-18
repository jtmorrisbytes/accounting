/// just the contracts

/// can driver construct RustType from WireType using Endianness byteroder


// a marker struct that represents byteorder

use std::sync::Arc;


// seperates the io layer from the decode layer
// could be file, cursor, reader, etc
pub trait Transport{}


// wire protocol
pub struct BigEndian;

impl BigEndian {
    pub fn from_le_u8(){}
}

// tells rust how to move bytes to and from type T for wire protoocl
// each trait tells the library that you can decode T for any type that has a trait

pub trait DecodeI64 {

}

pub trait DecodeI32
    where Self:'static
{
    fn decode_i32(){}
}
pub trait DecodeI16
    where Self:'static
{}
pub trait DecodeI8
    where Self:'static
{}

pub trait DecodeU8
    where Self:'static
{}

pub trait DecodeU16
    where Self:'static
{}

pub trait DecodeU32
where Self:'static
{}

pub trait DecodeU64
where Self:'static
{}

pub trait DecodeU128
where Self:'static
{}

pub trait DecodeBytes
where Self:'static
{}

pub trait DecodeUuid
where Self: 'static {
    type Error: 'static;
    fn decode_uuid_from_bytes<B:AsRef<[u8]>>(b:B) -> Result<uuid::Uuid,Self::Error>;
}



/// impl D Decode for D for any T that implementd DecodeT
/// to add more support, create additional traits, blanket decode them
/// then D is the thing that allows D -> T



pub trait Decode<D>
    where Self: Sized + 'static
{
    // where Converter: Converter
    fn from_wire(data: Arc<[u8]>) -> Self;
}

pub trait TryDecode<D>
    where Self: Sized + 'static
{
    type Error;
    fn try_from_wire(data: Arc<[u8]>) -> Result<Self,Self::Error>;
}
// represents raw byte converions for pg wire protocol v3
pub struct PGWireProtocolV3;


impl<D> Decode<D> for i8 

    where D: DecodeI8 + 'static
{
     fn from_wire(data: Arc<[u8]>) -> Self {
         <D as DecodeI8>::decode_i8()
     }
}

impl<D> Decode<D> for i32
    where D: DecodeI32 + 'static,
{
    fn from_wire(data: Arc<[u8]>) -> Self {
        <D as DecodeI32>::decode_i32()
    }
}
impl<D> Decode<D> for i64
    where D:DecodeI64 + 'static,
{
    fn from_wire(data: Arc<[u8]>) -> Self {
        <D as DecodeI64>::decode_i64()
    }
}
impl<D> Decode<D> for i128
    where D:DecodeI128 + 'static {
        fn from_wire(data: Arc<[u8]>) -> Self {
            <D as DecodeI128>::decode_i128()
        }
}
impl<D> Decode<D> for u8 where D: DecodeU8 {
    fn from_wire(data: Arc<[u8]>) -> Self {
        <D as DecodeU8>::Decode_u8()
    }
}
impl<D> Decode<D> for u16 where D: DecodeU16 {
    fn from_wire(data: Arc<[u8]>) -> Self {
        <D as DecodeU16>::decode_u16()
    }
}
impl<D> Decode<D> for u32 where D: DecodeU32 {
    fn from_wire(data: Arc<[u8]>) -> Self {
        <D as DecodeU32>::decode_u32()
    }
}
impl<D> Deocde<D> for u64 where D: DecodeU64 {
    fn from_wire(data: Arc<[u8]>) -> Self {
        <D as DecodeU64>::decode_u64()
    }
}
impl<D> Decode<D> for u128 where D: DecodeU128 {
    fn from_wire(data: Arc<[u8]>) -> Self {
        <D as DecodeU128>::decode_u128()
    }
}
impl<D> Decode<D> for String where D: DecodeUtf8 {
    fn from_wire(data: Arc<[u8]>) -> Self {
        <D as DecodeUtf8>::decode_utf8()
    }
}

