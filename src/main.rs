use clap::{ArgGroup, Parser};
use std::error::Error;

mod decoder;
mod parser;

use decoder::decode_input;
use parser::parse_to_json;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(group(
    ArgGroup::new("encoding")
        .required(false)
        .args(&["utf_8", "euc_kr"]),
))] // 두 옵션을 동시에 사용할 수 없게 합니다.
struct Args {
    /// Input string
    input: String,

    /// Treat <INPUT> string as UTF-8 encoded string
    #[arg(long)] // --utf-8와 같은 긴 형식의 플래그로 사용
    utf_8: bool,

    /// Treat <INPUT> string as EUC-KR encoded string
    #[arg(long)] // --euc-kr와 같은 긴 형식의 플래그로 사용
    euc_kr: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let encoding = if args.utf_8 {
        encoding_rs::UTF_8
    } else if args.euc_kr {
        encoding_rs::EUC_KR
    } else {
        encoding_rs::WINDOWS_1252
    };

    // let input = args.input;
    let decoded = decode_input(&args.input, encoding)?;
    let json = parse_to_json(&decoded)?;
    let json_string = serde_json::to_string(&json)?;

    println!("{}", json_string);

    Ok(())
}
