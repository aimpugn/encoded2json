use encoding_rs::Encoding;
use percent_encoding::percent_decode;
use std::error::Error;

pub fn decode_input(input: &str, encoding: &'static Encoding) -> Result<String, Box<dyn Error>> {
    let percent_decoded = percent_decode(input.as_bytes()).collect::<Vec<u8>>();

    let (cow, _, had_errors) = encoding.decode(&percent_decoded);

    if had_errors {
        Err("EUC-KR decoding error".into())
    } else {
        Ok(decode_escape_sequences(&cow))
    }
}

// `\xNN` 형식의 이스케이프 시퀀스를 원래의 문자나 데이터로 변환하는 디코딩 함수
fn decode_escape_sequences(input: &str) -> String {
    // 결과 문자열을 위한 공간을 미리 할당하여 재할당 횟수를 줄입니다.
    // 예: input = "hello\x41\x42world" (len = 15)
    // result의 초기 용량 = 15
    let mut result = String::with_capacity(input.len());

    // 유효한 UTF-8 바이트 시퀀스를 저장할 벡터
    // 예: "\x41\x42" 처리 시 [65, 66]을 저장
    let mut byte_seq = Vec::new();

    // 입력 문자열을 문자 단위로 순회할 반복자
    // peekable()을 사용하여 다음 문자를 미리 볼 수 있게 함
    let mut chars = input.chars().peekable();

    // 입력 문자열의 모든 문자를 순회
    while let Some(ch) = chars.next() {
        // 16진수 이스케이프 시퀀스 시작 여부를 확인
        // 예: ch = '\', next char = 'x'. 즉 `\xNN` 형식인지 여부 판단합니다.
        if ch == '\\' && chars.peek() == Some(&'x') {
            chars.next(); // 'x' 문자 건너뛰기
                          // 다음 두 문자를 한 번에 가져와 처리
                          // 예: "\x41" -> h1 = '4', h2 = '1'
            if let (Some(h1), Some(h2)) = (chars.next(), chars.next()) {
                let hex_chars = format!("{}{}", h1, h2);
                // 16진수 문자열을 u8로 변환 시도
                // 예: "41" -> Ok(65)
                if let Ok(byte) = u8::from_str_radix(&hex_chars, 16) {
                    // 유효한 16진수 시퀀스인 경우 바이트 시퀀스에 추가
                    // 예: byte_seq.push(65)
                    byte_seq.push(byte);
                    continue; // 다음 문자로 넘어감
                }
                // 유효하지 않은 16진수 시퀀스인 경우 원래 문자열 그대로 추가
                // 예: "\xGG" -> result에 "\\xGG" 추가
                result.push_str("\\x");
                result.push_str(&hex_chars);
            } else {
                // 불완전한 이스케이프 시퀀스 처리
                // 예: 입력의 끝에 "\x"가 있는 경우
                result.push_str("\\x");
            }
        } else {
            // 이스케이프 시퀀스가 아닌 경우
            if !byte_seq.is_empty() {
                // 누적된 바이트 시퀀스가 있으면 UTF-8로 디코딩하여 결과에 추가
                // 예: byte_seq = [65, 66] -> result에 "AB" 추가
                result.push_str(&String::from_utf8_lossy(&byte_seq));
                byte_seq.clear(); // 바이트 시퀀스 초기화
            }
            // 일반 문자는 그대로 결과에 추가
            // 예: ch = 'a' -> result에 'a' 추가
            result.push(ch);
        }
    }

    // 남은 바이트 시퀀스 처리
    // 예: 입력의 끝에 "\x41\x42"가 있었다면 여기서 "AB"가 result에 추가됨
    if !byte_seq.is_empty() {
        result.push_str(&String::from_utf8_lossy(&byte_seq));
    }

    // 최종 결과 반환
    result
}

// 테스트 코드
#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_decode_input_iso_8859_1() {
        let input = "Caf%E9"; // "Café" in ISO-8859-1
        let result = decode_input(input, encoding_rs::WINDOWS_1252).unwrap();
        assert_eq!(result, "Café");
    }

    #[test]
    fn test_decode_input_with_query_params() {
        let input = "key=%BE%C8%B3%E7&arr[]=value2&somemap[key3]=value3";
        let result = decode_input(input, encoding_rs::EUC_KR).unwrap();
        assert_eq!(result, "key=안녕&arr[]=value2&somemap[key3]=value3");
    }
}

// percent_decode 함수가 어떻게 동작하는지 확인하기 위한 테스트 코드입니다.
#[cfg(test)]
mod test_percent_decode {
    use super::*;

    #[test]
    fn test_default_ascii() {
        // 테스트 1: 기본 ASCII 문자
        let decoded = percent_decode("Hello%20World".as_bytes())
            .decode_utf8()
            .unwrap();
        assert_eq!(decoded, "Hello World");
    }

    #[test]
    fn test_special_characters() {
        // 테스트 2: 특수 문자
        let decoded = percent_decode("Hello%21%40%23%24%25%5E%26%2A%28%29".as_bytes())
            .decode_utf8()
            .unwrap();
        assert_eq!(decoded, "Hello!@#$%^&*()");
    }

    #[test]
    fn test_unicode() {
        // 테스트 3: 유니코드 문자
        let decoded = percent_decode("%E3%81%93%E3%82%93%E3%81%AB%E3%81%A1%E3%81%AF".as_bytes())
            .decode_utf8()
            .unwrap();
        assert_eq!(decoded, "こんにちは");
    }

    #[test]
    fn test_mixed_case_unicode() {
        // 테스트 4: 인코딩되지 않은 문자 포함
        let decoded = percent_decode("Hello%20World%21%20This%20is%20a%20test.".as_bytes())
            .decode_utf8()
            .unwrap();
        assert_eq!(decoded, "Hello World! This is a test.");
    }

    #[test]
    fn test_percent_sign_itself() {
        // 테스트 5: 퍼센트 기호 자체
        let decoded = percent_decode("50%25%20off".as_bytes())
            .decode_utf8()
            .unwrap();
        assert_eq!(decoded, "50% off");
    }

    #[test]
    fn test_invalid_encoding() {
        // 테스트 6: 잘못된 인코딩 처리
        let decoded = percent_decode("Invalid%2".as_bytes())
            .decode_utf8()
            .unwrap();
        assert_eq!(decoded, "Invalid%2");
    }

    #[test]
    fn test_empty_string() {
        // 테스트 7: 빈 문자열
        let decoded = percent_decode("".as_bytes()).decode_utf8().unwrap();
        assert_eq!(decoded, "");
    }
}
