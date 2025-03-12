use crate::exports::edgee::components::data_collection::{Dict, EdgeeRequest, Event, HttpMethod};
use exports::edgee::components::data_collection::Guest;
mod s3_payload;

wit_bindgen::generate!({
    world: "data-collection",
    path: ".edgee/wit",
    additional_derives: [serde::Serialize],
    generate_all,
});
export!(Component);

struct Component;

impl Guest for Component {
    fn page(edgee_event: Event, settings: Dict) -> Result<EdgeeRequest, String> {
        send_to_s3(edgee_event, settings)
    }

    fn track(edgee_event: Event, settings: Dict) -> Result<EdgeeRequest, String> {
        send_to_s3(edgee_event, settings)
    }

    fn user(edgee_event: Event, settings: Dict) -> Result<EdgeeRequest, String> {
        send_to_s3(edgee_event, settings)
    }
}

fn send_to_s3(edgee_event: Event, settings_dict: Dict) -> Result<EdgeeRequest, String> {
    let s3_settings = s3_payload::Settings::new(settings_dict).map_err(|e| e.to_string())?;

    // serialize the entire event into JSON
    let file_content = serde_json::to_string(&edgee_event).unwrap_or_default();

    // generate full URL and HTTP headers
    let s3_url = s3_settings.generate_s3_url(); // S3 key is auto-generated (.json)
    let sigv4_headers = s3_settings.generate_s3_headers(s3_url.clone(), file_content.clone());

    Ok(EdgeeRequest {
        method: HttpMethod::Put,
        url: s3_url,
        headers: sigv4_headers,
        body: file_content,
        forward_client_headers: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::exports::edgee::components::data_collection::{
        Campaign, Client, Context, Data, EventType, PageData, Session, TrackData, UserData,
    };
    use exports::edgee::components::data_collection::Consent;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    fn sample_user_data(edgee_id: String) -> UserData {
        UserData {
            user_id: "123".to_string(),
            anonymous_id: "456".to_string(),
            edgee_id,
            properties: vec![
                ("prop1".to_string(), "value1".to_string()),
                ("prop2".to_string(), "10".to_string()),
            ],
        }
    }

    fn sample_context(edgee_id: String, locale: String, session_start: bool) -> Context {
        Context {
            page: sample_page_data(),
            user: sample_user_data(edgee_id),
            client: Client {
                city: "Paris".to_string(),
                ip: "192.168.0.1".to_string(),
                locale,
                timezone: "CET".to_string(),
                user_agent: "Chrome".to_string(),
                user_agent_architecture: "x86".to_string(),
                user_agent_bitness: "64".to_string(),
                user_agent_full_version_list: "abc".to_string(),
                user_agent_version_list: "abc".to_string(),
                user_agent_mobile: "mobile".to_string(),
                user_agent_model: "don't know".to_string(),
                os_name: "MacOS".to_string(),
                os_version: "latest".to_string(),
                screen_width: 1024,
                screen_height: 768,
                screen_density: 2.0,
                continent: "Europe".to_string(),
                country_code: "FR".to_string(),
                country_name: "France".to_string(),
                region: "West Europe".to_string(),
            },
            campaign: Campaign {
                name: "random".to_string(),
                source: "random".to_string(),
                medium: "random".to_string(),
                term: "random".to_string(),
                content: "random".to_string(),
                creative_format: "random".to_string(),
                marketing_tactic: "random".to_string(),
            },
            session: Session {
                session_id: "random".to_string(),
                previous_session_id: "random".to_string(),
                session_count: 2,
                session_start,
                first_seen: 123,
                last_seen: 123,
            },
        }
    }

    fn sample_page_data() -> PageData {
        PageData {
            name: "page name".to_string(),
            category: "category".to_string(),
            keywords: vec!["value1".to_string(), "value2".into()],
            title: "page title".to_string(),
            url: "https://example.com/full-url?test=1".to_string(),
            path: "/full-path".to_string(),
            search: "?test=1".to_string(),
            referrer: "https://example.com/another-page".to_string(),
            properties: vec![
                ("prop1".to_string(), "value1".to_string()),
                ("prop2".to_string(), "10".to_string()),
                ("currency".to_string(), "USD".to_string()),
            ],
        }
    }

    fn sample_track_data(event_name: String) -> TrackData {
        return TrackData {
            name: event_name,
            products: vec![],
            properties: vec![
                ("prop1".to_string(), "value1".to_string()),
                ("prop2".to_string(), "10".to_string()),
                ("currency".to_string(), "USD".to_string()),
            ],
        };
    }

    fn sample_page_event(
        consent: Option<Consent>,
        edgee_id: String,
        locale: String,
        session_start: bool,
    ) -> Event {
        Event {
            uuid: Uuid::new_v4().to_string(),
            timestamp: 123,
            timestamp_millis: 123,
            timestamp_micros: 123,
            event_type: EventType::Page,
            data: Data::Page(sample_page_data()),
            context: sample_context(edgee_id, locale, session_start),
            consent,
        }
    }

    fn sample_track_event(
        event_name: String,
        consent: Option<Consent>,
        edgee_id: String,
        locale: String,
        session_start: bool,
    ) -> Event {
        return Event {
            uuid: Uuid::new_v4().to_string(),
            timestamp: 123,
            timestamp_millis: 123,
            timestamp_micros: 123,
            event_type: EventType::Track,
            data: Data::Track(sample_track_data(event_name)),
            context: sample_context(edgee_id, locale, session_start),
            consent: consent,
        };
    }

    fn sample_user_event(
        consent: Option<Consent>,
        edgee_id: String,
        locale: String,
        session_start: bool,
    ) -> Event {
        return Event {
            uuid: Uuid::new_v4().to_string(),
            timestamp: 123,
            timestamp_millis: 123,
            timestamp_micros: 123,
            event_type: EventType::User,
            data: Data::User(sample_user_data(edgee_id.clone())),
            context: sample_context(edgee_id, locale, session_start),
            consent: consent,
        };
    }

    #[test]
    fn page_works_fine() {
        let event = sample_page_event(
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );

        let settings = vec![
            ("aws_access_key".to_string(), "TEST".to_string()),
            ("aws_secret_key".to_string(), "TEST".to_string()),
            ("aws_region".to_string(), "eu-west-1".to_string()),
            ("s3_bucket".to_string(), "test-bucket".to_string()),
        ];
        let result = Component::page(event, settings);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Put);
        assert_eq!(edgee_request.body.is_empty(), false);
        assert_eq!(
            edgee_request
                .url
                .starts_with("https://test-bucket.s3.amazonaws.com/"),
            true
        );

        assert_eq!(edgee_request.headers.len(), 3);
        assert_eq!(
            edgee_request
                .headers
                .iter()
                .any(|(key, _value)| key == "authorization"),
            true
        );
        assert_eq!(
            edgee_request
                .headers
                .iter()
                .any(|(key, _value)| key == "x-amz-content-sha256"),
            true
        );
        assert_eq!(
            edgee_request
                .headers
                .iter()
                .any(|(key, _value)| key == "x-amz-date"),
            true
        );
        // add more checks (headers, querystring, etc.)
    }

    #[test]
    fn track_works_fine() {
        let event = sample_track_event(
            "custom-event".to_string(),
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );

        let settings = vec![
            ("aws_access_key".to_string(), "TEST".to_string()),
            ("aws_secret_key".to_string(), "TEST".to_string()),
            ("aws_region".to_string(), "eu-west-1".to_string()),
            ("s3_bucket".to_string(), "test-bucket".to_string()),
        ];
        let result = Component::track(event, settings);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Put);
        assert_eq!(edgee_request.body.is_empty(), false);
        assert_eq!(
            edgee_request
                .url
                .starts_with("https://test-bucket.s3.amazonaws.com/"),
            true
        );

        assert_eq!(edgee_request.headers.len(), 3);
        assert_eq!(
            edgee_request
                .headers
                .iter()
                .any(|(key, _value)| key == "authorization"),
            true
        );
        assert_eq!(
            edgee_request
                .headers
                .iter()
                .any(|(key, _value)| key == "x-amz-content-sha256"),
            true
        );
        assert_eq!(
            edgee_request
                .headers
                .iter()
                .any(|(key, _value)| key == "x-amz-date"),
            true
        );
        // add more checks (headers, querystring, etc.)
    }

    #[test]
    fn user_works_fine() {
        let event = sample_user_event(
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );

        let settings = vec![
            ("aws_access_key".to_string(), "TEST".to_string()),
            ("aws_secret_key".to_string(), "TEST".to_string()),
            ("aws_region".to_string(), "eu-west-1".to_string()),
            ("s3_bucket".to_string(), "test-bucket".to_string()),
        ];
        let result = Component::user(event, settings);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Put);
        assert_eq!(edgee_request.body.is_empty(), false);
        assert_eq!(
            edgee_request
                .url
                .starts_with("https://test-bucket.s3.amazonaws.com/"),
            true
        );

        assert_eq!(edgee_request.headers.len(), 3);
        assert_eq!(
            edgee_request
                .headers
                .iter()
                .any(|(key, _value)| key == "authorization"),
            true
        );
        assert_eq!(
            edgee_request
                .headers
                .iter()
                .any(|(key, _value)| key == "x-amz-content-sha256"),
            true
        );
        assert_eq!(
            edgee_request
                .headers
                .iter()
                .any(|(key, _value)| key == "x-amz-date"),
            true
        );
        // add more checks (headers, querystring, etc.)
    }

    #[test]
    fn page_with_session_token() {
        let event = sample_page_event(
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );

        let settings = vec![
            ("aws_access_key".to_string(), "TEST".to_string()),
            ("aws_secret_key".to_string(), "TEST".to_string()),
            ("aws_session_token".to_string(), "TEST".to_string()),
            ("aws_region".to_string(), "eu-west-1".to_string()),
            ("s3_bucket".to_string(), "test-bucket".to_string()),
        ];
        let result = Component::page(event, settings);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Put);
        assert_eq!(edgee_request.body.is_empty(), false);
        assert_eq!(
            edgee_request
                .url
                .starts_with("https://test-bucket.s3.amazonaws.com/"),
            true
        );

        assert_eq!(edgee_request.headers.len(), 4);
        assert_eq!(
            edgee_request
                .headers
                .iter()
                .any(|(key, _value)| key == "authorization"),
            true
        );
        assert_eq!(
            edgee_request
                .headers
                .iter()
                .any(|(key, _value)| key == "x-amz-content-sha256"),
            true
        );
        assert_eq!(
            edgee_request
                .headers
                .iter()
                .any(|(key, _value)| key == "x-amz-date"),
            true
        );
        assert_eq!(
            edgee_request
                .headers
                .iter()
                .any(|(key, _value)| key == "x-amz-security-token"),
            true
        );
        // add more checks (headers, querystring, etc.)
    }

    #[test]
    fn page_with_s3_key_prefix() {
        let event = sample_page_event(
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );

        let settings = vec![
            ("aws_access_key".to_string(), "TEST".to_string()),
            ("aws_secret_key".to_string(), "TEST".to_string()),
            ("aws_region".to_string(), "eu-west-1".to_string()),
            ("s3_bucket".to_string(), "test-bucket".to_string()),
            ("s3_key_prefix".to_string(), "sub-folder/".to_string()),
        ];
        let result = Component::page(event, settings);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Put);
        assert_eq!(edgee_request.body.is_empty(), false);
        assert_eq!(
            edgee_request
                .url
                .starts_with("https://test-bucket.s3.amazonaws.com/"),
            true
        );
        assert_eq!(edgee_request.url.contains("sub-folder/"), true);
    }

    #[test]
    fn breaks_without_settings() {
        let event = sample_page_event(
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let mut settings = vec![
            //("aws_access_key".to_string(), "TEST".to_string()),
            ("aws_secret_key".to_string(), "TEST".to_string()),
            ("aws_region".to_string(), "eu-west-1".to_string()),
            ("s3_bucket".to_string(), "test-bucket".to_string()),
        ];
        let result = Component::page(event.clone(), settings);
        assert_eq!(result.is_err(), true);
        assert_eq!(
            result
                .clone()
                .err()
                .unwrap()
                .to_string()
                .contains("Missing AWS Access Key"),
            true
        );

        // test without secret key
        settings = vec![
            ("aws_access_key".to_string(), "TEST".to_string()),
            //("aws_secret_key".to_string(), "TEST".to_string()),
            ("aws_region".to_string(), "eu-west-1".to_string()),
            ("s3_bucket".to_string(), "test-bucket".to_string()),
        ];
        let result = Component::page(event.clone(), settings);
        assert_eq!(result.is_err(), true);
        assert_eq!(
            result
                .clone()
                .err()
                .unwrap()
                .to_string()
                .contains("Missing AWS Secret Key"),
            true
        );

        // test without region
        settings = vec![
            ("aws_access_key".to_string(), "TEST".to_string()),
            ("aws_secret_key".to_string(), "TEST".to_string()),
            //("aws_region".to_string(), "eu-west-1".to_string()),
            ("s3_bucket".to_string(), "test-bucket".to_string()),
        ];
        let result = Component::page(event.clone(), settings);
        assert_eq!(result.is_err(), true);
        assert_eq!(
            result
                .clone()
                .err()
                .unwrap()
                .to_string()
                .contains("Missing AWS region"),
            true
        );

        // test without bucket
        settings = vec![
            ("aws_access_key".to_string(), "TEST".to_string()),
            ("aws_secret_key".to_string(), "TEST".to_string()),
            ("aws_region".to_string(), "eu-west-1".to_string()),
            //("s3_bucket".to_string(), "test-bucket".to_string()),
        ];
        let result = Component::page(event, settings);
        assert_eq!(result.is_err(), true);
        assert_eq!(
            result
                .clone()
                .err()
                .unwrap()
                .to_string()
                .contains("Missing S3 bucket"),
            true
        );
    }
}
