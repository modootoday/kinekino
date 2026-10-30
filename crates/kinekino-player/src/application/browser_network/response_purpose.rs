//! Admission of already observed response bodies; never initiates a request.
use super::{NetworkResponse, ResponsePurpose};

pub(super) fn classify(
    method: &str,
    resource_type: Option<&str>,
    url: &str,
    response: &NetworkResponse,
) -> Option<ResponsePurpose> {
    let observed = url::Url::parse(url).ok()?;
    let player =
        crate::domain::player_document::PlayerDocumentLocation::admitted_response_origin(&observed);
    let xhr = matches!(resource_type, Some("XHR" | "Fetch"));
    if method == "GET"
        && resource_type == Some("Script")
        && player
        && observed.path() == "/player/assets/scripts.php"
    {
        Some(ResponsePurpose::PlayerScript)
    } else if method == "POST" && xhr && player && observed.path() == "/player/index.php" {
        Some(ResponsePurpose::PlayerBootstrap)
    } else if method == "GET"
        && (200..300).contains(&response.status)
        && response.mime_type.to_ascii_lowercase().contains("mpegurl")
    {
        Some(ResponsePurpose::ManifestCandidate)
    } else if method == "GET" && xhr {
        Some(ResponsePurpose::MediaProbe)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn response() -> NetworkResponse {
        NetworkResponse {
            url: "https://media.example.test/player/index.php?fixture".into(),
            status: 200,
            mime_type: "text/html".into(),
            encoded_bytes: None,
            from_disk_cache: None,
            from_service_worker: None,
        }
    }
    #[test]
    fn bootstrap_requires_actual_method_kind_origin_and_path() {
        let response = response();
        assert!(matches!(
            classify("POST", Some("XHR"), &response.url, &response),
            Some(ResponsePurpose::PlayerBootstrap)
        ));
        assert!(!matches!(
            classify("GET", Some("XHR"), &response.url, &response),
            Some(ResponsePurpose::PlayerBootstrap)
        ));
        assert!(
            classify(
                "POST",
                Some("XHR"),
                "https://example.invalid/player/index.php",
                &response
            )
            .is_none()
        );
        assert!(classify("POST", Some("Script"), &response.url, &response).is_none());
    }
    #[test]
    fn observed_secondary_assets_keep_original_method_type_path_and_secure_origin_guards() {
        for host in ["media.example.test", "michealcdn.com"] {
            let script = format!("https://{host}/player/assets/scripts.php?v=7");
            let bootstrap = format!(
                "https://{host}/player/index.php?data=0123456789abcdef0123456789abcdef&do=getVideo"
            );
            let mut response = response();
            response.url = script.clone();
            assert!(matches!(
                classify("GET", Some("Script"), &script, &response),
                Some(ResponsePurpose::PlayerScript)
            ));
            response.url = bootstrap.clone();
            assert!(matches!(
                classify("POST", Some("XHR"), &bootstrap, &response),
                Some(ResponsePurpose::PlayerBootstrap)
            ));
            for (method, kind, raw) in [
                ("POST", Some("Script"), script.as_str()),
                ("GET", Some("XHR"), bootstrap.as_str()),
                ("POST", Some("Script"), bootstrap.as_str()),
                ("POST", Some("Image"), bootstrap.as_str()),
            ] {
                assert!(!matches!(
                    classify(method, kind, raw, &response),
                    Some(ResponsePurpose::PlayerScript | ResponsePurpose::PlayerBootstrap)
                ));
            }
        }
        let response = response();
        for raw in [
            "http://michealcdn.com/player/index.php",
            "https://michealcdn.com:444/player/index.php",
            "https://user@michealcdn.com/player/index.php",
            "https://michealcdn.com.evil.test/player/index.php",
            "https://sub.michealcdn.com/player/index.php",
            "https://michealcdn.com/other/index.php",
            "https://michealcdn.com/player/index.php/other",
        ] {
            assert!(
                classify("POST", Some("XHR"), raw, &response).is_none(),
                "Unexpected bootstrap admission: {raw}"
            );
        }
    }
    #[test]
    fn observed_xhr_can_be_probed_without_promoting_extension_or_mime_to_media() {
        let response = response();
        assert!(matches!(
            classify("GET", Some("XHR"), "https://cdn.example/a.txt", &response),
            Some(ResponsePurpose::MediaProbe)
        ));
        assert!(classify("GET", Some("Image"), "https://cdn.example/a.txt", &response).is_none());
    }
}
