use crate::{ConnectionHandle, Protocol, SharedString, connection::Connected};


pub struct CanExecuteUnprepared;

pub trait ExcutionCapabilities{
    type Mode;
}


pub trait Execute{}

pub trait ExecuteUnprepared {
    fn execute_unprepared(sql: SharedString);
}
