use crate::exports::edgee::protocols::data_collection::{Dict, EdgeeRequest, Event, HttpMethod};
use exports::edgee::protocols::data_collection::Guest;
mod s3_payload;

wit_bindgen::generate!({
    world: "data-collection",
    path: "wit",
    additional_derives: [serde::Serialize],
    generate_all,
});
export!(Component);

struct Component;

impl Guest for Component {

    fn page(_edgee_event: Event, _cred_map: Dict) -> Result<EdgeeRequest, String> {
        send_to_s3(_edgee_event, _cred_map)
    }

    fn track(_edgee_event: Event, _cred_map: Dict) -> Result<EdgeeRequest, String> {
        send_to_s3(_edgee_event, _cred_map)
    }

    fn user(_edgee_event: Event, _cred_map: Dict) -> Result<EdgeeRequest, String> {
        send_to_s3(_edgee_event, _cred_map)
    }
}

fn send_to_s3(edgee_event: Event, creds: Dict) -> Result<EdgeeRequest, String> {
    let s3_config = s3_payload::S3Config::new(creds).map_err(|e| e.to_string())?;

    // serialize the entire event into JSON
    let file_content = serde_json::to_string(&edgee_event).unwrap_or_else(| _ | "".to_string());
    
    // generate full URL and HTTP headers
    let s3_url = s3_config.generate_s3_url(); // S3 key is auto-generated (.json)
    let sigv4_headers = s3_config.generate_s3_headers(s3_url.clone(), file_content.clone());

    Ok(EdgeeRequest {
        method: HttpMethod::Put,
        url: s3_url,
        headers: sigv4_headers,
        body: file_content,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::exports::edgee::protocols::data_collection::{
        Campaign, Client, Context, Data, EventType, PageData, Session, UserData,
    };
    use exports::edgee::protocols::data_collection::Consent;
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
                user_agent_architecture: "fuck knows".to_string(),
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

    #[test]
    fn page_works_fine() {
        let event = sample_page_event(
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let credentials = vec![("your-credentials".to_string(), "abc".to_string())];
        let result = Component::page(event, credentials);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert_eq!(edgee_request.body.is_empty(), true);
        assert_eq!(edgee_request.url.starts_with("https://example.com/"), true);
        // add more checks (headers, querystring, etc.)
    }
}
