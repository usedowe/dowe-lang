#[cfg(test)]
mod tests {
    use crate::helpers::read_path;
    use crate::{StdlibArgument, StdlibCall, StdlibSurface, StdlibValue, evaluate, validate_call};
    use serde_json::{Number, Value};

    fn call(namespace: &str, function: &str, args: Vec<(&str, StdlibValue)>) -> StdlibCall {
        StdlibCall {
            namespace: namespace.to_string(),
            function: function.to_string(),
            args: args
                .into_iter()
                .map(|(name, value)| StdlibArgument {
                    name: name.to_string(),
                    value,
                })
                .collect(),
        }
    }

    fn string(value: &str) -> StdlibValue {
        StdlibValue::String(value.to_string())
    }

    #[test]
    fn evaluates_string_parse_and_math_functions() {
        let trim = call("str", "trim", vec![("value", string("  Ada  "))]);
        assert_eq!(
            evaluate(&trim, |_| None).unwrap(),
            Value::String("Ada".to_string())
        );

        let parsed = call("parse", "int", vec![("value", string("42"))]);
        assert_eq!(
            evaluate(&parsed, |_| None).unwrap(),
            Value::Number(Number::from(42))
        );

        let sum = call(
            "math",
            "sum",
            vec![(
                "values",
                StdlibValue::Array(vec![
                    StdlibValue::Number("1".to_string()),
                    StdlibValue::Number("2".to_string()),
                    StdlibValue::Number("3".to_string()),
                ]),
            )],
        );
        assert_eq!(
            evaluate(&sum, |_| None).unwrap(),
            Value::Number(Number::from_f64(6.0).unwrap())
        );
    }

    #[test]
    fn parses_csv_with_header() {
        let parsed = call(
            "csv",
            "parse",
            vec![
                ("value", string("name,score\nAda,10\nLinus,8")),
                ("header", StdlibValue::Bool(true)),
            ],
        );
        let value = evaluate(&parsed, |_| None).unwrap();
        assert_eq!(
            read_path(&value, "rows.0.name"),
            Some(&Value::String("Ada".to_string()))
        );
        assert_eq!(
            read_path(&value, "rowCount"),
            Some(&Value::Number(Number::from(2)))
        );
    }

    #[test]
    fn sorts_stably_by_field() {
        let rows = StdlibValue::Array(vec![
            StdlibValue::Object(vec![
                ("id".to_string(), string("a")),
                ("score".to_string(), StdlibValue::Number("2".to_string())),
            ]),
            StdlibValue::Object(vec![
                ("id".to_string(), string("b")),
                ("score".to_string(), StdlibValue::Number("1".to_string())),
            ]),
        ]);
        let sorted = call(
            "sort",
            "by",
            vec![("values", rows), ("field", string("score"))],
        );
        let value = evaluate(&sorted, |_| None).unwrap();
        assert_eq!(
            read_path(&value, "0.id"),
            Some(&Value::String("b".to_string()))
        );
    }

    #[test]
    fn descending_sort_by_preserves_null_policy() {
        let rows = StdlibValue::Array(vec![
            StdlibValue::Object(vec![
                ("id".to_string(), string("a")),
                ("score".to_string(), StdlibValue::Number("2".to_string())),
            ]),
            StdlibValue::Object(vec![
                ("id".to_string(), string("b")),
                ("score".to_string(), StdlibValue::Null),
            ]),
            StdlibValue::Object(vec![
                ("id".to_string(), string("c")),
                ("score".to_string(), StdlibValue::Number("3".to_string())),
            ]),
            StdlibValue::Object(vec![("id".to_string(), string("d"))]),
            StdlibValue::Object(vec![
                ("id".to_string(), string("e")),
                ("score".to_string(), StdlibValue::Number("2".to_string())),
            ]),
        ]);
        let descending_last = call(
            "sort",
            "by",
            vec![
                ("values", rows.clone()),
                ("field", string("score")),
                ("direction", string("desc")),
                ("nulls", string("last")),
            ],
        );
        let descending_first = call(
            "sort",
            "by",
            vec![
                ("values", rows),
                ("field", string("score")),
                ("direction", string("desc")),
                ("nulls", string("first")),
            ],
        );

        let ids = |value: Value| {
            value
                .as_array()
                .expect("sorted rows")
                .iter()
                .map(|row| row["id"].as_str().expect("row id").to_string())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            ids(evaluate(&descending_last, |_| None).expect("descending last")),
            vec!["c", "a", "e", "b", "d"]
        );
        assert_eq!(
            ids(evaluate(&descending_first, |_| None).expect("descending first")),
            vec!["b", "d", "c", "a", "e"]
        );
    }

    #[test]
    fn handles_json_url_and_date() {
        let query = call(
            "url",
            "querySet",
            vec![
                ("value", string("/search")),
                ("name", string("q")),
                ("param", string("dowe lang")),
            ],
        );
        assert_eq!(
            evaluate(&query, |_| None).unwrap(),
            Value::String("/search?q=dowe%20lang".to_string())
        );

        let json = call(
            "json",
            "get",
            vec![
                (
                    "value",
                    StdlibValue::Object(vec![(
                        "user".to_string(),
                        StdlibValue::Object(vec![("name".to_string(), string("Ada"))]),
                    )]),
                ),
                ("path", string("user.name")),
            ],
        );
        assert_eq!(
            evaluate(&json, |_| None).unwrap(),
            Value::String("Ada".to_string())
        );

        let date = call(
            "date",
            "addDays",
            vec![
                ("value", string("2026-06-30T00:00:00Z")),
                ("days", StdlibValue::Number("2".to_string())),
            ],
        );
        assert_eq!(
            evaluate(&date, |_| None).unwrap(),
            Value::String("2026-07-02T00:00:00Z".to_string())
        );
    }

    #[test]
    fn generates_server_only_ulids() {
        let call = call("id", "ulid", Vec::new());
        let value = evaluate(&call, |_| None).expect("ulid");
        let value = value.as_str().expect("ulid string");

        assert_eq!(value.len(), 26);
        assert!(dowe_id::validate_ulid(value).is_ok());
        assert!(validate_call(&call, StdlibSurface::Views).is_err());
    }
}
