/// just the contracts

/// can driver construct RustType from WireType using Endianness byteroder
pub mod types;
pub mod query;
pub mod prepare;
pub mod connection;
pub mod execute;
pub mod fetch;
pub mod encode_decode;

// a marker struct that represents byteorder

use std::{fmt::Debug, io, marker::PhantomData, ops::Deref, sync::Arc};

use tokio::{io::{AsyncRead, AsyncWrite}, sync::oneshot};

use crate::prepare::ClientPrepareMessage;




#[derive(Debug,Clone)]
#[repr(transparent)]
pub struct SharedString(Arc<str>);

impl Deref for SharedString {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}


/// our 'connection' type. talks to our background thread or task that we manage
pub struct Query;



pub struct ConnectionAborted;

pub struct Retry;






pub struct ConnectionHandle<Request,Response,Phase> 
    where Self: 'static
{
    inner:tokio::sync::oneshot::Sender<(Request,tokio::sync::oneshot::Sender<Response>)>,
    phase:Phase
}


// pub trait Connect
//     where Self: 'static
// {
//     type Stream: AsyncRead + AsyncWrite + Unpin + Send + 'static;

//     async fn connect(&self) -> std::io::Result<Self::Stream>;
// }

pub trait RawPipe {
    fn recv_packet(&mut self) -> impl Future<Output=io::Result<Arc<[u8]>>> + use<'_,Self>;
    fn send_packet(&mut self) -> impl Future<Output=io::Result<Arc<[u8]>>> + use<'_,Self>;
}


/// impl D Decode for D for any T that implementd DecodeT
/// to add more support, create additional traits, blanket decode them
/// then D is the thing that allows D -> T
// 1. THE GULP: Only the Protocol implements this.




pub struct Row<Protocol> {
    data: Arc<[u8]>,
    metadata: Arc<Protocol>
}
impl<ProtocolStruct> Row<ProtocolStruct> where ProtocolStruct: Protocol {
    fn get<RustType>(&self,index:usize) -> std::io::Result<RustType>
        // where DatabaseType: 'static
     {
        let c = self.metadata.decode_column::<RustType>(self.data.clone(),index)?;
        Ok(c)
    }
    fn get_by_name<RustType>(&self,index: usize) -> std::io::Result<RustType> {
        self.metadata.decode_column::<RustType>(self.data.clone(),index)
    }
    fn extract_column<RustType,DatabaseType>(&self,index: usize) -> std::io::Result<RustType> {
        self.metadata.as_ref().decode_column(self.data.clone(),index)
    }
}


// the protocol trait decribes how to shuffle bytes for the decode function and
// must guarentee that it can understand rust type T and Database Type D
// and convert from to




pub trait Protocol
    where Self: Sized +'static + Debug
    
{
    type InnerConnection;
    type ByteOrder;
    type ConnectOptions;


    fn connect(&self, options: &Self::ConnectOptions)-> impl Future<Output =  crate::connection::Connection<Self::InnerConnection>> + use<'_,Self>;
    fn close_connection<T>(&self,connection:&Self::InnerConnection) -> std::io::Result<()>;
    // called by drop whenever close returns an error
    fn on_close_fail(&self,error:std::io::Error) {
        println!("WARNING: Failed to close connection because of an error: {error}");
        println!("{self:?}")
    }
    fn spawn(addr:&str)-> impl Future<Output = std::io::Result<tokio::task::JoinHandle<()>>> + use<'_,Self>;
    fn fetch_packet_from_reader<R: std::io::Read>(&mut self,reader: &mut R) -> std::io::Result<Arc<[u8]>>;
    fn gulp<DatabaseType>(&mut self, data: Arc<[u8]>) -> std::io::Result<DatabaseType>
    where DatabaseType: Sized + 'static;

    // decode logic
    fn decode_column<RustType>(&self,data: Arc<[u8]>,index:usize) -> std::io::Result<RustType>;
    fn decode_column_named<RustType>(&self,data: Arc<[u8]>,index:usize) -> std::io::Result<RustType>;
    // prepare_logic
    fn prepare_sql(&mut self,sql: &str) -> std::io::Result<()>;
}


#[cfg(test)]
fn test() {}