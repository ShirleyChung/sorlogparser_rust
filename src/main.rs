use structopt::StructOpt;
use std::io::*;
use std::io::{BufReader, Write};
use std::fs::{File, self, OpenOptions};
use std::path::Path;
use chrono::Local;

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
	/// scan date-named directories and parse SorReqOrd.log files
	#[structopt(short="d", long="dir", default_value = ".")]
	scan_dir: String,
	/// output to PKILog-{date}.log file
	#[structopt(long="pki")]
	pki_output: bool,
}

/// 檢查目錄名是否為日期格式 (8位數字)
fn is_date_directory(name: &str) -> bool {
	name.len() == 8 && name.chars().all(|c| c.is_numeric())
}

/// 取得指定目錄下所有日期格式命名的子目錄
fn find_date_directories(dir_path: &str) -> Result<Vec<String>> {
	let mut date_dirs = Vec::new();
	let entries = fs::read_dir(dir_path)?;
	
	for entry in entries {
		let entry = entry?;
		let path = entry.path();
		if path.is_dir() {
			if let Some(name) = path.file_name() {
				if let Some(name_str) = name.to_str() {
					if is_date_directory(name_str) {
						date_dirs.push(path.to_string_lossy().to_string());
					}
				}
			}
		}
	}
	
	date_dirs.sort();
	Ok(date_dirs)
}

/// 處理單個SorReqOrd.log檔案
fn process_log_file(filepath: &str, encoding: &str, pki_mode: bool, search_field: &str) -> Result<String> {
	let mut output = String::new();
	
	if let Ok(f) = File::open(filepath) {
		let mut reader = BufReader::new(f);
		let mut parser = Parser::new();
		
		read_data_log(&mut reader, &mut parser, encoding);
		
		if pki_mode {
			// PKI 模式：執行搜尋或輸出所有記錄
			if !search_field.is_empty() {
				// 執行搜尋，find_by_conditions 會自動輸出到檔案
				parser.find_by_conditions(search_field, "", &true, true, true);
				// 搜尋已由 find_by_conditions 處理，此處返回空
				return Ok(String::new());
			} else {
				// 沒有搜尋條件，返回所有記錄的 PKI 格式供上層累積
				output = parser.get_pki_output();
			}
			// Parser 會在此方法結束後自動釋放，每個檔案都用新的 Parser
		} else {
			// 普通模式：輸出詳細資訊
			output.push_str("=== ");
			output.push_str(filepath);
			output.push_str(" ===\n");
			output.push_str(parser.get_info());
			output.push_str("\n");
			
			let unlinkreqs_info = parser.list_unlink_req();
			if !unlinkreqs_info.is_empty() {
				output.push_str("there are unlink reqs:\n");
				output.push_str(&unlinkreqs_info);
				output.push_str("\n");
			}
			output.push_str("\n");
		}
	} else {
		output.push_str(&format!("error opening {}\n\n", filepath));
	}
	
	Ok(output)
}

/// 掃描日期目錄並解析所有SorReqOrd.log
fn scan_and_parse_date_dirs(base_dir: &str, encoding: &str, use_pki: bool, search_field: &str) -> Result<()> {
	let date_dirs = match find_date_directories(base_dir) {
		Ok(dirs) => dirs,
		Err(e) => {
			println!("Error reading directory {}: {}", base_dir, e);
			return Ok(());
		}
	};
	
	if date_dirs.is_empty() {
		println!("No date-named directories found in {}", base_dir);
		return Ok(());
	}
	
	let mut pki_file = if use_pki {
		let now = Local::now();
		let date_str = now.format("%Y%m%d").to_string();
		let output_file = format!("PKILog-{}.log", date_str);
		match OpenOptions::new()
			.create(true)
			.append(true)
			.open(&output_file) {
			Ok(file) => {
				println!("Creating PKI output file: {}", output_file);
				Some(file)
			},
			Err(e) => {
				println!("Error creating {}: {}", output_file, e);
				None
			}
		}
	} else {
		None
	};
	
	let mut found_logs = false;
	
	for dir in date_dirs {
		let log_path = Path::new(&dir).join("SorReqOrd.log");
		let log_file = log_path.to_string_lossy().to_string();
		
		if log_path.exists() {
			found_logs = true;
			println!("Processing: {}", log_file);
			match process_log_file(&log_file, encoding, use_pki, search_field) {
				Ok(output) => {
					if use_pki {
						// 直接寫入檔案，然後釋放記憶體
						if let Some(ref mut file) = pki_file {
							let _ = file.write_all(output.as_bytes());
						}
					} else {
						print!("{}", output);
					}
				},
				Err(e) => {
					let err_msg = format!("Error processing {}: {}\n\n", log_file, e);
					if use_pki {
						if let Some(ref mut file) = pki_file {
							let _ = file.write_all(err_msg.as_bytes());
						}
					} else {
						print!("{}", err_msg);
					}
				}
			}
		}
	}
	
	if !found_logs {
		println!("No SorReqOrd.log files found in date directories");
		return Ok(());
	}
	
	// PKI 模式的檔案已在迴圈中逐個寫入
	if use_pki {
		if pki_file.is_some() {
			let now = Local::now();
			let date_str = now.format("%Y%m%d").to_string();
			let output_file = format!("PKILog-{}.log", date_str);
			println!("PKI output saved to: {}", output_file);
		}
	}
	
	Ok(())
}

/// 第一參數指定檔案
/// 將其讀入陣列以便解析
fn main() -> Result<()> {
	let mut options = Options::from_args();

    if options.gui {
        gui::run();
        return Ok(());
    }

	// 若沒有任何輸入參數，設定預設值：目錄掃描 + 搜尋條件 + PKI 輸出
	if options.filepath.is_none() && options.field.is_empty() && !options.pki_output && !options.save && !options.show_flow && options.table_field.is_empty() {
		// 設定預設值
		options.field = "TwfNew:SesName:SorAPI|TwfChg:SesName:SorAPI|FrfNew:SesName:SorAPI|FrfChg:SesName:SorAPI".to_string();
		options.pki_output = true;
	}

	// 若未指定檔案參數，則掃描日期目錄
	if options.filepath.is_none() {
		return scan_and_parse_date_dirs(&options.scan_dir, &options.encoding, options.pki_output, &options.field);
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
				parser.find_by_conditions(&options.field, &savepath, &options.hide, options.pki_output, false);
			}

			// 若沒有搜尋條件但指定 --pki 時，輸出所有記錄的 PKI 格式到檔案
		if options.pki_output && options.field.is_empty() {
			let pki_output = parser.get_pki_output();
			if !pki_output.is_empty() {
				let now = chrono::Local::now();
				let filename = format!("PKILog-{}.log", now.format("%Y%m%d"));
				if let Ok(mut file) = OpenOptions::new()
					.create(true)
					.append(true)
					.open(&filename) {
					let _ = file.write_all(pki_output.as_bytes());
					println!("PKI output saved to: {}", filename);
				}
			}
			return Ok(());
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
	}

	Ok(())
}
