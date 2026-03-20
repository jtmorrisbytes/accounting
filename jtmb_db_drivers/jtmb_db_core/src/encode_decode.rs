use std::sync::Arc;

use crate::Protocol;

pub trait RawDecode {
    type ByteOrder;
    fn gulp(data: &[u8]) -> Self;
}

pub trait RawEncode {
    type ByteOrder;
    fn slurp(data: &[u8]) -> Self;
}

pub trait TryRawDecode<DB> {
    fn try_gulp(data: Arc<[u8]>) -> std::io::Result<DB>;
}




pub struct LittleEndian;
pub struct BigEndian;


// 2. The Decode trait just manages the 'Movement' (Protocol -> Data)
pub trait Decode<P: Protocol>: RawDecode
    where Self: Sized + 'static
{
    fn decode<R: std::io::Read>(bytes: Arc<[u8]>, _protocol: &P) -> std::io::Result<Self> {
        // let packet = protocol.fetch_packet_from_reader(reader)?;
        Ok(Self::gulp(&bytes))
    }
}

impl<P: Protocol, T: RawDecode> Decode<P> for T
    where Self:Sized + 'static

{}


pub trait Encode<P,T>
    where
    T: Sized + 'static,
    P: Sized + 'static + Protocol
{
    type ByteOrder;
    fn encode(value: T, protocol: &P) -> std::io::Result<&[u8]>{
            let bytes = value.raw_encode();
            Ok(bytes)
    }
}

