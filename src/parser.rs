use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::LinkedList;
use std::fmt;
use chrono::prelude::*;
use std::fs::File;
use std::io::prelude::*;
use chrono::LocalResult::Single;

// 每一筆資料由 string array組成每一個欄位，原資料ReqOrd, 以及相關的log
pub struct Rec {
	reqs_vec: Vec<String>,
	line    : String,
	log     : String,
	linked  : bool,
}

fn get_ordst(st: i32) -> String {
	match st {
		6 => "委託傳送中".to_string(),
		7 => "委託已傳送".to_string(),
		90=> "委託成功".to_string(),
		99 => "委託失敗".to_string(),
		101 => "交易所已接受".to_string(),
		110 => "部份成交".to_string(),
		111 => "全部成交".to_string(),
		120 => "交易所取消".to_string(),
		_ => "未知".to_string(),
	}
}

// Rec具有print的操作, 可將timestamp印出
impl Rec {
	pub fn is_req(&self) -> bool {
		&self.reqs_vec[0] == "Req"
	}
	pub fn get_timestamp(&self) -> String {
		let mut dt = String::new();
		if self.reqs_vec.len() > 3 {
			let ts_toks : Vec<String> = self.reqs_vec[3].split('.').map(|s| s.to_string()).collect();
			if ts_toks.len() > 1 {
				if let Ok(u_secs) = ts_toks[0].parse::<i64>() {
					if let Single(datetime) = Local.timestamp_opt(u_secs, 0) {
						dt = datetime.format("%Y%m%d%H%M%S.").to_string() + &ts_toks[1];
					}
				}
			}
		}
		dt
	}
	pub fn print(&self) {
		print!("{}", self.to_string());
	}
	pub fn to_string(&self) -> String {
		let mut ret = String::new();
		if self.reqs_vec.len() > 5 {
			if self.is_req() {
				let type_key = &self.reqs_vec[4][..];
				let ord_type: &str = match type_key
				{ "1" => "新單", "2" => "改量", "3" => "改價", "4" => "刪單", "10" =>"成交", _=> "" };
				ret = format!("{} ({})\n", self.get_timestamp(), ord_type);
			} 
			else {
				if let Ok(st) = self.reqs_vec[6].parse::<i32>() {
					ret = format!("{} =>{}\n", self.get_timestamp(), get_ordst(st));
				} else {
					ret = format!("{}\n", self.get_timestamp());
				}
			}
		}
		ret.push_str(&format!("{}\n{}\n", self.line, self.log));
		ret
	}
}

pub struct TableRec {
	pub index: HashMap<String, usize>,
	pub recs : Vec<String>,
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
	pub tables : HashMap<String, TableRec>, // table_name-table fields
	pub reqs   : ReqRecMap,
	pub ords   : OrdRecMap,
	req2ord: HashMap<String, String>,   // req對應到的ord
}

pub struct OrdInfo {
	rid   : String,
	ordno : String,
	status: String,
}

impl OrdInfo {
	pub fn new() -> OrdInfo {
		OrdInfo {
			rid   : String::new(),
			ordno : String::new(),
			status: String::new(),
		}
	}
	pub fn to_string(&self) -> String {
		String::from("\n===== 流水號:") + &self.rid + " 委託書號:" + &self.ordno + " 最後狀態:" + &self.status + " ====="
	}
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
			let tabrec = self.tables.entry(table_name.to_string()).or_insert(TableRec::new());
			for (idx, name) in toks.iter().enumerate() { // 插入每個Provider的Field
				tabrec.index.insert(name.to_string(), idx);
			}
			tabrec.recs = toks.to_vec();
		}
		else if "Req" == hdr {  // 依key將記錄儲存到hashmap中
			self.reqs.insert(key.to_string(), Rec{reqs_vec: toks.to_vec(), line: line.to_string(), log: log.to_string(), linked: false});
			return ("Req", key.to_string())
		}
		else if "Ord" == hdr {	
			let rec = Rec{reqs_vec: toks.to_vec(), line: line.to_string(), log: log.to_string(), linked: false};
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
			match self.reqs.get_mut(reqkey) {
				Some(req) => {
					req.linked = true;
				},
				_ => (),
			}
			return ("Ord", key.to_string())
		}
		else {
			//println!("unknow toks");
		}
		("", "".to_string())
	}
	/// 取得 指定Ord key 的 ReqOrd 的 LinkedList
	pub fn get_target_ordlist(&self, key: &str) -> LinkedList<&Rec> {
		let mut reqord_list = LinkedList::<&Rec>::new();
		match self.ords.get(key) {
			Some(list) => {
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
			},
			_=> (),
		};
		reqord_list
	}
	/// 取得該記錄中，指定欄位的值
	pub fn get_value(&self, rec: &Rec, field_name: &str) -> String {
		if rec.reqs_vec.len() > 2 {
			match self.tables.get(&rec.reqs_vec[2]) {
				Some(tabrec) => { 
					match tabrec.index.get(field_name) {
						Some(idx) => {
							return rec.reqs_vec[*idx].to_string();
						},
						_=> return String::new(),
					};
				},
				_=> return String::new(),
			};
		} else {
			return String::new();
		}
	}
	/// 統計某一欄位的數量: 例如TwfNew總共有多少個user
	pub fn statistic_field(&self, table_name: &str, field_name: &str) -> String {
		let mut field_set = HashSet::<String>::new();
		match self.tables.get(table_name) { // 先從tables中, 找到要的table(例如TwfNew)
			Some(tabrec) => {
				match tabrec.index.get(field_name) {  // 有找到table的話, 再定位出field的index(例如user可能是在4之類)
					Some(idx) => {
						for req in &self.reqs { // 借用結構的reqs成員避免move
							let rec = req.1;
							if rec.reqs_vec.len() > 2 {
								if rec.reqs_vec[2] == table_name {
									let val = rec.reqs_vec[*idx].to_string();
									if !val.is_empty() {
										field_set.insert( val );
									}
								}
							}
						}
						let mut ret= format!("there are totally {} {} of {}:\n", field_set.len(), field_name, table_name);
						for user in field_set {							
							ret.push_str(&user);
							ret.push_str("\n");
						}
						return ret;
					},
					_=> return format!("there is no {} field", field_name),
				};
			},
			_=> return format!("there is no {} table", table_name),
		};
	}
	/// 取得該筆LinkedList的彙總說明
	fn get_ord_summary(&self, list: &LinkedList<&Rec>) -> OrdInfo {
		let mut info = OrdInfo::new();
		let mut ordst :i32 = 0;
		let mut reqst :i32 = 0;
		//let mut reqst :i32 = 0;
		for rec in list {
			if rec.reqs_vec[0] == "Req" && rec.reqs_vec[4] == "1" { // 若是新單要求，則取流水號
				info.rid = self.get_value(rec, "SorRID");
				//reqst = rec.reqs_vec[6].parse::<i32>().unwrap();
			}
			if rec.reqs_vec[0] == "Ord" {
				info.ordno = self.get_value(rec, "OrdNo");
				if let Ok(st) = self.get_value(rec, "OrderSt").parse::<i32>() {
					if st > ordst {
						ordst = st;
						if let Ok(rst) = self.get_value(rec, "ReqStep").parse::<i32>() {
							reqst = rst;
						}
					}
				}
			}
		}
		info.status = get_ordst(reqst);
		info.status.push_str("/");
		info.status.push_str(&get_ordst(ordst));
		info
	}
	/// 檢查rec是否符合條件
	pub fn check_rec(&self, rec: &Rec, table_name: &str, key_index: usize, target: &str) -> Option<String> {
		if  rec.reqs_vec.len() < 3 || rec.reqs_vec[2] != table_name {
			return None;
		}
		if rec.reqs_vec.len() > key_index {
			if rec.reqs_vec[key_index] == target.to_string() {
				let key = &rec.reqs_vec[1];
				if rec.reqs_vec[0] == "Ord" {
					return Some(key.to_string());
				} else if rec.reqs_vec[0] == "Req" {
					return Some(self.req2ord[key].to_string());
				} else {
					return None;
				}
			}
		}
		None
	}
	/// 印出指定 Ord Key 的 彙總以及 所有Log; 每筆Log會有timestamp
	/*pub fn print_ord(&self, key: &str) {
		let list = &self.get_target_ordlist(key);
		println!("{}", self.get_ord_summary(list).to_string() );
		for rec in list {
			rec.print();
		}
	}*/
	/// 將ord list轉為字串
	pub fn ord_list_to_string(&self, list: &LinkedList<&Rec>) -> String {
		let mut list_str = String::new();
		list_str.push_str(&self.get_ord_summary(&list).to_string());
		list_str.push_str("\n");
		for rec in list {
			list_str.push_str(&rec.to_string());
		}
		list_str
	}
	/// 印出 Ord list 的 彙總以及 所有Log; 每筆Log會有timestamp
	pub fn print_ord_list(&self, list: &LinkedList<&Rec>) {
		println!("{}", self.get_ord_summary(&list).to_string() );
		for rec in list {
			rec.print();
		}
	}
	/// 從前一次的搜尋結果中, 以給定的條件再次搜尋
	#[allow(dead_code)]
	pub fn find_list(&self, list_of_list: LinkedList<LinkedList<&Rec>>, table_name: &str, field_name: &str, search_target: &str) -> Option<LinkedList<LinkedList<&Rec>>> {
		println!("checking {}, {}", field_name, search_target);
		let mut result_list = LinkedList::<LinkedList<&Rec>>::new();
		match self.tables.get(table_name) {
			Some(tabrec) => { // 有對應到指定的table
				match tabrec.index.get(field_name) {
					Some(idx) => {  // 有對應到指定的filed
						for list in list_of_list { // 從給定的list of list裡搜尋每一筆list
							for rec in list {       // 比對list裡的每一筆 rec
								if let Some(key) = self.check_rec(rec, table_name, *idx, search_target) {
									result_list.push_back(self.get_target_ordlist(&key)); // 有找到的話存進結果裡
									break;
								}
							}
						}
					},
					_=> println!("field {} not found", field_name),
				}
			},
			_=> println!("{} doesn't exist", table_name),
		};
		Some(result_list)
	}
	/// 以index, 找出ords中相等於target的rec
	pub fn find_req(&self, table_name: &str, key_index: usize, target: &str) -> LinkedList<LinkedList<&Rec>> {
		let mut found = false;
		let mut list_of_list = LinkedList::<LinkedList<&Rec>>::new();
		for (_, rec) in &self.reqs  {
			match self.check_rec(&rec, table_name, key_index, target)
			{
				Some(key) => { 
					//self.print_ord(&key);
					list_of_list.push_back(self.get_target_ordlist(&key));
					found = true; 
				},
				None      => continue,
			}
		};
		if !found {
			println!("{} not found", target);
		}
		list_of_list
	}
	/// 以index, 找出reqs中相等於target的rec
	pub fn find_ord(&self, table_name: &str, key_index: usize, target: &str) -> LinkedList<LinkedList<&Rec>> {
		let mut found = false;
		let mut list_of_list = LinkedList::<LinkedList<&Rec>>::new();
		for (key, list) in &self.ords {
			match list.back() {
				Some(rec) => {
					match self.check_rec(&rec, table_name, key_index, target)
					{
						Some(key) => { 
							//self.print_ord(&key);
							list_of_list.push_back(self.get_target_ordlist(&key));
							found = true;
						},
						None      => continue,
					}
				},
				_=>println!("{} is empty", key),
			}
		}
		if !found {
			println!("{} not found", target);
		}
		list_of_list
	}

	pub fn check_req_data(&self, table_name: &str, field_name: &str, search_target: &str, hide: &bool) -> Option<LinkedList<LinkedList<&Rec>>> {
		println!("checking {}, {}", field_name, search_target);
		if !hide {
			for (_, tab) in &self.tables {
				tab.print();
			}
		}
		match self.tables.get(table_name) {
			Some(tabrec) => { 
				match tabrec.index.get(field_name) {
					Some(idx) => {
						if tabrec.recs[0] == "Req" {
							return Some(self.find_req(table_name, *idx, search_target));
						}
						else if tabrec.recs[0] == "Ord" {
							return Some(self.find_ord(table_name, *idx, search_target));
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
		None	
	}
}

// 4. 解析管理
pub struct Parser {
	pub ord_rec : OrderRec,
	info    : String,
	prevkey : (&'static str, String)
}
/*
pub struct Conditions {
	table: String,
	field: String,
	value: String,
}
*/
/// 陣列的操作函式
impl Parser {
	pub fn new()->Parser {
		Parser{ 
			ord_rec: OrderRec::new(),
			info   : String::new(),
			prevkey: ("", "".to_string()),
		}
	}

	///取得統計資訊
	pub fn get_info(&mut self) -> &str {
		if self.info.is_empty() {
			let mut deals = 0;
			let mut fails = 0;
			// 掃描req列表，統計
			for (_, req) in &self.ord_rec.reqs {
				if req.reqs_vec[4] == "10" || req.reqs_vec[4] == "11" {
					deals = deals + 1;
				}
			}
			// 掃描req列表，統計
			for (_, ord) in &self.ord_rec.ords {
				if let Some(rec) = ord.back() {
					if rec.reqs_vec.len() > 7 && rec.reqs_vec[7] == "99" {
						fails = fails + 1;
					}
				}
			}
			self.info = format!("tables:\t{}\nreqs:\t{}\nords:\t{}\ndeals:\t{}\ninvalid:\t{}\n", 
				self.ord_rec.tables.len(), self.ord_rec.reqs.len(), self.ord_rec.ords.len(),
				deals, fails);
			
			&self.info
		}
		else {
			&self.info
		}
	}

	/// 回傳未連結的Req的統計資料
	pub fn list_unlink_req(&mut self) -> String {
		let unlinked_req: Vec<(&String, &Rec)> = self.ord_rec.reqs.iter().filter(|v| !v.1.linked ).collect();
		let mut ret = String::new();
		if !unlinked_req.is_empty() {
			let cnt_str = format!("count:{}\n", unlinked_req.len());
			ret.push_str(&cnt_str);
		}
		for (k, r) in unlinked_req {
			let reqinfo = format!("{} reqkey:{}", r.get_timestamp(), k);
			ret.push_str(&reqinfo);			
			let user = self.ord_rec.get_value(r, "User");
			if !user.is_empty() {
				let desc_user = format!(", user: {}", user);
				ret.push_str(&desc_user);
			}

			ret.push_str("\n");
		}
		ret
	}

	pub fn req_flow_statistic(&self) -> String {
		let mut ret = String::new();
		// 建一個統計流量的hasp map
		let mut flow_map = HashMap::<i64, i32>::new();
		// 取全部的req, 取出其中的timestamp, 拆出整數部份(秒), 填入haspmap中統計次數
		for req in &self.ord_rec.reqs {
			if req.1.reqs_vec.len() > 3 {
				let tm = &req.1.reqs_vec[3];
				let parts: Vec<&str> = tm.split('.').collect();
				if let Ok(int_part) = parts[0].parse::<i64>() {
					let cnt = match flow_map.get(&int_part) {
						Some(v) => v.to_owned(),
						_ => 0,
					};
					flow_map.insert(int_part, cnt + 1);
				}
			}
		};
		// 將hashmap轉為Vec
		let mut sort_map = flow_map.into_iter().collect::<Vec<_>>();
		// 將Vec排序
		sort_map.sort_by(|a, b| a.0.cmp(&b.0));
		// 印出結果
		for (t, cnt) in sort_map {
			if let Single(datetime) = Local.timestamp_opt(t, 0) {
				let tmstr = format!("{}, {},{}\n", t, datetime.format("%Y%m%d%H%M%S"), cnt);
				ret.push_str(&tmstr);
			}
		}
		ret
	}

	/// 統計某一欄位的數量: 例如TwfNew總共有多少個user
	pub fn statistic_field(&self, table_name: &str, field_name: &str) -> String {
		return self.ord_rec.statistic_field(table_name, field_name);
	}

	/// 解析每一行的內容, 並儲存到HashMap
	pub fn parse_line(&mut self, line: &str, log: &str) {
		let toks : Vec<String> = line.split('\x01').map(|s| s.to_string()).collect();

		if toks.len() > 3 {
			self.prevkey = self.ord_rec.insert_rec(toks, line, log);
		} else {
			//println!("log line: {}", line);
		}
	}

	/// 從輸入中解析出所有條件
	pub fn find_by_conditions(&mut self, condstr: &str, savefile: &str, hide: &bool) {
		let mut list_of_list = None;
		for cond in condstr.split(',') {
			let toks : Vec<&str> = cond.split(':').collect();
			if toks.len() > 2 {
				match list_of_list {
					Some(lol) => list_of_list = self.ord_rec.find_list(lol, toks[0], toks[1], toks[2]),
					None => list_of_list = self.ord_rec.check_req_data(toks[0], toks[1], toks[2], hide),
				}					
			} else {
				println!("{} is not correct! please specify TableName:FieldName:Value", cond);
			}
		};
		match list_of_list {
			Some(ret) => {
				println!("{} occurence found.", ret.len());
				if !hide {
					for list in &ret {
						self.ord_rec.print_ord_list(&list);
					}
				}
				self.save_to_file(&ret, savefile);
			},
			None => println!("not found any matches"),
		};
	}

	/// 把list of list 存到檔案
	pub fn save_to_file(&self, list_of_list: &LinkedList<LinkedList<&Rec>>, savefile: &str) {
		if let Ok(mut buff) = File::create(savefile) {
			for list in list_of_list {
				match buff.write(self.ord_rec.ord_list_to_string(&list).as_bytes()) {
					Ok(_) => (),
					_ => (),
				}
			}					
		}
	}
	
	/// 輸入 表名/欄位名/值 來尋找目標
	#[allow(dead_code)]
	pub fn find_by_field(&mut self, table_name: &str, field_name: &str, search_target: &str) {
		// 先找看看 Req表
		match self.ord_rec.check_req_data(table_name, field_name, search_target, &true) {
			Some(list_of_list) =>
			for list in list_of_list {
				self.ord_rec.print_ord_list(&list);
			},
			None => println!("not found"),
		}
	}
}

/// 使Parse類別能以println列印出來
impl fmt::Display for Parser {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "tables: {} reqs: {} ords:{}", 
			self.ord_rec.tables.len(), self.ord_rec.reqs.len(), self.ord_rec.ords.len())
	}
}