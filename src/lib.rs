use std::collections::HashMap;

use base64::{
    alphabet::STANDARD,
    engine::{general_purpose::PAD, GeneralPurpose},
    Engine,
};
use chrono::{DateTime, Utc};
use exports::provider::{Dict, EdgeeRequest, Guest, Payload};
use serde::Serialize;

mod string_ext;
use string_ext::StringExt;

wit_bindgen::generate!({world: "data-collection"});
export!(SegmentComponent);

struct SegmentComponent;

impl SegmentComponent {
    fn build_headers(p: &Payload, cred: HashMap<String, String>) -> Vec<(String, String)> {
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
        headers.push((
            String::from("user-agent"),
            String::from(&p.client.user_agent),
        ));
        headers.push((String::from("x-forwarded-for"), String::from(&p.client.ip)));
        return headers;
    }
}

impl Guest for SegmentComponent {
    fn page(p: Payload, cred_map: Dict) -> Result<EdgeeRequest, String> {
        let cred: HashMap<String, String> = cred_map
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();

        if cred.get("segment_project_id").is_none() {
            return Err(String::from("Segment project id is required"));
        }

        if cred.get("segment_write_key").is_none() {
            return Err(String::from("Segment write key is required"));
        }

        let mut payload = SegmentPayload::new(p.clone(), cred.clone());
        payload.event_type = String::from("page");

        let mut properties = HashMap::new();
        properties.insert(String::from("path"), p.page.path.to_string());
        properties.insert(String::from("title"), p.page.title.to_string());
        properties.insert(String::from("url"), p.page.url.to_string());
        properties.insert(String::from("referrer"), p.page.referrer.to_string());
        properties.insert(String::from("search"), p.page.search.to_string());
        properties.insert(String::from("keywords"), p.page.keywords.join(","));

        for (key, value) in p.page.properties.iter() {
            properties.insert(key.to_string(), value.to_string());
        }

        payload.properties = properties;

        Ok(EdgeeRequest {
            method: exports::provider::HttpMethod::Post,
            url: String::from("https://api.segment.io/v1/track"),
            headers: SegmentComponent::build_headers(&p, cred.clone()),
            body: serde_json::to_string(&payload).unwrap(),
        })
    }

    fn track(p: Payload, cred_map: Dict) -> Result<EdgeeRequest, String> {
        let cred: HashMap<String, String> = cred_map
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();

        if cred.get("segment_project_id").is_none() {
            return Err(String::from("Segment project id is required"));
        }

        if cred.get("segment_write_key").is_none() {
            return Err(String::from("Segment write key is required"));
        }

        if p.track.name.is_empty() {
            return Err(String::from("No tracking"));
        }

        let mut payload = SegmentPayload::new(p.clone(), cred.clone());
        payload.event_type = String::from("track");
        payload.event = p.track.name.clone();
        payload.properties = p
            .track
            .properties
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        Ok(EdgeeRequest {
            method: exports::provider::HttpMethod::Post,
            url: String::from("https://api.segment.io/v1/track"),
            headers: SegmentComponent::build_headers(&p, cred.clone()),
            body: serde_json::to_string(&payload).unwrap(),
        })
    }

    fn identify(p: Payload, cred_map: Dict) -> Result<EdgeeRequest, String> {
        let cred: HashMap<String, String> = cred_map
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();

        if cred.get("segment_project_id").is_none() {
            return Err(String::from("Segment project id is required"));
        }

        if cred.get("segment_write_key").is_none() {
            return Err(String::from("Segment write key is required"));
        }

        if p.track.name.is_empty() {
            return Err(String::from("No tracking"));
        }

        let mut payload = SegmentPayload::new(p.clone(), cred.clone());
        payload.event_type = String::from("identify");

        if p.identify.anonymous_id.is_empty() && p.identify.user_id.is_empty() {
            return Err(String::from("No user id or anonymous id"));
        }

        payload.traits = p
            .identify
            .properties
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        Ok(EdgeeRequest {
            method: exports::provider::HttpMethod::Post,
            url: String::from("https://api.segment.io/v1/identify"),
            headers: SegmentComponent::build_headers(&p, cred.clone()),
            body: serde_json::to_string(&payload).unwrap(),
        })
    }
}

#[derive(Debug, Default, Serialize)]
struct SegmentPayload {
    #[serde(rename = "projectId")]
    project_id: String,
    timestamp: DateTime<Utc>,
    #[serde(rename = "type")]
    event_type: String,
    context: Context,
    #[serde(rename = "userId", skip_serializing_if = "String::is_empty")]
    user_id: String,
    #[serde(rename = "anonymousId", skip_serializing_if = "String::is_empty")]
    anonymous_id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    category: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    properties: HashMap<String, String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    event: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    traits: HashMap<String, String>,
}

impl SegmentPayload {
    fn new(p: Payload, cred: HashMap<String, String>) -> Self {
        let mut data = Self::default();

        data.project_id = cred
            .get("segment_project_id")
            .map(String::from)
            .unwrap_or_default();

        data.timestamp = DateTime::from_timestamp_millis(p.timestamp).unwrap();
        data.user_id = p.identify.user_id;
        data.anonymous_id = p.identify.anonymous_id.or(&p.identify.edgee_id).to_string();

        if !p.page.url.is_empty() {
            let mut page = Page::default();
            page.title = p.page.title;
            page.url = p.page.url;
            page.path = p.page.path;
            page.referrer = p.page.referrer;
            page.search = p.page.search;
            data.context.page = Some(page);
        }

        if !p.campaign.name.is_empty() {
            let mut campaign = Campaign::default();
            campaign.name = p.campaign.name;
            campaign.source = p.campaign.source;
            campaign.medium = p.campaign.medium;
            campaign.term = p.campaign.term;
            campaign.content = p.campaign.content;
            data.context.campaign = Some(campaign);
        }

        data.context.ip = p.client.ip;
        data.context.locale = p.client.locale;

        data.context.os = Some(Os {
            name: p.client.os_name,
            version: p.client.os_version,
        });

        if p.client.screen_width != 0 && p.client.screen_height != 0 {
            data.context.screen = Some(Screen {
                width: Some(p.client.screen_width),
                height: Some(p.client.screen_height),
                density: Some(p.client.screen_density),
            });
        }

        data.context.timezone = p.client.timezone;
        data.context.user_agent = p.client.user_agent;

        return data;
    }
}

#[derive(Debug, Default, Serialize)]
struct Context {
    #[serde(skip_serializing_if = "Option::is_none")]
    active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    app: Option<App>,
    #[serde(skip_serializing_if = "Option::is_none")]
    campaign: Option<Campaign>,
    #[serde(skip_serializing_if = "Option::is_none")]
    device: Option<Device>,
    #[serde(skip_serializing_if = "String::is_empty")]
    ip: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    library: Option<Library>,
    #[serde(skip_serializing_if = "String::is_empty")]
    locale: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    network: Option<Network>,
    #[serde(skip_serializing_if = "Option::is_none")]
    os: Option<Os>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<Page>,
    #[serde(skip_serializing_if = "Option::is_none")]
    referrer: Option<Referrer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    screen: Option<Screen>,
    #[serde(skip_serializing_if = "String::is_empty")]
    group_id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    timezone: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    user_agent: String,
}

#[derive(Debug, Default, Serialize)]
struct App {
    #[serde(skip_serializing_if = "String::is_empty")]
    name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    version: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    build: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    namespace: String,
}

#[derive(Debug, Default, Serialize)]
struct Campaign {
    #[serde(skip_serializing_if = "String::is_empty")]
    name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    source: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    medium: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    term: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    content: String,
}

#[derive(Debug, Default, Serialize)]
struct Device {
    #[serde(skip_serializing_if = "String::is_empty")]
    id: String,
    #[serde(rename = "advertisingId", skip_serializing_if = "String::is_empty")]
    advertising_id: String,
    #[serde(rename = "adTrackingEnabled", skip_serializing_if = "Option::is_none")]
    ad_tracking_enabled: Option<bool>,
    #[serde(skip_serializing_if = "String::is_empty")]
    manufacturer: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    model: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    type_: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    token: String,
}

#[derive(Debug, Default, Serialize)]
struct Library {
    #[serde(skip_serializing_if = "String::is_empty")]
    name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    version: String,
}

#[derive(Debug, Default, Serialize)]
struct Network {
    #[serde(skip_serializing_if = "Option::is_none")]
    bluetooth: Option<bool>,
    #[serde(skip_serializing_if = "String::is_empty")]
    carrier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    cellular: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    wifi: Option<bool>,
}

#[derive(Debug, Default, Serialize)]
struct Os {
    #[serde(skip_serializing_if = "String::is_empty")]
    name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    version: String,
}

#[derive(Debug, Default, Serialize)]
struct Page {
    #[serde(skip_serializing_if = "String::is_empty")]
    path: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    referrer: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    search: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    title: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    url: String,
}

#[derive(Debug, Default, Serialize)]
struct Referrer {
    #[serde(skip_serializing_if = "String::is_empty")]
    id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    type_: String,
}

#[derive(Debug, Default, Serialize)]
struct Screen {
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    density: Option<i32>,
}
