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

        let truncated = call(
            "str",
            "truncate",
            vec![
                ("value", string("🙂 Dowe")),
                ("max", StdlibValue::Number("3".to_string())),
            ],
        );
        assert_eq!(
            evaluate(&truncated, |_| None).unwrap(),
            Value::String("🙂 D".to_string())
        );

        let parsed = call("parse", "int", vec![("value", string("42"))]);
        assert_eq!(
            evaluate(&parsed, |_| None).unwrap(),
            Value::Number(Number::from(42))
        );

        let greater = call(
            "math",
            "gte",
            vec![
                ("left", StdlibValue::Number("2".to_string())),
                ("right", StdlibValue::Number("2".to_string())),
            ],
        );
        assert_eq!(evaluate(&greater, |_| None).unwrap(), Value::Bool(true));

        let matching = call(
            "list",
            "filterContainsAny",
            vec![
                (
                    "values",
                    StdlibValue::Array(vec![StdlibValue::Object(vec![(
                        "content".to_string(),
                        string("landing page"),
                    )])]),
                ),
                ("field", string("content")),
                ("needles", StdlibValue::Array(vec![string("page")])),
            ],
        );
        assert_eq!(
            evaluate(&matching, |_| None)
                .unwrap()
                .as_array()
                .map(Vec::len),
            Some(1)
        );

        let joined = call(
            "list",
            "concat",
            vec![
                ("values", StdlibValue::Array(vec![string("a")])),
                ("other", StdlibValue::Array(vec![string("b")])),
            ],
        );
        assert_eq!(
            evaluate(&joined, |_| None).unwrap(),
            Value::Array(vec![
                Value::String("a".to_string()),
                Value::String("b".to_string())
            ])
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
        let equal = call(
            "str",
            "equals",
            vec![("value", string("1.0.25")), ("other", string("1.0.25"))],
        );
        assert_eq!(evaluate(&equal, |_| None).unwrap(), Value::Bool(true));

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
    fn hashes_receipts_and_stays_server_only() {
        let call = call("hash", "sha256", vec![("value", string("receipt"))]);
        let value = evaluate(&call, |_| None).expect("hash");
        assert_eq!(
            value,
            Value::String(
                "6f32860910ca0fb2a20c7fda143666b09dbf8db5238195c90a586fb542ff0cad".to_string()
            )
        );
        assert!(validate_call(&call, StdlibSurface::Views).is_err());
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
