//! Plugin wire protocol `source/1`: the host writes one request line to stdin, the
//! plugin answers one response line on stdout and exits.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ref {
    pub provider: String,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum Request {
    Browse { category: Ref, page: u32 },
    Search { query: String, page: u32 },
    Series { series: Ref },
    Episode { episode: Ref },
    Resolve { episode: Ref },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    #[serde(rename = "ref")]
    pub reference: Ref,
    pub title: String,
    /// What selecting the item opens. Absent (older plugins): a container.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<ItemKind>,
}

/// What a listed item is to the host. How a site groups its works is the plugin's
/// business; the host only browses into containers and queues playables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    /// `list` it again for the items under it.
    Container,
    /// `resolve` turns it into streams; a listing's playables become the play queue.
    Playable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stream {
    pub url: String,
    #[serde(default)]
    pub headers: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Response {
    Page {
        items: Vec<Item>,
        next_page: Option<u32>,
    },
    Adjacency {
        previous: Option<Ref>,
        next: Option<Ref>,
    },
    Streams {
        streams: Vec<Stream>,
    },
    Error {
        kind: String,
        message: String,
    },
}
