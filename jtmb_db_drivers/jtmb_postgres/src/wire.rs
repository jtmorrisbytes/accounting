use tokio::io::BufReader;
use tokio::net::TcpStream;
use tokio::io::{AsyncBufRead,AsyncBufReadExt};


type OID = u32;
const OID_PG_BOOL: OID = 16_u32;

pub trait HasType<Rust,Postgres> {
    fn oid() -> OID;
}

struct PGBool(bool,OID);


trait IntoOid{}



type Int32 = i32;



pub struct Connection {

}
impl Connection {
    pub fn connect() -> Self {




        Self {}
    }
}

impl std::future::Future for Connection {
    type Output = ();
    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        std::task::Poll::Ready(())
    }
}



#[tokio::test]

pub async fn test_wire() -> Result<(),Box<dyn std::error::Error>> {
    let stream = TcpStream::connect("127.0.0.1:5432").await?;
    let (read,write) = stream.into_split();
    
}