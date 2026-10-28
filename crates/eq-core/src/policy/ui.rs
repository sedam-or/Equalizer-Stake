//! Schema renderer hints (POL-06).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::module::ModuleId;

/// POL-06: setup step 3 or advanced settings section.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Group {
    Essentials,
    Advanced,
}

/// POL-06: supported policy renderer override.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Widget {
    Toggle,
    Number,
    Money,
    Percent,
    Duration,
    Time,
    WeekdayHours,
    DayOfMonth,
    Tiers,
    Chips,
    Select,
    TextareaI18n,
    MapPin,
    OfferingRef,
    Hidden,
}

/// POL-06: display suffix or value formatting.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    Minutes,
    Hours,
    Days,
    Meters,
    Percent,
    Money,
}

/// POL-06: builder for the JSON value stored in a schema's `x-ui` extension.
#[derive(Clone, Debug, Default)]
pub struct XUi(Map<String, Value>);

impl XUi {
    /// Starts a renderer hint with a group and sort order.
    pub fn new(group: Group, order: u32) -> Self {
        let mut values = Map::new();
        values.insert("group".into(), json!(group));
        values.insert("order".into(), json!(order));
        Self(values)
    }

    /// Sets the renderer override.
    pub fn widget(mut self, widget: Widget) -> Self {
        self.0.insert("widget".into(), json!(widget));
        self
    }

    /// Sets the display unit.
    pub fn unit(mut self, unit: Unit) -> Self {
        self.0.insert("unit".into(), json!(unit));
        self
    }

    /// Adds a visibility predicate expressed as a JSON Pointer and expected value.
    pub fn visible_if(mut self, path: impl Into<String>, equals: Value) -> Self {
        self.0.insert(
            "visible_if".into(),
            json!({"path": path.into(), "equals": equals}),
        );
        self
    }

    /// Restricts the field to tenants with the named module enabled.
    pub fn requires_module(mut self, module: ModuleId) -> Self {
        self.0.insert("requires_module".into(), json!(module));
        self
    }

    /// Returns the schema extension value.
    pub fn into_value(self) -> Value {
        Value::Object(self.0)
    }
}
