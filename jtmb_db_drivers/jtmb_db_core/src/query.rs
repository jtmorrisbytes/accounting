// pub struct Query;

use std::{fmt::Debug, marker::PhantomData, sync::Arc};

use crate::{Protocol, connection::{Connected, ResponsePacket, SendMessageAndWaitForOneShot}, encode_decode::Encode, prepare::{CanPrepare, CantPrepare, PreparedStatamentCapabilities}};
use crate::encode_decode::Encode;

pub enum QueryProtocolMessage {
    QueryAs,
    Query
}


pub struct Query<P>
    where P:Protocol
{
    // sender: tokio::sync::mpsc::Receiver<ResponsePacket<R,P>>,
    // _m:PhantomData<M>,
    parameters:Vec<u8>,
    protocol: Arc<P>
}

impl<P> Query<P>
    where P:Protocol
{
    pub fn bind<T,DB>(&mut self,value:T) -> std::io::Result<&mut Self>
    where T: Encode<P,DB>,
    DB: 'static {
        let db_val = value.encode(&mut self.protocol)?;
        self.params.push(db_val);
        Ok(self) 
    }
}



// pub trait Query<P> 
//     where P: Protocol

// {
//     fn query(&self) -> impl Future<Output=()> + use<'_,Self,P>;
// }

pub trait QueryAs<P>
where P:Protocol


{}



impl<M,R,P> QueryAs<P> for Connected<M,R,P> where Connected<M,R,P>: SendMessageAndWaitForOneShot<M,R,P>,
    P: crate::Protocol + PreparedStatamentCapabilities<Mode=CantPrepare>


{
    
}



// impl<R,P> Query<P> for Query<QueryProtocolMessage,R,P> where Query<QueryProtocolMessage,R,P>: SendMessageAndWaitForOneShot<QueryProtocolMessage,R,P>,
// P:crate::Protocol + PreparedStatamentCapabilities<Mode = CanPrepare>, R: Debug
// // M: Into<QueryProtocolMessage>,
// // M: Default
// {
// fn query(&self) -> impl Future<Output=()> + use<'_,R,P> {
//     // let sender = self.sender.clone()
//     // let m = M::default();
//     // let sender = self.sender.clone();
//     // let m:  = 
//     async{
//         let r = self.send_message_and_wait_for_oneshot(QueryProtocolMessage::Query).await;
//         dbg!(r);
//     }
// }
// }

// impl<M,R,P> QueryAs<P> for Query<M,R,P> where Connected<M,R,P>: SendMessageAndWaitForOneShot<M,R,P>,
//     P: crate::Protocol + PreparedStatamentCapabilities<Mode=CanPrepare>


// {
    
// }
