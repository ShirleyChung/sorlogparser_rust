use structopt::StructOpt;
use std::io::*;
use std::io::{BufReader};
//use std::io::prelude::*;
use std::fs::File;

mod parser;
use crate::parser::*;

mod fileread;
use crate::fileread::*;

pub mod gui;

/// SorReqOrd Parser
/// Retrieve record of specified fields from given SorReqOrd.log

// 1.Parameters parsing
#[derive(StructOpt)]
struct Options {
	/// Target SorReqOrd.log
	filepath: Option<String>, // Log file path
//
//	#[structopt(short="m", long="mllogfile", default_value = "MLStkRpt.log")]
//	mlrptlog : String,
	/// Please specify TableName:FieldName:SearchValue; ex: -f TwsNew:SorRID:100001, using "," to connect multiple conditions
	#[structopt(short="f", long="field", default_value = "")]
	field   : String,
	/// SorReqOrd.log encoding type
	#[structopt(short="e", long="encoding", default_value = "BIG5")]
	encoding: String,
	/// save the output contents
	#[structopt(short="s", long="save")]
	save: bool,
	/// do not print the result list
	#[structopt(short="h", long="hide")]
	hide: bool,
	/// path of the saving file
	#[structopt(short="o", long="output", default_value = "")]
	savepath: String,
	/// statistic a field's values
	#[structopt(short="t", long="statistic", default_value = "")]
	table_field: String,
	/// flow in seconds
	#[structopt(short="w", long="flow")]
	show_flow: bool,
    /// Launch the GUI
    #[structopt(long)]
    gui: bool,
}

/// 第一參數指定檔案
/// 將其讀入陣列以便解析
fn main() -> Result<()> {
	let options = Options::from_args();

    if options.gui {
        gui::run();
        return Ok(());
    }

	// 解析SorReqOrd.log
	if let Some(filepath) = options.filepath {
		if let Ok(f) = File::open(&filepath) {
			let mut reader = BufReader::new(f);
			let mut parser = Parser::new();

			// 依每行解析
			read_data_log(&mut reader, &mut parser, &options.encoding);

			// 解析完了, 顯示解析結果
			println!("-=summary=-\n{}", parser.get_info());

			let unlinkreqs_info = parser.list_unlink_req();
			if !unlinkreqs_info.is_empty() {
				println!("there are unlink reqs:\n{}", unlinkreqs_info);
			}

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

			// 統計某個欄位
			if !options.table_field.is_empty() {
				let params: Vec<&str> = options.table_field.split(':').collect();
				if params.len() > 1 {
					println!("{}", parser.statistic_field(params[0], params[1]));
				} else {
					println!("please correct -s format.  eg.: -s TwfNew:user");
				}
			}

			// 顯示每秒流量
			if options.show_flow {
				println!("{}", parser.req_flow_statistic());
			}

		} else {
			println!("error opening {}", filepath);
		}
	// 解析MLStkRpt.log
	}
/*                else if !options.mlrptlog.is_empty() {
		if let Ok(f) = File::open(&options.mlrptlog) {
			let mut reader = BufReader::new(f);
			let mut rptparer = RptParser::new();

			// 依每行解析
			println!("parsing {}", options.mlrptlog);
			read_rpt_log(&mut reader, &mut rptparer, &options.encoding);
		} else {
			println!("error opening {}", options.mlrptlog);
		}
	} */
                else {
		println!("Need SorReqOrd.log");
	}

	Ok(())
}
