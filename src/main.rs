use crate::parser::{DataProcessor, FormURLEncodedParser, JsonParser};

mod parser;

fn main() {
    let json_input =
        b"{\x22key1\x22:\x22value2\x22,\x22key2\x22:\x22value2\x22,\x22key3\x22:\x22key3\x22}";

    // JsonParser를 사용하는 DataProcessor 인스턴스 생성
    let json_processor = DataProcessor::new(Box::new(JsonParser));
    let result = json_processor.process(json_input);
    println!("JSON Parsing Result: {:?}", result);

    let form_data = vec![
        b"name=John+Doe&age=30".to_vec(),
        // {"key":"value","a":["123","false","last"],"b":["str"],"c":["3.5"]}
        b"key=value&a[]=123&a[]=false&b[]=str&c[]=3.5&a[]=last".to_vec(),
        // {"arr":{"0":"A","1":"B","9":"C","10":"D","foo":"E","11":"F","15.1":"G","12":"H"}}
        b"arr[]=A&arr[]=B&arr[9]=C&arr[]=D&arr[foo]=E&arr[]=F&arr[15.1]=G&arr[]=H".to_vec(),
        // {"a":"<==  yolo swag  ==>","b":"###Yolo Swag###"}
        b"a=%3c%3d%3d%20%20yolo+swag++%3d%3d%3e&b=%23%23%23Yolo+Swag%23%23%23".to_vec(),
        // {"first":"value","arr":["foo bar","baz"],"foo":{"bar":"foobar"},"test_field":"testing"}
        b"first=value&arr[]=foo+bar&arr[]=baz&foo[bar]=foobar&test.field=testing".to_vec(),
        // {"a":"<==  url encoded  ==>","b":"###Url Encoded###"}
        b"a=%3c%3d%3d%20%20url+encoded++%3d%3d%3e&b=%23%23%23Url+Encoded%23%23%23".to_vec(),
        b"alias=\xEA\xB4\x91\xEC\xA3\xBC\xEC\xB9\xB4\xEB\x93\x9C".to_vec(),
    ];

    // pre process
    // - 이스케이프 시퀀스 치환
    // - 퍼센트 디코드
    // - 문자셋을 utf-8로 변경

    for query in form_data {
        let decoded = parser::decode_input(&query, parser::EncodingType::Utf8);
        let form_processor = DataProcessor::new(Box::new(FormURLEncodedParser));
        let result = form_processor.process(decoded.as_bytes());
        println!("{}", result);
    }
    // FormURLEncodedParser를 사용하는 DataProcessor 인스턴스 생성
}

#[cfg(test)]
mod tests {
    use super::*; // 상위 모듈에서 정의된 함수나 구조체를 사용하기 위해

    #[test]
    fn test_euc_kr_form_url_encoded_to_json() {
        // 가정: `decode_euc_kr_and_parse_form_url_encoded` 함수는
        // EUC-KR로 인코딩된 form-url-encoded 문자열을 입력받아
        // JSON 문자열로 변환하는 함수입니다.
        let input = "name=%C8%AB%B1%E6%B5%BF&age=30"; // EUC-KR로 인코딩된 "홍길동"과 나이
        let expected_output = r#"{"name":"홍길동","age":"30"}"#;
        let decoded = parser::decode_input(input.as_bytes(), parser::EncodingType::EucKr);
        let form_processor = DataProcessor::new(Box::new(FormURLEncodedParser));
        let actual_output = form_processor.process(decoded.as_bytes());

        assert_eq!(
            serde_json::to_string(&actual_output).unwrap(),
            expected_output
        );
    }

    #[test]
    fn test_escape_sequences_json() {
        // `\x22`를 포함하는 JSON 문자열 처리
        let input = r"{\x22quote\x22:\x22This is a quote.\x22}";
        let expected_output = r#"{"quote":"This is a quote."}"#;
        let json_processor = DataProcessor::new(Box::new(JsonParser));
        let actual_output = json_processor.process(input.as_bytes());

        assert_eq!(
            serde_json::to_string(&actual_output).unwrap(),
            expected_output
        );
    }

    #[test]
    fn test_query_string() {
        // 가정: `parse_query_and_body_to_json` 함수는 쿼리 문자열과
        // HTTP 바디를 JSON 형식으로 변환합니다.
        let query = "search=rust%20programming";
        let expected_json = r#"{"search":"rust programming"}"#;
        let decoded = parser::decode_input(query.as_bytes(), parser::EncodingType::EucKr);
        let form_processor = DataProcessor::new(Box::new(FormURLEncodedParser));
        let actual_output = form_processor.process(decoded.as_bytes());
        assert_eq!(
            serde_json::to_string(&actual_output).unwrap(),
            expected_json
        );
    }

    #[test]
    fn test_complex_form_url_encoded() {
        // 가정: `parse_query_and_body_to_json` 함수는 쿼리 문자열과
        // HTTP 바디를 JSON 형식으로 변환합니다.
        let query = "first=value&arr[]=foo+bar&arr[]=baz&foo[bar]=foobar&test.field=testing";
        let expected_json = r#"{"first":"value","arr":["foo bar","baz"],"foo":{"bar":"foobar"},"test.field":"testing"}"#;
        let decoded = parser::decode_input(query.as_bytes(), parser::EncodingType::Utf8);
        let form_processor = DataProcessor::new(Box::new(FormURLEncodedParser));
        let actual_output = form_processor.process(decoded.as_bytes());
        assert_eq!(
            serde_json::to_string(&actual_output).unwrap(),
            expected_json
        );
    }

    #[test]
    fn test_url_encoded_escape_sequence() {
        // 가정: `parse_query_and_body_to_json` 함수는 쿼리 문자열과
        // HTTP 바디를 JSON 형식으로 변환합니다.
        let query = r"alias=\xEA\xB4\x91\xEC\xA3\xBC\xEC\xB9\xB4\xEB\x93\x9C";
        let expected_json = r#"{"alias":"광주카드"}"#;
        let decoded = parser::decode_input(query.as_bytes(), parser::EncodingType::Utf8);
        let form_processor = DataProcessor::new(Box::new(FormURLEncodedParser));
        let actual_output = form_processor.process(decoded.as_bytes());
        assert_eq!(
            serde_json::to_string(&actual_output).unwrap(),
            expected_json
        );
    }
}
