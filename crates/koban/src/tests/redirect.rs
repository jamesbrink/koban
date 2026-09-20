use super::*;

#[tokio::test]
async fn cross_host_redirect_is_not_followed_and_does_not_forward_token() {
    let origin = MockServer::start();
    let other = MockServer::start();

    let reached_other = other.mock(|when, then| {
        when.method(GET).path("/leak");
        then.status(200).body("{}");
    });
    let leaked_token = other.mock(|when, then| {
        when.method(GET).path("/leak").header_exists("X-API-TOKEN");
        then.status(200).body("{}");
    });

    let redirect = origin.mock(|when, then| {
        when.method(GET).path("/api/v1/invoices");
        then.status(302)
            .header("Location", format!("{}/leak", other.base_url()));
    });

    let client =
        ApiClient::new(Config::from_values(origin.base_url(), "secret-token").expect("config"));
    let result = client.get_json("api/v1/invoices", &[]).await;

    redirect.assert();
    leaked_token.assert_calls(0);
    reached_other.assert_calls(0);
    result.expect_err("cross-host redirect must fail, not silently succeed");
}

#[tokio::test]
async fn same_origin_redirect_is_followed() {
    let server = MockServer::start();

    let target = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/redirected")
            .header("X-API-TOKEN", "secret-token");
        then.status(200).body(r#"{"data":[]}"#);
    });
    let redirect = server.mock(|when, then| {
        when.method(GET).path("/api/v1/invoices");
        then.status(302).header(
            "Location",
            format!("{}/api/v1/redirected", server.base_url()),
        );
    });

    let client =
        ApiClient::new(Config::from_values(server.base_url(), "secret-token").expect("config"));
    let value = client
        .get_json("api/v1/invoices", &[])
        .await
        .expect("same-origin redirect should be followed");
    assert_eq!(value, serde_json::json!({"data":[]}));
    redirect.assert();
    target.assert();
}
