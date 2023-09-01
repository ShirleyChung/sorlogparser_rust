use std::io::*;
use std::io::{BufRead, BufReader};
use encoding::{Encoding, DecoderTrap};
use encoding::all::{ BIG5_2003, GB18030, ISO_2022_JP };

use crate::parser::Parser;
//use crate::rpt_parser::RptParser;

pub enum LineType<T> {
	EndOfFile,
	Rec(T),
	Log(T),
	LogExt(T),
	Empty,
}

#[derive(PartialEq)]
enum EncodingType {
	BIG5,
	JP,
	GB,
	UTF8,
}

/// get line from reader
#[allow(dead_code)]
fn get_reader_line<R: Read>(reader: &mut BufReader<R>, encoding: &EncodingType) -> LineType<String> {
	let mut line_buf = Vec::<u8>::new();
	let mut line = String::new();
	// 讀第一行
	line_buf.clear();
	match reader.read_until(b'\n', &mut line_buf) {
		Ok(sz_line) => {
			if sz_line == 0 {
				return LineType::EndOfFile;
			}
			let mut dont_need_utf8 = true;
			if encoding != &EncodingType::UTF8 {
				dont_need_utf8 = match encoding {
					EncodingType::BIG5 => BIG5_2003.decode_to(&mut line_buf, DecoderTrap::Strict, &mut line).is_ok(),
					EncodingType::JP => ISO_2022_JP.decode_to(&mut line_buf, DecoderTrap::Strict, &mut line).is_ok(),
					EncodingType::GB => GB18030.decode_to(&mut line_buf, DecoderTrap::Strict, &mut line).is_ok(),
					_ => false,
				};
			}
			if !dont_need_utf8 {
				line = String::from_utf8_lossy(&line_buf).to_string();
			}
			line = line.trim().to_string();
			if sz_line < 2 || line.len() < 2 {
				return LineType::Empty;
			}
			if line.as_bytes()[0] == ':' as u8 {
				LineType::Log(line)
			} else if sz_line > 3 && &line[..3] != "Req" && &line[..3] != "Ord" {
				LineType::LogExt(line)
			} 
			else {
				LineType::Rec(line)
			}
		},
		Err(_)=> LineType::EndOfFile,
	}
}

fn get_encoding_constant(encoding_opt: &str) -> EncodingType {
	match encoding_opt {
		"BIG5" => EncodingType::BIG5,
		"GB" => EncodingType::GB,
		"JP" => EncodingType::JP,
		_  => { println!("decod use UTF8"); EncodingType::UTF8 },
	}
}

/// line by line with log 解析
pub fn read_data_log<R: Read>(reader: &mut BufReader<R>, parser: &mut Parser, encoding_opt: &str) {
	print!("parsing data...\n");
	let mut rec_tmp: String = "".to_string();
	let mut log_tmp: String = "".to_string();
	let encoding = get_encoding_constant(encoding_opt);
	loop {
		match get_reader_line(reader, &encoding) {
			// 先把讀到的記錄暫存起來，為要和log一起parse
			LineType::Rec(line) => {
				if !rec_tmp.is_empty() {
					parser.parse_line(&rec_tmp, &log_tmp);
					log_tmp.clear();
				}
				rec_tmp = line;
			},
			// log 和 ext log 串成一串，等待rec再一併被parse
			LineType::Log(log)    => log_tmp = log_tmp + &log,
			LineType::LogExt(log) => log_tmp = log_tmp + &log,
			LineType::Empty     =>  continue,
			LineType::EndOfFile =>  break,
		};
	};
	parser.parse_line(&rec_tmp, &log_tmp);
}

//回報LOG檔解析
/*  example:
111/12/20 08:30:04.110<r<1GA2h000100368522330  005100000000001S0020083004111000000000001020000 00079000437540              
111/12/20 08:30:04.111>s>ACK
111/12/20 08:30:04.111<r<2QA32000100245185263  001415000000001S0420083003944000000000001020000 02880000000  0              
111/12/20 08:30:04.112>s>ACK
*/
/*
pub enum RptType<T> {
	EndOfFile,
	Deal(T),
	Order(T),
	Empty,
}

fn read_rpt_line<R: Read>(reader: &mut BufReader<R>, encoding: &EncodingType) -> RptType<String> {
	let mut line_buf = Vec::<u8>::new();
	let mut line = String::new();
	// 讀第一行
	line_buf.clear();
	match reader.read_until(b'\n', &mut line_buf) {
		Ok(sz_line) => {
			if sz_line == 0 {
				return RptType::EndOfFile;
			}
			RptType::Empty
		},
		Err(_)=> RptType::EndOfFile,
	}
}
/// line by line with log 解析
pub fn read_rpt_log<R: Read>(reader: &mut BufReader<R>, parser: &mut RptParser, encoding_opt: &str) {
	let mut rec_tmp: String = "".to_string();
	let mut log_tmp: String = "".to_string();
	let encoding = get_encoding_constant(encoding_opt);
	loop {
		match read_rpt_line(reader, &encoding) {
			// 先把讀到的記錄暫存起來，為要和log一起parse
			RptType::Order(line) => {
			},
			RptType::Deal(line) => {
			},
			// log 和 ext log 串成一串，等待rec再一併被parse
			RptType::Empty     =>  continue,
			RptType::EndOfFile =>  break,
		};
	};
	parser.parse_line(&rec_tmp, &log_tmp);
}
*/