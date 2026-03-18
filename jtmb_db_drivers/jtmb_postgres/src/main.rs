#![feature(never_type)]
use crossbeam::queue::ArrayQueue;

use jtmb_postgres::{Handle, Initialize, IntializeOutput, Postgres};
use pq_sys::*;
use std::ffi::CString;
use std::future::{Future, pending};
use std::sync::Arc;
use std::{future, ptr};



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut task = Postgres::initialize(()).await;
    match task {
        IntializeOutput::Keep(mut task) => task = task,
        IntializeOutput::Query(query) => {
            println!("a query object is available")
        }
        IntializeOutput::Error(e) => {
            panic!("initialization failed")
        }
    }

    // let o: Result<!, std::io::Error> = Postgres::enter(()).next().await;

    // 1. Define the connection string (localhost, default port)
    let conn_str = CString::new(
        "host=127.0.0.1 port=5432 dbname=postgres user=postgres password=postgres sslmode=disable",
    )
    .unwrap();

    unsafe {
        // 2. Establish the connection (Synchronous)
        let conn = PQconnectdb(conn_str.as_ptr());

        // Check if the connection was successful
        if PQstatus(conn) != ConnStatusType::CONNECTION_OK {
            let err = std::ffi::CStr::from_ptr(PQerrorMessage(conn));
            panic!("Connection failed: {:?}", err);
        }
        println!("Connected to PostgreSQL!");

        // 3. Execute 'Hello World' query
        let query = CString::new("SELECT 'Hello World'").unwrap();
        let res = PQexec(conn, query.as_ptr());

        // Check the result status
        if PQresultStatus(res) != ExecStatusType::PGRES_TUPLES_OK {
            let err = std::ffi::CStr::from_ptr(PQresultErrorMessage(res));
            PQclear(res);
            PQfinish(conn);
            panic!("Query failed: {:?}", err);
        }

        // 4. Retrieve and print the data
        // Column 0, Row 0
        let val_ptr = PQgetvalue(res, 0, 0);
        let val = std::ffi::CStr::from_ptr(val_ptr);
        println!("Database says: {:?}", val);

        // 5. Cleanup (Crucial for avoiding leaks)
        PQclear(res);
        PQfinish(conn);
    }
    println!("Connection closed.");
    Ok(())
}
