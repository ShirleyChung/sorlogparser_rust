use structopt::StructOpt;
use std::io::*;
use std::io::{BufReader};
//use std::io::prelude::*;
use std::fs::File;

mod parser;
use crate::parser::*;

mod fileread;
use crate::fileread::*;

/// SorReqOrd Parser
/// 可從檔案中, 取得與指定欄位值相符的記錄

// 1.參數取得
#[derive(StructOpt)]
struct Options {
	/// 要解析的SorReqOrd.log
	filepath: String, // Log檔路徑
	/// 指定TableName:FieldName:SearchValue; 例如 -f TwsNew:SorRID:100001
	#[structopt(short="f", long="field", default_value = "")]
	field   : String,
	/// SorReqOrd.log 檔案編碼格式, 預設BIG5
	#[structopt(short="e", long="encoding", default_value = "BIG5")]
	encoding: String,
}

/// 第一參數指定檔案
/// 將其讀入陣列以便解析
fn main() -> Result<()> {
	let options    = Options::from_args();

	let f          = File::open(options.filepath)?;
	let mut reader = BufReader::new(f);
	let mut parser = Parser::new();

	// 依每行解析
	read_data_log(&mut reader, &mut parser, &options.encoding);

	// 解析完了, 顯示解析結果
	println!("-=summary=-\n{}", parser.get_info());
	
	// 搜尋指定的目標
	if !options.field.is_empty() {
		parser.find_by_conditions(&options.field);
	}
	
	Ok(())
}

