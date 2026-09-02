// ---------------- [ File: osx-wallpaper-cycler/src/extract_dropbox_request_id_from_headers.rs ]
crate::ix!();

pub fn extract_dropbox_request_id_from_headers(
    headers: &reqwest::header::HeaderMap,
) -> Option<String> {
    headers
        .get("x-dropbox-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

pub async fn parse_json_body_or_none(resp: reqwest::Response) -> Option<serde_json::Value> {
    const MAX_CAPTURED_BYTES: usize = 32 * 1024;

    let content_type: Option<String> = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let bytes = resp.bytes().await.ok()?;
    let total_len = bytes.len();

    if total_len == 0 {
        return None;
    }

    let (slice, truncated): (&[u8], bool) = if total_len > MAX_CAPTURED_BYTES {
        (&bytes[..MAX_CAPTURED_BYTES], true)
    } else {
        (bytes.as_ref(), false)
    };

    if let Ok(v) = serde_json::from_slice::<serde_json::Value>(slice) {
        return Some(v);
    }

    let mut preview = String::from_utf8_lossy(slice).to_string();
    if truncated {
        preview.push_str("…<truncated>");
    }

    Some(serde_json::json!({
        "content_type": content_type,
        "raw_body_bytes": total_len,
        "raw_body_truncated": truncated,
        "raw_body_utf8_preview": preview,
    }))
}

#[cfg(test)]
mod dropbox_request_id_and_json_body_parsing_contract_suite {
    use super::*;

    #[traced_test]
    fn request_id_extraction_returns_some_when_present_and_valid_utf8() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "x-dropbox-request-id",
            reqwest::header::HeaderValue::from_static("RID123"),
        );

        let rid = extract_dropbox_request_id_from_headers(&headers);
        assert_eq!(rid.as_deref(), Some("RID123"));
    }

    #[traced_test]
    fn request_id_extraction_returns_none_when_missing_or_non_utf8() {
        let headers = reqwest::header::HeaderMap::new();
        assert_eq!(extract_dropbox_request_id_from_headers(&headers), None);

        let mut headers = reqwest::header::HeaderMap::new();
        let hv = reqwest::header::HeaderValue::from_bytes(b"\xff").unwrap();
        headers.insert("x-dropbox-request-id", hv);

        assert_eq!(extract_dropbox_request_id_from_headers(&headers), None);
    }

    #[traced_test]
    fn parse_json_body_or_none_returns_some_for_valid_json_and_none_for_invalid_json() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let server = httpmock::MockServer::start();

            server.mock(|when, then| {
                when.method(httpmock::Method::GET).path("/ok");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(serde_json::json!({"a":1,"b":"x"}));
            });

            server.mock(|when, then| {
                when.method(httpmock::Method::GET).path("/bad");
                then.status(200)
                    .header("content-type", "application/json")
                    .body("not-json");
            });

            let client = reqwest::Client::new();

            let resp_ok = client
                .get(format!("{}/ok", server.base_url()))
                .send()
                .await
                .unwrap();
            let v_ok = parse_json_body_or_none(resp_ok).await;
            assert_eq!(
                v_ok.as_ref().and_then(|v| v.get("a")).and_then(|x| x.as_i64()),
                Some(1)
            );

            let resp_bad = client
                .get(format!("{}/bad", server.base_url()))
                .send()
                .await
                .unwrap();
            let v_bad = parse_json_body_or_none(resp_bad).await;
            assert!(v_bad.is_some());

            let v_bad = v_bad.unwrap();
            assert_eq!(
                v_bad
                    .get("raw_body_utf8_preview")
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
                "not-json"
            );
            assert_eq!(
                v_bad.get("raw_body_bytes").and_then(|v| v.as_u64()),
                Some(8)
            );
            assert_eq!(
                v_bad.get("raw_body_truncated").and_then(|v| v.as_bool()),
                Some(false)
            );
            assert_eq!(
                v_bad
                    .get("content_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
                "application/json"
            );
        });
    }

    #[traced_test]
    fn parse_json_body_or_none_returns_some_for_valid_json_and_preview_for_invalid_json() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let server = httpmock::MockServer::start();

            server.mock(|when, then| {
                when.method(httpmock::Method::GET).path("/ok");
                then.status(200)
                    .header("content-type", "application/json")
                    .json_body(serde_json::json!({"a":1,"b":"x"}));
            });

            server.mock(|when, then| {
                when.method(httpmock::Method::GET).path("/bad");
                then.status(200)
                    .header("content-type", "application/json")
                    .body("not-json");
            });

            let client = reqwest::Client::new();

            let resp_ok = client
                .get(format!("{}/ok", server.base_url()))
                .send()
                .await
                .unwrap();
            let v_ok = parse_json_body_or_none(resp_ok).await;
            assert_eq!(
                v_ok.as_ref().and_then(|v| v.get("a")).and_then(|x| x.as_i64()),
                Some(1)
            );

            let resp_bad = client
                .get(format!("{}/bad", server.base_url()))
                .send()
                .await
                .unwrap();
            let v_bad = parse_json_body_or_none(resp_bad).await;
            assert!(v_bad.is_some());

            let v_bad = v_bad.unwrap();
            assert_eq!(
                v_bad
                    .get("raw_body_utf8_preview")
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
                "not-json"
            );
            assert_eq!(
                v_bad.get("raw_body_bytes").and_then(|v| v.as_u64()),
                Some(8)
            );
            assert_eq!(
                v_bad.get("raw_body_truncated").and_then(|v| v.as_bool()),
                Some(false)
            );
            assert_eq!(
                v_bad
                    .get("content_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
                "application/json"
            );
        });
    }
}
