use serde_json::{Map, Value};
use std::error::Error;
use url::form_urlencoded;

pub fn parse_to_json(input: &str) -> Result<Value, Box<dyn Error>> {
    // 무조건 json 파싱 시도
    match serde_json::from_str(input) {
        Ok(json) => Ok(json),
        Err(_) => {
            let mut map = serde_json::Map::new();
            for (key, value) in form_urlencoded::parse(input.as_bytes()) {
                insert_into_json_map(&mut map, &key, &value);
            }

            Ok(Value::Object(map))
        }
    }
}

fn insert_into_json_map(result: &mut Map<String, Value>, key: &str, value: &str) {
    // parts examples:
    // key=value: ["key"]
    // a[]=value: ["a", "]"]
    // foo[bar]=value: ["foo", "bar]"]
    let parts: Vec<&str> = key.split('[').collect();

    let (main_key, sub_key) = match parts.as_slice() {
        // key=value
        [main_key] => (main_key, None),
        // key[]=value OR key[sub_key]=value
        [main_key, sub_key] => (main_key, Some(sub_key.trim_end_matches(']'))),
        _ => panic!("Unexpected key format"),
    };

    match sub_key {
        None => {
            result.insert(main_key.to_string(), Value::String(value.to_string()));
        }
        Some(sub_key) => {
            if sub_key.is_empty() {
                // 배열 처리
                let array = result
                    .entry(main_key.to_string())
                    .or_insert_with(|| Value::Array(Vec::new()));

                if let Value::Array(ref mut arr) = array {
                    arr.push(Value::String(value.to_string()));
                }
            } else {
                // 중첩된 객체 처리
                let nested_object = result
                    .entry(main_key.to_string())
                    .or_insert_with(|| Value::Object(Map::new()));

                if let Value::Object(ref mut nested_map) = nested_object {
                    nested_map.insert(sub_key.to_string(), Value::String(value.to_string()));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::decode_input;

    #[test]
    fn test_decode_input_euc_kr() {
        let input = "%BE%C8%B3%E7%C7%CF%BC%BC%BF%E4";
        let result = decode_input(input, encoding_rs::EUC_KR).unwrap();
        assert_eq!(result, "안녕하세요");
    }

    #[test]
    fn test_decode_input_utf8() {
        let input = "%EC%95%88%EB%85%95%ED%95%98%EC%84%B8%EC%9A%94";
        let result = decode_input(input, encoding_rs::UTF_8).unwrap();
        assert_eq!(result, "안녕하세요");
    }

    #[test]
    fn test_decode_input_with_query_params() {
        let input = "key=%BE%C8%B3%E7&arr[]=value2&somemap[key3]=value3";
        let result = decode_input(input, encoding_rs::EUC_KR).unwrap();
        assert_eq!(result, "key=안녕&arr[]=value2&somemap[key3]=value3");
    }

    #[test]
    fn test_parse_to_json_form_urlencoded() {
        let input = "key1=value1&key2=value2&arr[]=item1&arr[]=item2";
        let result = parse_to_json(input).unwrap();
        let expected = serde_json::json!({
            "key1": "value1",
            "key2": "value2",
            "arr": ["item1", "item2"]
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_to_json_json() {
        let input = r#"{"key":"value","nested":{"array":[1,2,3]}}"#;
        let result = parse_to_json(input).unwrap();
        let expected = serde_json::json!({
            "key": "value",
            "nested": {
                "array": [1, 2, 3]
            }
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn test_complex_nested_structure() {
        let input = "parent[child]=value1&parent[child1][child2]=value2&arr[]=item1&arr[]=item2";
        let result = parse_to_json(input).unwrap();
        let expected = serde_json::json!({
            "parent": {
                "child": "value1",
                "child1": {
                    "child2": "value2"
                }
            },
            "arr": ["item1", "item2"]
        });
        assert_eq!(result, expected);
    }
}
