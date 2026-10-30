//! Headless half of the window: the Meari document, reading what the user did from its
//! source state, and turning that into the settings page protocol messages.
use crate::source::{self, BUTTONS, Row, StateText};
use meari::application::source_modules::ModuleProgram;
use meari::domain::{Value as MeariValue, build::BuildPath, component_state::StateValue};
use meari_native::domain::{Viewport, widgets::Action};
use meari_native::infrastructure::document::InteractiveDocument;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

pub const VIEWPORT: Viewport = Viewport {
    width: 760,
    height: 720,
    dark: true,
};
const ENTRY: &str = "settings/settings-window.tsx";

/// What the page edits look like now versus the page model.
pub struct Session {
    rows: Vec<Row>,
    counts: BTreeMap<&'static str, i64>,
    page: Value,
}

fn meari_props(page: &Value, errors: &[String]) -> BTreeMap<String, MeariValue> {
    source::props(page, errors)
        .into_iter()
        .map(|(k, v)| {
            let v = match v {
                Value::Bool(b) => MeariValue::Bool(b),
                other => MeariValue::String(other.as_str().unwrap_or_default().to_owned()),
            };
            (k.to_owned(), v)
        })
        .collect()
}

fn state(document: &InteractiveDocument) -> Result<BTreeMap<String, StateValue>, String> {
    let projection = document.module_projection()?;
    let mut values = BTreeMap::new();
    for commit in projection.instances.values() {
        values.extend(commit.values.clone());
    }
    Ok(values)
}

fn count(value: Option<&StateValue>) -> i64 {
    match value {
        Some(StateValue::Number(n)) => n.as_f64().map_or(0, |n| n as i64),
        _ => 0,
    }
}

fn text(value: Option<&StateValue>) -> Option<StateText> {
    match value? {
        StateValue::String(s) => Some(StateText::Text(s.clone())),
        StateValue::Bool(b) => Some(StateText::Bool(*b)),
        _ => None,
    }
}

impl Session {
    /// Mounts the page as a Meari document (fixed rows, initial values in source state).
    pub fn mount(page: Value) -> Result<(Self, InteractiveDocument), String> {
        let (tsx, rows) = source::build(&page);
        let entry = BuildPath::try_from(ENTRY.to_owned())?;
        let graph = meari::parse_module_graph(&entry, &BTreeMap::from([(entry.clone(), tsx)]))
            .map_err(|e| format!("{e:?}"))?;
        let program = ModuleProgram::mount(
            graph,
            meari_props(&page, &[]),
            "kinekino-settings".into(),
            1,
        )
        .map_err(|e| format!("{e:?}"))?;
        let styles = crate::styles::prepare()?;
        let document = InteractiveDocument::from_module_program_with_assets(
            program,
            &styles.css,
            VIEWPORT,
            styles.assets,
        )?;
        let counts = BUTTONS.iter().map(|b| (*b, 0)).collect();
        Ok((Self { rows, counts, page }, document))
    }

    pub fn page(&self) -> &Value {
        &self.page
    }

    /// Edited rows as typed JSON (`key → value`) — only rows whose text differs.
    pub fn edits(&self, document: &InteractiveDocument) -> Result<Map<String, Value>, String> {
        let state = state(document)?;
        let mut out = Map::new();
        for row in &self.rows {
            let now = text(state.get(&row.slot)).ok_or("settings state slot missing")?;
            if now != row.shown {
                out.insert(row.key.clone(), source::typed(&row.kind, &now));
            }
        }
        Ok(out)
    }

    /// Buttons clicked since the last call, as page protocol messages in click order.
    pub fn messages(&mut self, document: &InteractiveDocument) -> Result<Vec<String>, String> {
        let state = state(document)?;
        let mut out = Vec::new();
        for button in BUTTONS {
            let now = count(state.get(&format!("{button}Count")));
            let seen = self.counts.insert(button, now).unwrap_or_default();
            for _ in seen..now {
                out.push(
                    match button {
                        "apply" => json!({"op": "apply", "edits": self.edits(document)?}),
                        "reset" => json!({"op": "reset"}),
                        "close" => json!({"op": "close"}),
                        "dialogContinue" => json!({"op": "dialog", "action": "continue"}),
                        _ => json!({"op": "dialog", "action": "cancel"}),
                    }
                    .to_string(),
                );
            }
        }
        Ok(out)
    }

    /// A new page (after a successful apply) becomes the baseline; errors are shown.
    pub fn update_props(
        &mut self,
        page: Option<Value>,
        errors: &[String],
    ) -> BTreeMap<String, MeariValue> {
        if let Some(page) = page {
            for row in &mut self.rows {
                if let Some(field) = page["fields"]
                    .as_array()
                    .and_then(|f| f.iter().find(|f| f["key"] == row.key.as_str()))
                {
                    row.shown = source::shown(&row.kind, &field["value"]);
                }
            }
            self.page = page;
        }
        meari_props(&self.page, errors)
    }

    /// Puts every row back to the page value through the same input routes a user uses.
    pub fn reset(&self, document: &mut InteractiveDocument) -> Result<(), String> {
        let state = state(document)?;
        for (index, row) in self.rows.iter().enumerate() {
            let now = text(state.get(&row.slot));
            if now.as_ref() == Some(&row.shown) {
                continue;
            }
            let base = format!("field-{}", self.field_index(index));
            match &row.shown {
                StateText::Bool(_) => activate(document, &base)?,
                StateText::Text(t) if row.kind == "choice" => {
                    let option = self.page["fields"][self.field_index(index)]["options"]
                        .as_array()
                        .and_then(|o| o.iter().position(|x| x == t.as_str()))
                        .ok_or("choice option missing")?;
                    activate(document, &format!("{base}-{option}"))?
                }
                StateText::Text(t) => {
                    let path = path_of(document, &base)?;
                    document.set_input_value(&path, t)?;
                }
            }
        }
        Ok(())
    }

    fn field_index(&self, row: usize) -> usize {
        let key = &self.rows[row].key;
        self.page["fields"]
            .as_array()
            .and_then(|f| f.iter().position(|f| f["key"] == key.as_str()))
            .unwrap_or(row)
    }

    /// Rows on screen and whether the dialog / errors are visible (smoke evidence).
    pub fn rendered(&self, document: &InteractiveDocument) -> Value {
        let elements = document.elements();
        let rows = elements
            .iter()
            .filter(|e| {
                e.attributes
                    .get("class")
                    .is_some_and(|c| c.contains("aw-settings-row"))
            })
            .count();
        let visible = |id: &str| {
            elements
                .iter()
                .find(|e| e.id == id)
                .is_some_and(|e| !e.attributes.contains_key("hidden"))
        };
        let errors = elements
            .iter()
            .find(|e| e.id == "errors-text")
            .map(|e| {
                e.text_runs
                    .iter()
                    .map(|r| r.text.as_str())
                    .collect::<String>()
            })
            .unwrap_or_default();
        json!({"rendered": rows, "dialog": visible("dialog"), "errors_shown": errors})
    }
}

pub fn path_of(document: &InteractiveDocument, id: &str) -> Result<String, String> {
    document
        .elements()
        .iter()
        .find(|e| e.id == id)
        .map(|e| e.path.clone())
        .ok_or_else(|| format!("settings element {id} missing"))
}

pub fn activate(document: &mut InteractiveDocument, id: &str) -> Result<(), String> {
    let path = path_of(document, id)?;
    document.dispatch(Action::Activate { path }).map(|_| ())
}
