mod segment_payload;

use base64::{
    alphabet::STANDARD,
    engine::{general_purpose::PAD, GeneralPurpose},
    Engine,
};
use exports::provider::{Dict, EdgeeRequest, Guest, Payload};
use segment_payload::SegmentPayload;
use std::collections::HashMap;

wit_bindgen::generate!({world: "data-collection"});
export!(SegmentComponent);

struct SegmentComponent;

impl Guest for SegmentComponent {
    fn page(edgee_payload: Payload, cred_map: Dict) -> Result<EdgeeRequest, String> {
        // create a new segment payload
        let mut segment_payload =
            SegmentPayload::new(&edgee_payload, &cred_map, "page".to_string())
                .map_err(|e| e.to_string())?;

        // page event properties
        let mut properties = HashMap::new();
        properties.insert(
            "path".to_string(),
            serde_json::Value::String(edgee_payload.page.path.clone()),
        );
        properties.insert(
            "title".to_string(),
            serde_json::Value::String(edgee_payload.page.title.clone()),
        );
        properties.insert(
            "url".to_string(),
            serde_json::Value::String(edgee_payload.page.url.clone()),
        );
        if !edgee_payload.page.referrer.is_empty() {
            properties.insert(
                "referrer".to_string(),
                serde_json::Value::String(edgee_payload.page.referrer.clone()),
            );
        }
        if !edgee_payload.page.search.is_empty() {
            properties.insert(
                "search".to_string(),
                serde_json::Value::String(edgee_payload.page.search.clone()),
            );
        }
        if edgee_payload.page.keywords.len() > 0 {
            // convert keywords to a json array of strings
            // clone keywords and transform them to a json array of strings like `let v = json!(["an", "array"]);`
            let keywords_json = serde_json::to_value(edgee_payload.page.keywords.clone());
            if keywords_json.is_ok() {
                properties.insert("keywords".to_string(), keywords_json.unwrap());
            }
        }

        // iterate over page.properties and add them to properties
        for (key, value) in edgee_payload.page.properties.clone().iter() {
            properties.insert(key.clone(), value.clone().parse().unwrap_or_default());
        }

        segment_payload.properties = Some(properties);

        Ok(build_edgee_request(segment_payload, &cred_map))
    }

    fn track(edgee_payload: Payload, cred_map: Dict) -> Result<EdgeeRequest, String> {
        // check if edgee_payload.track is empty
        if edgee_payload.track.name.is_empty() {
            return Err("Track is not set".to_string());
        }

        // create a new segment payload
        let mut segment_payload =
            SegmentPayload::new(&edgee_payload, &cred_map, "track".to_string())
                .map_err(|e| e.to_string())?;

        // event name and properties
        segment_payload.event = Option::from(edgee_payload.track.name.clone());
        let properties = edgee_payload
            .track
            .properties
            .iter()
            .map(|(key, value)| (key.clone(), value.clone().parse().unwrap_or_default()))
            .collect();
        segment_payload.properties = Some(properties);

        Ok(build_edgee_request(segment_payload, &cred_map))
    }

    fn identify(edgee_payload: Payload, cred_map: Dict) -> Result<EdgeeRequest, String> {
        // check if edgee_payload.identify is empty
        if edgee_payload.identify.user_id.is_empty()
            && edgee_payload.identify.anonymous_id.is_empty()
        {
            return Err("userId or anonymousId is not set".to_string());
        }

        // Convert edgee_payload to segment Payload
        let mut segment_payload =
            SegmentPayload::new(&edgee_payload, &cred_map, "identify".to_string())
                .map_err(|e| e.to_string())?;

        // get edgee_payload.identify.properties and set segment_payload.traits with it
        let properties = edgee_payload
            .identify
            .properties
            .iter()
            .map(|(key, value)| (key.clone(), value.clone().parse().unwrap_or_default()))
            .collect();
        segment_payload.traits = Some(properties);

        Ok(build_edgee_request(segment_payload, &cred_map))
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
        method: exports::provider::HttpMethod::Post,
        url: String::from("https://api.segment.io/v1/track"),
        headers,
        body: serde_json::to_string(&segment_payload).unwrap(),
    }
}
