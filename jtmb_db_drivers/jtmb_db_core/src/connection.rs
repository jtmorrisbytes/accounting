use std::{marker::PhantomData, sync::Arc};

use crate::{Protocol, SharedString, prepare::{CantPrepare, PreparedStatamentCapabilities}, query::Query};




pub trait ProcotolWorker {
    fn run_loop(&self) -> impl Future<Output=Result<(),()>> + Send + use<Self>;
}



pub struct Reactor<P: Protocol> {
    protocol: Arc<P>,
    connections: Vec<Connection<P::InnerConnection,P>>
}
impl<P: Protocol> Reactor<P> {
    pub fn proto(&self) -> &P {
        &*self.protocol
    }
}


// pub trait Spawn {
//     fn spawn(&self) -> tokio::task::JoinHandle<()>;
// }

impl <P> Reactor<P>
    where P: ProcotolWorker + Protocol + Send + Sync + 'static {
        async fn spawn(&self,options: &<P as Protocol>::ConnectOptions) -> tokio::task::JoinHandle<()> {
            let protocol = self.protocol.clone();
            let connection = protocol.connect(options).await.unwrap();
            tokio::spawn(async move {
                loop {
                    let r = protocol.run_loop().await;
                    match r {
                        Ok(_) => continue,
                        Err(_) => break
                    }
                }
                

            })
        }

    }


    /*
    
    
    so we dont need a query or decode or execute traits necessairly. just reactor.query -> query handle or whatever
Exactly. You have reached the "Systems Minimalism" endgame.
By consolidating into a reactor.query() call, those separate traits (Query, Decode, Execute) stop being Public Bureaucracy and start being Internal Mechanics. You aren't deleting the logic; you're just hiding the "Fluff" from the user.
The "No Fluff" Pipeline on your 5600X:
The Trigger: let result = reactor.query("SELECT * FROM users", params![id]).await;
The Movement: The Reactor (owning the 12 threads) picks a worker, grabs the !Sync SQLite handle, and binds the X Bytes of your params.
The Conversion: The Protocol (inside the worker) "Gulps" the result into an Arc<[u8]> and sends it back.
The Handle: You get a QueryHandle (or RowStream) that lets you pull data when you're ready.
    
    
    
    
    
     */


// pub trait Connect {
//     fn connect<T,O>(&self, address:&str) -> impl Future<Output= std::io::Result<self::Connection<T>>>+ use<Self,T>;
// }

// impl<P> Connect for P where P: Protocol {
//     fn connect<T>(&self,options: &P::ConnectOptions) -> impl Future<Output= std::io::Result<self::Connection<T>>>+ use<Self,T> {
//         let t: T = self.connect(options).await
//     }
// }


pub struct Connection<T,P: Protocol>
    where  P: Protocol<InnerConnection = T>
{
    inner:T,
    protocol: Arc<P>
}
impl<T,P> Drop for Connection<T,P>
    where P:Protocol<InnerConnection = T>
{
    fn drop(&mut self) {
        match self.protocol.close_connection::<T>(&self.inner) {
            Err(e) => self.protocol.on_close_fail(e),
            _=>{}
        }
        
    }
}
impl<T,ProtocolStruct> Connection<T,ProtocolStruct> 
    where ProtocolStruct:Protocol<InnerConnection = T>
{
    pub fn new(t:T,s:ProtocolStruct) -> Self {
        Self{inner:t,protocol:Arc::new(s)}
    }
}
// it is not safe to use connection between threads unless it is not a pointer
unsafe impl<T,ProtocolStruct> Send for Connection<T,ProtocolStruct>
    where ProtocolStruct:Protocol<InnerConnection = T>

{} 
#[derive(Debug)]
pub struct ResponsePacketData<M,P> where P: Protocol {
    data: Arc<u8>,protocol: Arc<P>,sender:tokio::sync::oneshot::Sender<M>
}

// if your type implements this trait, you support prepared statements
#[derive(Debug)]
pub enum ResponsePacket<M,P>
    where P: Protocol

{
    Ok(ResponsePacketData<M,P>),
    Err(()),
}
impl <P: Send+ Protocol,M> ResponsePacket<M,P> where Self:Send, M: Send {}

pub struct Connected<ProtocolStruct: Protocol> {
    // sender: tokio::sync::mpsc::Sender<(MessageType, tokio::sync::oneshot::Sender<Response>)>,
    protocol:Arc<ProtocolStruct>
}
impl<Protocol> Connected<ProtocolStruct> {
    pub fn query<S: AsRef<str>>(sql: S) -> Query<P,DB>
}
// impl<Message, Response,ProtocolStruct> Connected<Message, Response,ProtocolStruct>
//     where ProtocolStruct: Protocol
// {
//     pub fn new(
//         sender: tokio::sync::mpsc::Sender<(Message, tokio::sync::oneshot::Sender<Response>)>,
//     ) -> Self {
//         Self { sender, _protocol:PhantomData }
//     }
//     // pub async fn send_message(&self,message:MessageType) -> ConnectedMessageResponse<MessageType> {
//     //     let (thread_sender,thread_reciever) = tokio::sync::oneshot::channel();
//     //     self.sender.send((message,thread_sender)).await;
//     // }
// }





// impl<M,R,P> Query<P> for Connected<M,R,P> where Connected<M,R,P>: SendMessageAndWaitForOneShot<M,R,P>,
//     P: crate::Protocol + PreparedStatamentCapabilities<Mode=CantPrepare>


// {
//     fn query(&self) -> impl Future<Output=()> + use<M,R,P> {
//         let sender = self.sender.clone();
//         async{}
//     }
// }



pub trait SendMessageAndWaitForOneShot<Message, Response,P>
    where P:Protocol,
    
{
    // type Output;

    fn send_message_and_wait_for_oneshot(
        &self,
        message: Message,
    ) ->impl Future<Output=ResponsePacket<Response,P>> + Send + use<Self,Message,Response,P>;
}

impl<Message, Response,ProtocolStruct> SendMessageAndWaitForOneShot<Message, Response,ProtocolStruct>
    for Connected<Message, Response,ProtocolStruct>
    where ProtocolStruct:Protocol + Send,
    Message:Send,
    Response:Send,
    Response:Into<ResponsePacketData<Response,ProtocolStruct>>,
{
    // type Output = ConnectedMessageResponse<Response>;
    fn send_message_and_wait_for_oneshot(
        &self,
        message: Message,
    ) -> impl Future<Output=ResponsePacket<Response,ProtocolStruct>> + Send + use<Message,Response,ProtocolStruct> {
        let sender = self.sender.clone();
        async move {

            let (thread_sender, thread_reciever) = tokio::sync::oneshot::channel();
            let msg = sender.send((message, thread_sender)).await;
            match msg {
                Err(e) => return ResponsePacket::Err(()),
                Ok(_) => {}
            }
            let response = thread_reciever.await;
            match response {
                Ok(d) => ResponsePacket::Ok(d.into()),
                Err(e) =>{ResponsePacket::Err(())}
            }
        }
    }
}
