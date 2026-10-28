//! RFC 7386 merge patch (POL-09).

use serde_json::Value;

/// POL-09: applies a JSON Merge Patch, replacing arrays and removing null-valued object members.
pub fn merge_patch(mut base: Value, patch: &Value) -> Value {
    json_patch::merge(&mut base, patch);
    base
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::merge_patch;

    #[test]
    fn merge_patch_rfc7386_vectors() {
        let vectors = [
            (json!({"a":"b"}), json!({"a":"c"}), json!({"a":"c"})),
            (json!({"a":"b"}), json!({"b":"c"}), json!({"a":"b","b":"c"})),
            (json!({"a":"b"}), json!({"a":null}), json!({})),
            (json!({"a":"b","b":"c"}), json!({"a":null}), json!({"b":"c"})),
            (json!({"a":["b"]}), json!({"a":"c"}), json!({"a":"c"})),
            (json!({"a":"c"}), json!({"a":["b"]}), json!({"a":["b"]})),
            (
                json!({"a":{"b":"c"}}),
                json!({"a":{"b":"d"}}),
                json!({"a":{"b":"d"}}),
            ),
            (json!({"a":[{"b":"c"}]}), json!({"a":[1]}), json!({"a":[1]})),
            (json!(["a", "b"]), json!(["c", "d"]), json!(["c", "d"])),
            (json!({"a":"b"}), json!(["c"]), json!(["c"])),
            (json!({"a":"foo"}), Value::Null, Value::Null),
            (json!({"a":"b"}), json!({}), json!({"a":"b"})),
            (json!({"a":{"b":"c"}}), json!({"a":{"b":null}}), json!({"a":{}})),
            (json!({"a":[1,2]}), json!({"a":[3,4]}), json!({"a":[3,4]})),
            (json!({"a":{"x":1}}), json!({"a":{"x":null}}), json!({"a":{}})),
        ];

        for (target, patch, expected) in vectors {
            let target = merge_patch(target, &patch);
            assert_eq!(target, expected);
        }
    }
}
