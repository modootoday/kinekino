//! Display-only live catalog input; source rows are not browser or playback authority.
use meari::domain::{Prop, PropType, Value, array_value::ArrayValue, record_value::RecordValue};
use std::collections::BTreeMap;
pub struct CatalogRow {
    pub key: String,
    pub title: String,
    pub disabled: bool,
}
#[derive(Default)]
pub struct CatalogBrowserView {
    pub series: Vec<CatalogRow>,
    pub episodes: Vec<CatalogRow>,
    pub selected_series: String,
    pub selected_episode: String,
    pub status: String,
    pub error: String,
    pub busy: bool,
    pub can_poll: bool,
    pub can_cancel: bool,
    pub can_read_series: bool,
    pub can_read_episodes: bool,
    pub can_inspect: bool,
    pub can_play: bool,
    pub can_release: bool,
}
pub fn props(view: CatalogBrowserView) -> Result<BTreeMap<String, Value>, String> {
    let mut props = BTreeMap::from([
        ("catalogCurrentEpisode".into(), Value::String(String::new())),
        (
            "catalogConnectingEpisode".into(),
            Value::String(String::new()),
        ),
        (
            "catalogEpisodeConnectionLabel".into(),
            Value::String(String::new()),
        ),
        ("catalogSeriesRows".into(), rows(view.series)?),
        ("catalogEpisodeRows".into(), rows(view.episodes)?),
    ]);
    for (key, value) in [
        ("catalogSelectedSeries", view.selected_series),
        ("catalogSelectedEpisode", view.selected_episode),
        ("catalogStatus", view.status),
        ("catalogError", view.error),
    ] {
        props.insert(key.into(), Value::String(value));
    }
    for (key, value) in [
        ("catalogBusy", view.busy),
        ("catalogPollDisabled", !view.can_poll),
        ("catalogCancelDisabled", !view.can_cancel),
        ("catalogReadSeriesDisabled", !view.can_read_series),
        ("catalogReadEpisodesDisabled", !view.can_read_episodes),
        ("catalogInspectDisabled", !view.can_inspect),
        ("catalogPlayDisabled", !view.can_play),
        ("catalogReleaseDisabled", !view.can_release),
    ] {
        props.insert(key.into(), Value::Bool(value));
    }
    Ok(props)
}
fn rows(rows: Vec<CatalogRow>) -> Result<Value, String> {
    let schema = vec![
        Prop {
            name: "key".into(),
            kind: PropType::String,
        },
        Prop {
            name: "title".into(),
            kind: PropType::String,
        },
        Prop {
            name: "disabled".into(),
            kind: PropType::Bool,
        },
    ];
    let items = rows
        .into_iter()
        .map(|row| {
            RecordValue::new(
                schema.clone(),
                BTreeMap::from([
                    ("key".into(), Value::String(row.key)),
                    ("title".into(), Value::String(row.title)),
                    ("disabled".into(), Value::Bool(row.disabled)),
                ]),
            )
            .map(Value::Record)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let value = ArrayValue {
        element: PropType::Record(schema),
        items,
    };
    value.validate()?;
    Ok(Value::Array(value))
}
