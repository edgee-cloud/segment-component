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
            if data.keywords.len() > 0 {
                let keywords_json = serde_json::to_value(data.keywords.clone());
                if keywords_json.is_ok() {
                    properties.insert("keywords".to_string(), keywords_json.unwrap());
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
    } else if let Some(_v) = value.parse::<f64>().ok() {
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
