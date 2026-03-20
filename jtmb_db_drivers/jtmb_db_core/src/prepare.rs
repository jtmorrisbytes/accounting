use crate::{ConnectionAborted, ConnectionHandle, Protocol, Query, SharedString, connection::{Connected, ResponsePacket, SendMessageAndWaitForOneShot}};


#[derive(Debug)]
pub enum ClientPrepareMessage {
    PrepareSql{sql: SharedString},
}
pub enum ClientPrepareResponse {
    /// the prepare succeeded
    Ok(tokio::sync::oneshot::Sender<()>),
    /// the connection aborted or the socket hung up
    ConnectionAborted(tokio::sync::oneshot::Sender<()>),
    // reserved
}

pub enum PrepareResult {
    ReadyForQuery(ConnectionHandle<(),(),Query>),
    ConnectionAborted(ConnectionHandle<(),(),ConnectionAborted>),
    BackgroundTheadAbortedOrHungUp{fatal_error: String}
}

pub struct PrepareHandle {
    inner: tokio::sync::oneshot::Sender<(ClientPrepareMessage,tokio::sync::oneshot::Sender<ClientPrepareResponse>)>
}
impl PrepareHandle {

}



pub trait Prepare

{
    fn prepare(& self,s: SharedString) -> impl Future<Output= PrepareResult> + Send + use<'_,Self>;
}




// impl Prepare for Connected<ClientPrepareMessage,ClientPrepareResponse>
// where Connected<ClientPrepareMessage,ClientPrepareResponse> : 
// SendMessageAndWaitForOneShot<ClientPrepareMessage,ClientPrepareResponse>

// {
//      async fn prepare_sql(&self,s: SharedString) -> PrepareResult {
//         let r = self.send_message_and_wait_for_oneshot(ClientPrepareMessage::PrepareSql { sql: s.clone() }).await;

//         todo!()
        
//     }
// }

pub struct CanPrepare;
pub struct CantPrepare;

pub trait PreparedStatamentCapabilities {
    type Mode;
}




// impl<M,R,P> Prepare for Connected<M,R,P> where Connected<M,R,P>: SendMessageAndWaitForOneShot<M,R,P>,
// // M: IntoPrepareSqlMessage
//     M: From<SharedString> + Send,
//     PrepareResult: From<ResponsePacket<R,P>>,
//     P:Protocol + PreparedStatamentCapabilities<Mode=CanPrepare> + Send + Sync,
//     R: Send
// {
//         fn prepare(&self,s: SharedString) ->impl Future<Output= PrepareResult> + Send + use<'_,M,R,P> {
//         async move {
//             let r = self.send_message_and_wait_for_oneshot(M::from(s)).await;
//             PrepareResult::from(r)
//         }
        
//     }
// }

