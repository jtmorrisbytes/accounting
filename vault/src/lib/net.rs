// use std::{io::{Read, Write}, net::TcpListener};

// use hyper::{Method, Request, Response, body::Incoming, header::HeaderName, server::conn::http1};
// use hyper_util::rt::TokioIo;

// /// spins up a small server on the main thread to listen for the bips 39
// /// DO NOT USE NOT FINISHED
// #[deprecated]
// pub fn listen_on_net_for_bips39()-> Result<(), Box<dyn std::error::Error>> {

//     let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
//     rt.block_on(async {
//         // let addr = std::net::SocketAddr::from((["0","0","0","0"],8080));
//         let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
//         let local_address = listener.local_addr()?;
//         println!("Program listening at http://{}:{}/",local_address.ip(),local_address.port());
//         loop {
//             // check if we recieved the secret here later
//             let  (mut stream,_socketaddress) = listener.accept().await?;
//             // the actual 'server' function
//             let io = TokioIo::new(stream);
//             tokio::task::spawn(async move {
//                 let service = hyper::service::service_fn(move | req: Request<Incoming>|{
//                     async move {
//                         let response =
//                         format!("<!doctype html>\
//                                 <html>\
//                                 <head>\
//                                 <script src=\"https://raw.githubusercontent.com/mebjas/html5-qrcode/master/minified/html5-qrcode.min.js\"></script>\
//                                 <title>Scan Code</title>
//                                 </head>
//                                 <body>
//                                     <p> Scan your bips39 secret to continue </p>
//                                     <div id=\"reader\" style=\"width=100%; max-width:600px\"></div>
//                                     <script>
//                                         try {{
//                                         const scanner = new Html5QrcodeScanner(\"reader\",{{fps:10,qrbox:250}});
//                                                 scanner.render((decodedText) => {{
//                                                     // When a QR code is detected, send it to the Rust server
//                                                     fetch('/submit', {{
//                                                         method: 'POST',
//                                                         body: decodedText
//                                                     }}).then(() => {{
//                                                         alert(\"BIP-39 Received! You can close this tab.\");
//                                                         scanner.clear();
//                                                     }});
//                                                 }});
//                                         }}
//                                         catch(e) {{
//                                           alert(e.toString());
//                                         }}
//                                     </script>
//                                 </body>
//                                 ");

//                         match (req.method(), req.uri())  {
//                             (&Method::GET,uri) if uri == "/" => {
//                                 Response::builder()
//                                 .header("Content-Type", "text/html")
//                                 .header("Content-Length", response.len().to_string())
//                                 .body(response)
//                             }

//                             _=> {
//                                 let response = "Not Found or bad request".to_string();
//                                 Response::builder()
//                                 .header("Content-Length", response.len().to_string())
//                                 .status(404).body(response)
//                             }
//                         }
//                         // Result::<_,String>::Ok()
//                         // Ok(())
//                     }
//                 });

//                 http1::Builder::new().serve_connection(io,service).await
//             });
//             // break;
//         }

//         Result::<_,Box<dyn std::error::Error>>::Ok(())
//     })?;

//     Ok(())
// }

// #[cfg(test)]
// // #[test]
// pub fn test_net_listener() -> Result<(), Box<dyn std::error::Error>> {
//     listen_on_net_for_bips39()?;
//     Ok(())
// }
