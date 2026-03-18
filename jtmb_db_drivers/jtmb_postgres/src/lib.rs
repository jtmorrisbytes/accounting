pub mod wire;
use std::{ffi::CString, ops::Deref, sync::Arc};

use pq_sys::*;
use crossbeam::queue::ArrayQueue;


#[derive(PartialEq,Debug)]
#[repr(transparent)]
/// does not clone the underlying data, only its reference
pub struct Handle<T: Sized>(Arc<T>);
impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T> Deref for Handle<T> where T: Deref {
        type Target = T::Target; // Tunnel to the inner-inner type

    #[inline(always)] // "Blazing fast" hint for the compiler
    fn deref(&self) -> &Self::Target {
        // self (Handle) -> self.0 (T) -> .deref() (Target)
        self.0.deref()
    }
}


#[derive(Debug)]
#[repr(transparent)]
pub struct ConnectionHandle(Handle<__Connection>);
impl PartialEq for ConnectionHandle {
    fn eq(&self, other: &Self) -> bool {
        self.0.0.handle== other.0.0.handle
    }

}
// impl Deref for ConnectionHandle {
//     type Target = Handle<__Connection>;
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }


impl ConnectionHandle {
        fn new(handle: *mut PGconn) -> Self {

    ConnectionHandle(Handle(Arc::new(__Connection::new((handle)))))
    }
}

// DO NOT USE OUTSIDE THIS CRATE
// unafely implements SEND + SYNC. threaing only guarenteed
// to work inside crate
#[derive(Debug)]
pub(crate) struct __Connection{
    handle:*mut PGconn,
    statements: dashmap::DashMap<String,()>,
    socket: i32
}
unsafe impl Send for __Connection{}
unsafe impl Sync for __Connection{}

impl Drop for __Connection{
    fn drop(&mut self) {
        debug_assert!(self.handle.is_null() == false);
        unsafe {
            pq_sys::PQfinish(self.handle);
        }
    }
}

impl __Connection {
    fn new(handle: *mut PGconn) -> Self {
            let socket_id = unsafe { pq_sys::PQsocket(handle.cast())};

        __Connection{handle,statements: dashmap::DashMap::new(),socket:socket_id}
    }

}


/// a structure that represents a prepare statement task. async and non blocking
pub struct PrepareStatement {
    handle: ConnectionHandle,
    name:String,
    sql:String,
    state:usize,
    #[cfg(windows)]
    io_socket_stream: std::mem::ManuallyDrop<tokio::net::TcpStream>,
    #[cfg(not(windows))]
    io_socket_stream: tokio::io::unix::AsyncFd
}
impl PrepareStatement {
    pub fn prepare(
        handle:ConnectionHandle,
        name:String,
        sql:String,
    ) -> Self {
        let socket_id = handle.0.0.socket;
        #[cfg(windows)]
        {
            use std::{mem::ManuallyDrop, os::windows::io::{FromRawSocket,RawSocket}};
            use tokio::net::TcpStream;
            // 1. Get the raw socket from libpq
            let raw_socket: RawSocket = socket_id as RawSocket;
            let std_stream = unsafe { std::net::TcpStream::from_raw_socket(raw_socket) };
            std_stream.set_nonblocking(true).unwrap(); 
                   return  Self {
            handle,
            name,
            sql,
            state:0,
            io_socket_stream: ManuallyDrop::new(TcpStream::from_std(std_stream).unwrap()) 
        }

        }
        #[cfg(not(target_os="windows"))] {
            Self {
                handle,
                name,
                sql,
                state:0,
                io_socket_stream: AsyncFd::new(raw_conn).unwrap()
            }
        }
    }

}

pub enum PrepareStatementOutput {
    Ok,
    Err(String)
}


impl Future for PrepareStatement {
    type Output = PrepareStatementOutput;
    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        debug_assert!(self.handle.0.0.handle.is_null() == false);
        match self.state {
            0 => {
                unsafe {
                    let name_c = std::ffi::CString::new(self.name.clone());
                    let name_c = match name_c {
                        Ok(c)=>c,
                        Err(e) => return std::task::Poll::Ready(PrepareStatementOutput::Err(e.to_string()))
                    };
                    let sql_c = std::ffi::CString::new(self.name.clone());
                    let sql_c = match sql_c {
                        Ok(c)=>c,
                        Err(e) => return std::task::Poll::Ready(PrepareStatementOutput::Err(e.to_string()))
                    };
                    pq_sys::PQsendPrepare(self.handle.0.0.handle.cast(),name_c.as_ptr(),sql_c.as_ptr(),0,core::ptr::null());
                    self.as_mut().state+=1;

                }
            }
            1 => {
                let _ = std::task::ready!(self.io_socket_stream.poll_read_ready(cx)).unwrap();
            },
            _=>{

            }
        }       
        todo!()
    }
}






pub enum IntializeOutput {
    Keep(Handle<Postgres>),
    Query(Query),
    Error(std::io::Error),
}

impl Future for Handle<Postgres> {
    type Output = IntializeOutput;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        std::task::Poll::Ready(IntializeOutput::Keep(self.to_owned()))
    }
}







impl Initialize for Postgres {
    type Options = ();
    fn initialize(options: ()) -> crate::Handle<Self> {
        let conn_str = CString::new("host=127.0.0.1 port=5432 dbname=postgres user=postgres password=postgres sslmode=disable").unwrap();
        // perform the connection pooling step.
        let mut pool = ArrayQueue::new(10);
        for _ in 0..9 {
            unsafe {
                let conn = PQconnectdb(conn_str.as_ptr());
                if PQstatus(conn) != ConnStatusType::CONNECTION_OK {
                    let err = std::ffi::CStr::from_ptr(PQerrorMessage(conn));
                    panic!("Connection failed: {:?}", err);
                }
                pool.push(ConnectionHandle::new(conn)).unwrap();
            }
        }
        let s = Self {
            connection_pool: pool,
        };

        Handle(Arc::new(s))
    }
}

#[derive(PartialEq)]
pub enum Step<Keep, Next>
where
    Keep: PartialEq,
    Next: PartialEq,
{
    Keep(Keep),
    Next(Next),
    Done,
}

pub trait Initialize
where
    Self: Sized,
{
    // type Error: std::error::Error + Send + 'static;
    type Options: Send + 'static;
    /// configures the library to get ready to perform its startup tasks
    fn initialize(options: ()) -> crate::Handle<Self>;
}

#[derive(Debug)]
pub struct Postgres {
    connection_pool: ArrayQueue<crate::ConnectionHandle>,
}
impl Postgres
where
    Self: Send + Sync + Sized + 'static,
{
    pub fn query(&self) -> Query {
        Query {}
    }
    pub fn query_as() {}
}

#[derive(PartialEq)]
pub struct Query {}
impl Query {
    fn with_a(self) -> Self {
        self
    }
    fn with_b(self) -> Self {
        self
    }
}
impl Future for Query {
    type Output = ();
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        std::task::Poll::Ready(())
    }
}
