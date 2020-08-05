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
	/// 指定TableName:FieldName:SearchValue; 例如 -f TwsNew:SorRID:100001, 可指定多組做交集運算，以","為分隔符號
	#[structopt(short="f", long="field", default_value = "")]
	field   : String,
	/// SorReqOrd.log 檔案編碼格式, 預設BIG5
	#[structopt(short="e", long="encoding", default_value = "BIG5")]
	encoding: String,
	/// 輸出存檔
	#[structopt(short="s", long="save")]
	save: bool,	
	/// 不印出搜尋結果list
	#[structopt(short="h", long="hide")]
	hide: bool,		
	/// 選擇存檔路徑
	#[structopt(short="o", long="output", default_value = "")]
	savepath: String,
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
		let savepath = if options.save {
			if options.savepath.is_empty() {
				let mut tmp: String = options.field.chars().map(|x| match x {','=>'_', ':' => '_', _ => x}).collect();
				tmp.push_str(".log");
				tmp
			} else {
				options.savepath
			}
		} else {
			"".to_string()
		};
		parser.find_by_conditions(&options.field, &savepath, &options.hide);
	}
	
	Ok(())
}

