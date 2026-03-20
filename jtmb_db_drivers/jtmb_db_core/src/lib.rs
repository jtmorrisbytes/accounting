#![feature(random)]
#![feature(oneshot_channel)]
pub mod drivers;

// allows the driver to initialize itself before we hand out connections
// essentailly the entry point into 
pub trait DriverInitialize {
    type InitOptions;
    type ConnectionOptions;
    // type Error;
    fn driver_initialize(driver_options: Self::InitOptions,connect_options: Self::ConnectionOptions) -> std::io::Result<Self>
    where Self:Sized;
}

pub trait Connection {
    type Rawhandle;
    type ConnectionOptions;

    fn connect(&mut self,options:Self::ConnectionOptions) -> std::io::Result<()>;
}