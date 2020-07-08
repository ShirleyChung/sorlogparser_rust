use std::collections::HashMap;
use std::collections::LinkedList;
use std::fmt;
use chrono::prelude::*;

// 每一筆資料由 string array組成每一個欄位，原資料ReqOrd, 以及相關的log
pub struct Rec {
	reqs_vec: Vec<String>,
	line    : String,
	log     : String,
}

impl Rec {
	pub fn print(&self) {
		if self.reqs_vec.len() > 3 {
			let ts_toks : Vec<String> = self.reqs_vec[3].split('.').map(|s| s.to_string()).collect();
			if ts_toks.len() > 1 {
				let u_secs = ts_toks[0].parse::<i64>().unwrap();
				let u_ms   = ts_toks[1].parse::<i64>().unwrap();
				let datetime: DateTime<Local> = Local.timestamp(u_secs, 0);
				let newdate = datetime.format("%Y-%m-%d %H:%M:%S");
				if &self.reqs_vec[0] == "Req" {
					println!("");
				}
    			println!("{} {}", newdate, u_ms);
			}
		}
		println!("{}\n{}", self.line, self.log);
	}
}

pub struct TableRec {
	index: HashMap<String, usize>,
	recs : Vec<String>,
}

impl TableRec {
	pub fn new() -> TableRec {
		TableRec {
			index: HashMap::<String, usize>::new(),
			recs : Vec::<String>::new(),
		}
	}
	pub fn print(&self) {
		for rec in &self.recs {
			print!("{} ", rec);
		}
		println!("");
	}
}

type ReqRecMap   = HashMap<String, Rec>;            // ReqKey-Rec
type OrdRecMap   = HashMap<String, LinkedList<Rec>>;// OrdKey-Rec

pub struct OrderRec {
	tables : HashMap<String, TableRec>, // table_name-table fields
	reqs   : ReqRecMap,
	ords   : OrdRecMap,
	req2ord: HashMap<String, String>,   // req對應到的ord
}

// 3. ReqOrd資料的輸入與輸出
impl OrderRec {
	pub fn new() -> OrderRec {
		OrderRec {
			tables: HashMap::<String, TableRec>::new(), // 表格名-欄位-index
			reqs  : ReqRecMap::new(),                   // reqKey-一筆Req
			ords  : OrdRecMap::new(),                   // ordKey-一筆Ord
			req2ord: HashMap::<String, String>::new(),
		}
	}
	pub fn insert_rec(&mut self, toks: Vec<String>, line: &str, log: &str) -> (&'static str, String) {
		let key = &toks[1];
		let hdr = &toks[0];	
		if key == "-" { // 沒有key值的，是表格
			let table_name = &toks[2];
			let mut tabrec = self.tables.entry(table_name.to_string()).or_insert(TableRec::new());
			for (idx, name) in toks.iter().enumerate() { // 插入每個Provider的Field
				tabrec.index.insert(name.to_string(), idx);
			}
			tabrec.recs = toks.to_vec();
		}
		else if "Req" == hdr {  // 依key將記錄儲存到hashmap中
			self.reqs.insert(key.to_string(), Rec{reqs_vec: toks.to_vec(), line: line.to_string(), log: log.to_string()});
			return ("Req", key.to_string())
		}
		else if "Ord" == hdr {	
			let rec = Rec{reqs_vec: toks.to_vec(), line: line.to_string(), log: log.to_string()};
			self.ords.entry(key.to_string()).or_insert(LinkedList::<Rec>::new()).push_back(rec);
			// 檢查Req-Ord對應是否有覆蓋的情況
			let reqkey = &toks[4];
			match self.req2ord.get(reqkey) {
				Some(ordkey) => {
					if ordkey != key {
						println!("There is MISS-MAPPING req-ord: req:{} ord:{}", reqkey, ordkey);
					}
				},
				_ => (),
			}
			self.req2ord.insert(reqkey.to_string(), key.to_string());
			return ("Ord", key.to_string())
		}
		else {
			//println!("unknow toks");
		}
		("", "".to_string())
	}
	pub fn print_ord(&self, key: &str) {
		match self.ords.get(key) {
			Some(list) => {
				let mut reqord_list = LinkedList::<&Rec>::new();
				let mut reqkey: String = String::from("");
				for ord in list {
					let ord_reqkey = &ord.reqs_vec[4];
					if &reqkey != ord_reqkey {
						match self.reqs.get(ord_reqkey) {
							Some(rec) => reqord_list.push_back(rec),
							_=> println!("req {} not found", ord_reqkey),
						}
						reqkey = ord_reqkey.to_string();
					};
					reqord_list.push_back(ord);
				}
				println!("--== ordkey:{} ==--", key);
				for rec in reqord_list {
					rec.print();
				}
			},
			_=> println!("order {} not exist", key),
		};
	}
	/// 以index, 找出ords中相等於target的rec
	pub fn find_req(&self, table_name: &str, key_index: usize, target: &str) {
		println!("find req {}, index={} ords:{}", target, key_index, self.ords.len());
		let mut found = false;
		for (key, rec) in &self.reqs  {
			if  rec.reqs_vec.len() < 3 || rec.reqs_vec[2] != table_name {
				continue;
			}
			if rec.reqs_vec.len() > key_index {
				if rec.reqs_vec[key_index] == target.to_string() {
					self.print_ord(&self.req2ord[key]);
					found = true;
				}
			} else {
				println!("line:{}, \n fields miss match.", rec.line);
			}
		};
		if !found {
			println!("{} not found", target);
		}
	}
	/// 以index, 找出reqs中相等於target的rec
	pub fn find_ord(&self, table_name: &str, key_index: usize, target: &str) {
		let mut found = false;
		for (key, list) in &self.ords {
			match list.back() {
				Some(rec) => {
					if  rec.reqs_vec.len() < 3 || rec.reqs_vec[2] != table_name {
						continue;
					}
					if rec.reqs_vec.len() > key_index {
						if rec.reqs_vec[key_index] == target.to_string() {
							self.print_ord(&rec.reqs_vec[1]);
							found = true;
						}
					} else {
						println!("line:{}, \n fields miss match.", rec.line);
					}
				},
				_=>println!("{} is empty", key),
			}
		}
		if !found {
			println!("{} not found", target);
		}
	}

	pub fn check_req_data(&self, table_name: &str, field_name: &str, search_target: &str) {
		println!("checking {}, {}", field_name, search_target);
		self.tables[table_name].print();
		match self.tables.get(table_name) {
			Some(tabrec) => { 
				match tabrec.index.get(field_name) {
					Some(idx) => {
						if tabrec.recs[0] == "Req" {
							self.find_req(table_name, *idx, search_target);
						}
						else if tabrec.recs[0] == "Ord" {
							self.find_ord(table_name, *idx, search_target);							
						}
						else {
							println!("cannot find {}, {}", field_name, search_target);
						}
					},
					_=> println!("field {} not found", field_name),
				}
			},
			_=> println!("{} doesn't exist", field_name),
		}	
	}
}

// 4. 解析管理
pub struct Parser {
	ord_rec : OrderRec,
	prevkey : (&'static str, String)
}

/// 陣列的操作函式
impl Parser {
	pub fn new()->Parser {
		Parser{ 
			ord_rec: OrderRec::new(),
			prevkey: ("", "".to_string()),
		}
	}

	/// 解析每一行的內容, 並儲存到HashMap
	pub fn parse_line(&mut self, line: &str, log: &str) {
		let toks : Vec<String> = line.to_string().split('\x01').map(|s| s.to_string()).collect();

		if toks.len() > 3 {
			self.prevkey = self.ord_rec.insert_rec(toks, line, log);
		} else {
			//println!("log line: {}", line);
		}
	}
	
	/// 輸入 表名/欄位名/值 來尋找目標
	pub fn find_by_field(&mut self, table_name: &str, field_name: &str, search_target: &str) {
		// 先找看看 Req表
		self.ord_rec.check_req_data(table_name, field_name, search_target);

	}
}

/// 使Parse類別能以println列印出來
impl fmt::Display for Parser {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "tables: {} reqs: {} ords:{}", 
			self.ord_rec.tables.len(), self.ord_rec.reqs.len(), self.ord_rec.ords.len())
	}
}