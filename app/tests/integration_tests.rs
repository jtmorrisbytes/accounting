#[test]
fn test_auth_flow() -> Result<(),Box<dyn std::error::Error>> {
    let rocket = accounting::server::rocket();
    let test_client = rocket::local::blocking::Client::tracked(rocket)?;

    // private app. no access on any path except auth flow

    let response = test_client.get("/").dispatch();
    if !response.status().class().is_redirection() {
        return Err("Client is not authenticated. Client should perform redirect to auth flow".into());
    }
    Ok(())
}