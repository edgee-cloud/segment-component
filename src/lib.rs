mod segment_payload;

use base64::{
    alphabet::STANDARD,
    engine::{general_purpose::PAD, GeneralPurpose},
    Engine,
};
use exports::edgee::protocols::provider::{Data, Dict, EdgeeRequest, Event, Guest, HttpMethod};
use segment_payload::SegmentPayload;
use std::collections::HashMap;

wit_bindgen::generate!({world: "data-collection", path: "wit", with: { "edgee:protocols/provider": generate }});
export!(SegmentComponent);

struct SegmentComponent;

impl Guest for SegmentComponent {
    fn page(edgee_event: Event, cred_map: Dict) -> Result<EdgeeRequest, String> {
        // create a new segment payload
        let mut segment_payload = SegmentPayload::new(&edgee_event, &cred_map, "page".to_string())
            .map_err(|e| e.to_string())?;

        if let Data::Page(ref data) = edgee_event.data {
            // page event properties
            let mut properties = HashMap::new();

            properties.insert("title".to_string(), data.title.clone().into());
            segment_payload.context.page.as_mut().unwrap().title = Some(data.title.clone());

            properties.insert("url".to_string(), data.url.clone().into());
            segment_payload.context.page.as_mut().unwrap().url = Some(data.url.clone());

            properties.insert("path".to_string(), data.path.clone().into());
            segment_payload.context.page.as_mut().unwrap().path = Some(data.path.clone());

            if !data.referrer.is_empty() {
                properties.insert("referrer".to_string(), data.referrer.clone().into());
                segment_payload.context.page.as_mut().unwrap().referrer =
                    Some(data.referrer.clone());
            }
            if !data.search.is_empty() {
                properties.insert("search".to_string(), data.search.clone().into());
                segment_payload.context.page.as_mut().unwrap().search = Some(data.search.clone());
            }
            if !data.keywords.is_empty() {
                let keywords_json = serde_json::to_value(data.keywords.clone());
                if let Ok(keywords_value) = keywords_json {
                    properties.insert("keywords".to_string(), keywords_value);
                }
            }

            // iterate over page.properties and add them to properties
            for (key, value) in data.properties.clone().iter() {
                properties.insert(key.clone(), parse_value(value));
            }

            segment_payload.properties = Some(properties);

            Ok(build_edgee_request(segment_payload, &cred_map))
        } else {
            Err("Missing page data".to_string())
        }
    }

    fn track(edgee_event: Event, cred_map: Dict) -> Result<EdgeeRequest, String> {
        if let Data::Track(ref data) = edgee_event.data {
            // check if edgee_payload.track is empty
            if data.name.is_empty() {
                return Err("Track is not set".to_string());
            }

            // create a new segment payload
            let mut segment_payload =
                SegmentPayload::new(&edgee_event, &cred_map, "track".to_string())
                    .map_err(|e| e.to_string())?;

            // event name and properties
            segment_payload.event = Option::from(data.name.clone());

            let mut properties = HashMap::new();

            // iterate over page.properties and add them to properties
            for (key, value) in data.properties.clone().iter() {
                properties.insert(key.clone(), parse_value(value));
            }
            segment_payload.properties = Some(properties);

            Ok(build_edgee_request(segment_payload, &cred_map))
        } else {
            Err("Missing track data".to_string())
        }
    }

    fn user(edgee_event: Event, cred_map: Dict) -> Result<EdgeeRequest, String> {
        if let Data::User(ref data) = edgee_event.data {
            // check if edgee_payload.identify is empty
            if data.user_id.is_empty() && data.anonymous_id.is_empty() {
                return Err("user_id or anonymous_id is not set".to_string());
            }

            // Convert edgee_payload to segment Payload
            let mut segment_payload =
                SegmentPayload::new(&edgee_event, &cred_map, "identify".to_string())
                    .map_err(|e| e.to_string())?;

            // get edgee_payload.identify.properties and set segment_payload.traits with it
            let mut properties = HashMap::new();

            // iterate over page.properties and add them to properties
            for (key, value) in data.properties.clone().iter() {
                properties.insert(key.clone(), parse_value(value));
            }
            segment_payload.traits = Some(properties);

            Ok(build_edgee_request(segment_payload, &cred_map))
        } else {
            Err("Missing user data".to_string())
        }
    }
}

fn parse_value(value: &str) -> serde_json::Value {
    if value == "true" {
        serde_json::Value::from(true)
    } else if value == "false" {
        serde_json::Value::from(false)
    } else if value.parse::<f64>().is_ok() {
        serde_json::Value::Number(value.parse().unwrap())
    } else {
        serde_json::Value::String(value.to_string())
    }
}

fn build_edgee_request(segment_payload: SegmentPayload, cred_map: &Dict) -> EdgeeRequest {
    let cred: HashMap<String, String> = cred_map
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();

    let key = cred
        .get("segment_write_key")
        .unwrap_or(&String::new())
        .to_owned();
    let key = GeneralPurpose::new(&STANDARD, PAD).encode(format!("{}:", key));

    let mut headers = vec![];
    headers.push((String::from("authorization"), format!("Basic {}", key)));
    headers.push((
        String::from("content-type"),
        String::from("application/json"),
    ));

    EdgeeRequest {
        method: HttpMethod::Post,
        url: String::from("https://api.segment.io/v1/track"),
        headers,
        body: serde_json::to_string(&segment_payload).unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exports::edgee::protocols::provider::{
        Campaign, Client, Context, EventType, PageData, Session, TrackData, UserData,
    };
    use exports::edgee::protocols::provider::Consent;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    fn sample_user_data(edgee_id: String) -> UserData {
        return UserData {
            user_id: "123".to_string(),
            anonymous_id: "456".to_string(),
            edgee_id: edgee_id,
            properties: vec![
                ("prop1".to_string(), "value1".to_string()),
                ("prop2".to_string(), "10".to_string()),
            ],
        };
    }

    fn sample_user_data_without_anonymous_id() -> UserData {
        return UserData {
            user_id: "123".to_string(),
            anonymous_id: "".to_string(),
            edgee_id: "abc".to_string(),
            properties: vec![
                ("prop1".to_string(), "value1".to_string()),
                ("prop2".to_string(), "10".to_string()),
            ],
        };
    }

    fn sample_user_data_invalid_without_ids() -> UserData {
        return UserData {
            user_id: "".to_string(),
            anonymous_id: "".to_string(),
            edgee_id: "abc".to_string(),
            properties: vec![
                ("prop1".to_string(), "value1".to_string()),
                ("prop2".to_string(), "10".to_string()),
            ],
        };
    }

    fn sample_campaign_data() -> Campaign {
        return Campaign {
            name: "random".to_string(),
            source: "random".to_string(),
            medium: "random".to_string(),
            term: "random".to_string(),
            content: "random".to_string(),
            creative_format: "random".to_string(),
            marketing_tactic: "random".to_string(),
        };
    }

    fn sample_campaign_data_empty() -> Campaign {
        return Campaign {
            name: "".to_string(),
            source: "".to_string(),
            medium: "".to_string(),
            term: "".to_string(),
            content: "".to_string(),
            creative_format: "".to_string(),
            marketing_tactic: "".to_string(),
        };
    }

    fn sample_context(
        locale: String,
        session_start: bool,
        user_data: UserData,
        page_data: PageData,
        campaign_data: Campaign,
    ) -> Context {
        return Context {
            page: page_data,
            user: user_data,
            client: Client {
                city: "Paris".to_string(),
                ip: "192.168.0.1".to_string(),
                locale: locale,
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
            campaign: campaign_data,
            session: Session {
                session_id: "random".to_string(),
                previous_session_id: "random".to_string(),
                session_count: 2,
                session_start: session_start,
                first_seen: 123,
                last_seen: 123,
            },
        };
    }

    fn sample_page_data() -> PageData {
        return PageData {
            name: "page name".to_string(),
            category: "category".to_string(),
            keywords: vec!["value1".to_string(), "value2".into()],
            title: "page title".to_string(),
            url: "https://example.com/full-url?test=1".to_string(),
            path: "/full-path".to_string(),
            search: "?test=1".to_string(),
            referrer: "https://example.com/another-page".to_string(),
            properties: vec![
                ("prop1".to_string(), "false".to_string()),
                ("prop2".to_string(), "true".to_string()),
                ("currency".to_string(), "USD".to_string()),
            ],
        };
    }

    fn sample_page_data_empty() -> PageData {
        return PageData {
            name: "".to_string(),
            category: "".to_string(),
            keywords: vec![],
            title: "".to_string(),
            url: "".to_string(),
            path: "".to_string(),
            search: "".to_string(),
            referrer: "".to_string(),
            properties: vec![],
        };
    }

    fn sample_page_event(
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
            event_type: EventType::Page,
            data: Data::Page(sample_page_data()),
            context: sample_context(
                locale,
                session_start,
                sample_user_data(edgee_id),
                sample_page_data(),
                sample_campaign_data(),
            ),
            consent: consent,
        };
    }

    fn sample_page_event_without_context_page_data(
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
            event_type: EventType::Page,
            data: Data::Page(sample_page_data()),
            context: sample_context(
                locale,
                session_start,
                sample_user_data(edgee_id),
                sample_page_data_empty(),
                sample_campaign_data(),
            ),
            consent: consent,
        };
    }

    fn sample_page_event_without_context_campaign_data(
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
            event_type: EventType::Page,
            data: Data::Page(sample_page_data()),
            context: sample_context(
                locale,
                session_start,
                sample_user_data(edgee_id),
                sample_page_data_empty(),
                sample_campaign_data_empty(),
            ),
            consent: consent,
        };
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
            context: sample_context(
                locale,
                session_start,
                sample_user_data(edgee_id),
                sample_page_data(),
                sample_campaign_data(),
            ),
            consent: consent,
        };
    }

    fn sample_user_event(
        consent: Option<Consent>,
        edgee_id: String,
        locale: String,
        session_start: bool,
    ) -> Event {
        let user_data = sample_user_data(edgee_id.clone());
        return Event {
            uuid: Uuid::new_v4().to_string(),
            timestamp: 123,
            timestamp_millis: 123,
            timestamp_micros: 123,
            event_type: EventType::User,
            data: Data::User(user_data.clone()),
            context: sample_context(
                locale,
                session_start,
                user_data,
                sample_page_data(),
                sample_campaign_data(),
            ),
            consent: consent,
        };
    }

    fn sample_user_event_without_ids(
        consent: Option<Consent>,
        locale: String,
        session_start: bool,
    ) -> Event {
        let user_data = sample_user_data_invalid_without_ids();
        return Event {
            uuid: Uuid::new_v4().to_string(),
            timestamp: 123,
            timestamp_millis: 123,
            timestamp_micros: 123,
            event_type: EventType::User,
            data: Data::User(user_data.clone()),
            context: sample_context(
                locale,
                session_start,
                user_data,
                sample_page_data(),
                sample_campaign_data(),
            ),
            consent: consent,
        };
    }

    fn sample_user_event_without_anonymous_id(
        consent: Option<Consent>,
        locale: String,
        session_start: bool,
    ) -> Event {
        let user_data = sample_user_data_without_anonymous_id();
        return Event {
            uuid: Uuid::new_v4().to_string(),
            timestamp: 123,
            timestamp_millis: 123,
            timestamp_micros: 123,
            event_type: EventType::User,
            data: Data::User(user_data.clone()),
            context: sample_context(
                locale,
                session_start,
                user_data,
                sample_page_data(),
                sample_campaign_data(),
            ),
            consent: consent,
        };
    }

    fn sample_credentials() -> Vec<(String, String)> {
        return vec![
            ("segment_project_id".to_string(), "abc".to_string()),
            ("segment_write_key".to_string(), "abc".to_string()),
        ];
    }

    #[test]
    fn page_with_consent() {
        let event = sample_page_event(
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let credentials = sample_credentials();
        let result = SegmentComponent::page(event, credentials);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert_eq!(edgee_request.body.len() > 0, true);
        assert_eq!(
            edgee_request.url.starts_with("https://api.segment.io"),
            true
        );
        // add more checks (headers, querystring, etc.)
    }

    #[test]
    fn page_without_consent() {
        let event = sample_page_event(None, "abc".to_string(), "fr".to_string(), true);
        let credentials = sample_credentials();
        let result = SegmentComponent::page(event, credentials);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert_eq!(edgee_request.body.len() > 0, true);
    }

    #[test]
    fn page_with_edgee_id_uuid() {
        let event = sample_page_event(
            Some(Consent::Granted),
            Uuid::new_v4().to_string(),
            "fr".to_string(),
            true,
        );
        let credentials = sample_credentials();
        let result = SegmentComponent::page(event, credentials);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert_eq!(edgee_request.body.len() > 0, true);
    }

    #[test]
    fn page_with_empty_locale() {
        let event = sample_page_event(
            Some(Consent::Granted),
            Uuid::new_v4().to_string(),
            "".to_string(),
            true,
        );

        let credentials = sample_credentials();
        let result = SegmentComponent::page(event, credentials);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert_eq!(edgee_request.body.len() > 0, true);
    }

    #[test]
    fn page_without_context_page_data() {
        let event = sample_page_event_without_context_page_data(
            Some(Consent::Granted),
            Uuid::new_v4().to_string(),
            "".to_string(),
            true,
        );

        let credentials = sample_credentials();
        let result = SegmentComponent::page(event, credentials);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert_eq!(edgee_request.body.len() > 0, true);
    }

    #[test]
    fn page_without_context_campaign_data() {
        let event = sample_page_event_without_context_campaign_data(
            Some(Consent::Granted),
            Uuid::new_v4().to_string(),
            "".to_string(),
            true,
        );

        let credentials = sample_credentials();
        let result = SegmentComponent::page(event, credentials);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert_eq!(edgee_request.body.len() > 0, true);
    }

    #[test]
    fn page_not_session_start() {
        let event = sample_page_event(None, Uuid::new_v4().to_string(), "".to_string(), false);
        let credentials = sample_credentials();
        let result = SegmentComponent::page(event, credentials);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert_eq!(edgee_request.body.len() > 0, true);
    }

    #[test]
    fn page_without_project_id_fails() {
        let event = sample_page_event(None, "abc".to_string(), "fr".to_string(), true);
        let credentials: Vec<(String, String)> = vec![]; // empty
        let result = SegmentComponent::page(event, credentials); // this should panic!
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn page_without_write_key_fails() {
        let event = sample_page_event(None, "abc".to_string(), "fr".to_string(), true);
        let credentials: Vec<(String, String)> = vec![
            ("segment_project_id".to_string(), "abc".to_string()), // only project ID
        ];
        let result = SegmentComponent::page(event, credentials); // this should panic!
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn track_with_consent() {
        let event = sample_track_event(
            "event-name".to_string(),
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let credentials = sample_credentials();
        let result = SegmentComponent::track(event, credentials);
        assert_eq!(result.clone().is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert_eq!(edgee_request.body.len() > 0, true);
    }

    #[test]
    fn track_with_empty_name_fails() {
        let event = sample_track_event(
            "".to_string(),
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let credentials = sample_credentials();
        let result = SegmentComponent::track(event, credentials);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn user_event() {
        let event = sample_user_event(
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        let credentials = sample_credentials();
        let result = SegmentComponent::user(event, credentials);

        assert_eq!(result.clone().is_err(), false);
    }

    #[test]
    fn user_event_without_anonymous_id() {
        let event =
            sample_user_event_without_anonymous_id(Some(Consent::Granted), "fr".to_string(), true);
        let credentials = sample_credentials();
        let result = SegmentComponent::user(event, credentials);

        assert_eq!(result.clone().is_err(), false);
    }

    #[test]
    fn user_event_without_ids_fails() {
        let event = sample_user_event_without_ids(Some(Consent::Granted), "fr".to_string(), true);
        let credentials = sample_credentials();
        let result = SegmentComponent::user(event, credentials);

        assert_eq!(result.clone().is_err(), true);
        assert_eq!(
            result
                .clone()
                .err()
                .unwrap()
                .to_string()
                .contains("is not set"),
            true
        );
    }

    #[test]
    fn track_event_without_user_context_properties_and_empty_user_id() {
        let mut event = sample_track_event(
            "event-name".to_string(),
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            true,
        );
        event.context.user.properties = vec![]; // empty context user properties
        event.context.user.user_id = "".to_string(); // empty context user id
        let credentials: Vec<(String, String)> = sample_credentials();
        let result = SegmentComponent::track(event, credentials);
        //println!("Error: {}", result.clone().err().unwrap().to_string().as_str());
        assert_eq!(result.clone().is_err(), false);
    }
}
