use anyhow::anyhow;
use chrono::{DateTime, TimeZone, Utc};
use serde::Serialize;
use std::collections::HashMap;

use crate::exports::edgee::protocols::data_collection::{Dict, Event};

#[derive(Debug, Default, Serialize)]
pub(crate) struct SegmentPayload {
    #[serde(rename = "projectId")]
    project_id: String,
    timestamp: DateTime<Utc>,
    #[serde(rename = "type")]
    pub(crate) event_type: String,
    pub(crate) context: Context,
    #[serde(rename = "userId", skip_serializing_if = "Option::is_none")]
    user_id: Option<String>,
    #[serde(rename = "anonymousId", skip_serializing_if = "Option::is_none")]
    anonymous_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) properties: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) traits: Option<HashMap<String, serde_json::Value>>,
}

impl SegmentPayload {
    pub fn new(edgee_event: &Event, settings: &Dict, event_type: String) -> anyhow::Result<Self> {
        let mut segment_payload = SegmentPayload {
            event_type,
            ..SegmentPayload::default()
        };

        let credentials: HashMap<String, String> = settings
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();

        if !credentials.contains_key("segment_project_id") {
            return Err(anyhow!("Segment project id is required"));
        }

        if !credentials.contains_key("segment_write_key") {
            return Err(anyhow!("Segment write key is required"));
        }

        segment_payload.project_id = credentials.get("segment_project_id").unwrap().to_string();
        // Convert i64 timestamp (with microseconds) to DateTime<Utc>
        segment_payload.timestamp = Utc.timestamp_micros(edgee_event.timestamp_micros).unwrap();

        // user_id
        if !edgee_event.context.user.user_id.is_empty() {
            segment_payload.user_id = Some(edgee_event.context.user.user_id.clone());
        }
        // anonymous_id
        if !edgee_event.context.user.anonymous_id.is_empty() {
            segment_payload.anonymous_id = Some(edgee_event.context.user.anonymous_id.clone());
        } else {
            segment_payload.anonymous_id = Some(edgee_event.context.user.edgee_id.to_string());
        }

        // add context.page
        let mut page = Page::default();
        if !edgee_event.context.page.title.is_empty() {
            page.title = Some(edgee_event.context.page.title.clone());
        }
        if !edgee_event.context.page.url.is_empty() {
            page.url = Some(edgee_event.context.page.url.clone());
        }
        if !edgee_event.context.page.path.is_empty() {
            page.path = Some(edgee_event.context.page.path.clone());
        }
        if !edgee_event.context.page.referrer.is_empty() {
            page.referrer = Some(edgee_event.context.page.referrer.clone());
        }
        if !edgee_event.context.page.search.is_empty() {
            page.search = Some(edgee_event.context.page.search.clone());
        }

        // set context.page (even if empty because SegmentComponent expects it)
        // see segment_payload.context.page.as_mut().unwrap().title = ...
        segment_payload.context.page = Some(page);

        // if edgee_payload.campaign is Some
        let mut campaign = Campaign::default();
        if !edgee_event.context.campaign.name.is_empty() {
            campaign.name = Some(edgee_event.context.campaign.name.clone());
        }
        if !edgee_event.context.campaign.source.is_empty() {
            campaign.source = Some(edgee_event.context.campaign.source.clone());
        }
        if !edgee_event.context.campaign.medium.is_empty() {
            campaign.medium = Some(edgee_event.context.campaign.medium.clone());
        }
        if !edgee_event.context.campaign.term.is_empty() {
            campaign.term = Some(edgee_event.context.campaign.term.clone());
        }
        if !edgee_event.context.campaign.content.is_empty() {
            campaign.content = Some(edgee_event.context.campaign.content.clone());
        }
        // set context.campaign only if it has any value
        if campaign.name.is_some()
            || campaign.source.is_some()
            || campaign.medium.is_some()
            || campaign.term.is_some()
            || campaign.content.is_some()
        {
            segment_payload.context.campaign = Some(campaign);
        }

        // if edgee_payload.client is Some
        if !edgee_event.context.client.ip.is_empty() {
            segment_payload.context.ip = Some(edgee_event.context.client.ip.clone());
        }
        if !edgee_event.context.client.locale.is_empty() {
            segment_payload.context.locale = Some(edgee_event.context.client.locale.clone());
        }
        let mut os = Os::default();
        if !edgee_event.context.client.os_name.is_empty() {
            os.name = Some(edgee_event.context.client.os_name.clone());
        }
        if !edgee_event.context.client.os_version.is_empty() {
            os.version = Some(edgee_event.context.client.os_version.clone());
        }
        // set context.os only if it has any value
        if os.name.is_some() || os.version.is_some() {
            segment_payload.context.os = Some(os);
        }

        let mut screen = Screen::default();
        if edgee_event.context.client.screen_width != 0 {
            screen.width = Some(edgee_event.context.client.screen_width.try_into()?);
        }
        if edgee_event.context.client.screen_height != 0 {
            screen.height = Some(edgee_event.context.client.screen_height.try_into()?);
        }
        if edgee_event.context.client.screen_density != 0.0 {
            screen.density = Some(edgee_event.context.client.screen_density);
        }
        // set context.screen only if it has any value
        if screen.width.is_some() || screen.height.is_some() || screen.density.is_some() {
            segment_payload.context.screen = Some(screen);
        }

        if !edgee_event.context.client.timezone.is_empty() {
            segment_payload.context.timezone = Some(edgee_event.context.client.timezone.clone());
        }

        if !edgee_event.context.client.user_agent.is_empty() {
            segment_payload.context.user_agent =
                Some(edgee_event.context.client.user_agent.clone());
        }

        Ok(segment_payload)
    }
}

#[derive(Serialize, Debug, Default)]
pub struct Context {
    #[serde(skip_serializing_if = "Option::is_none")]
    active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    app: Option<App>,
    #[serde(skip_serializing_if = "Option::is_none")]
    campaign: Option<Campaign>,
    #[serde(skip_serializing_if = "Option::is_none")]
    device: Option<Device>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    library: Option<Library>,
    #[serde(skip_serializing_if = "Option::is_none")]
    locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    network: Option<Network>,
    #[serde(skip_serializing_if = "Option::is_none")]
    os: Option<Os>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) page: Option<Page>,
    #[serde(skip_serializing_if = "Option::is_none")]
    referrer: Option<Referrer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    screen: Option<Screen>,
    #[serde(skip_serializing_if = "Option::is_none")]
    group_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_agent: Option<String>,
}

#[derive(Serialize, Debug)]
struct App {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    build: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace: Option<String>,
}

#[derive(Serialize, Debug, Default)]
struct Campaign {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    medium: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    term: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
}

#[derive(Serialize, Debug)]
struct Device {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(rename = "advertisingId", skip_serializing_if = "Option::is_none")]
    advertising_id: Option<String>,
    #[serde(rename = "adTrackingEnabled", skip_serializing_if = "Option::is_none")]
    ad_tracking_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    manufacturer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    type_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    token: Option<String>,
}

#[derive(Serialize, Debug)]
struct Library {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
}

#[derive(Serialize, Debug)]
struct Network {
    #[serde(skip_serializing_if = "Option::is_none")]
    bluetooth: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    carrier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cellular: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    wifi: Option<bool>,
}

#[derive(Serialize, Debug, Default)]
struct Os {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
}

#[derive(Serialize, Debug, Default)]
pub(crate) struct Page {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) referrer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) search: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) url: Option<String>,
}

#[derive(Serialize, Debug)]
struct Referrer {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    type_: Option<String>,
}

#[derive(Serialize, Debug, Default)]
struct Screen {
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    density: Option<f32>,
}
