use std::collections::HashMap;
use std::collections::LinkedList;
use std::fmt;
use chrono::prelude::*;
use std::fs::File;
//use std::io::prelude::*;
//use chrono::{Utc, DateTime};

pub struct Rec {
	time: String,
	rptline: String,
}
impl Rec {
	#[allow(dead_code)]
	pub fn new()-> Rec {
		Rec {
			time: "".to_string(),
			rptline: "".to_string(),
		}
	}	
}
// 4. 解析管理
pub struct RptParser {
	recs: HashMap<String, Rec>
}

/// 陣列的操作函式
impl RptParser {
	#[allow(dead_code)]
	pub fn new()->RptParser {
		RptParser {
			recs: HashMap::<String, Rec>::new()
		}
	}

	///取得統計資訊
	#[allow(dead_code)]
	pub fn get_info(&mut self) -> &str {
		"none"
	}

	/// 解析每一行的內容, 並儲存到HashMap
	#[allow(dead_code)]
	pub fn parse_line(&mut self, line: &str, log: &str) {

	}

	/// 從輸入中解析出所有條件
	#[allow(dead_code)]
	pub fn find_by_conditions(&mut self, condstr: &str, savefile: &str, hide: &bool) {

	}

	/// 把list of list 存到檔案
	#[allow(dead_code)]
	pub fn save_to_file(&self, list_of_list: &LinkedList<LinkedList<&Rec>>, savefile: &str) {
		if let Ok(mut buff) = File::create(savefile) {
		}
	}
	
	/// 輸入 表名/欄位名/值 來尋找目標
	#[allow(dead_code)]
	pub fn find_by_field(&mut self, table_name: &str, field_name: &str, search_target: &str) {

	}
}

/// 使Parse類別能以println列印出來
impl fmt::Display for RptParser {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "all orders:  \nfails:  deals:  \ndeal prior to order:  \n")
	}
}