// 引入所需的 crate
use rfd::FileDialog;
use std::fs;
use crate::parser::Parser;
use crate::fileread::read_data_log;
use std::io::BufReader;

// 使用 slint! 巨集來定義 GUI
slint::slint! {
    import { Button, ScrollView } from "std-widgets.slint";

    export component AppWindow inherits Window {
        title: "SOR Log Parser - PKI Mode";
        width: 1024px;
        height: 768px;

        in-out property <[string]> column_data: [];
        in-out property <[[string]]> row_data: [[]];
        in-out property <string> status_text: "Ready";

        callback open_dir_dialog();

        VerticalLayout {
            Button {
                text: "Select Directory to Parse";
                clicked => { root.open_dir_dialog() }
            }
            Text {
                text: root.status_text;
                color: #666;
                font-size: 12px;
            }
            ScrollView {
                VerticalLayout {
                    // Header
                    HorizontalLayout {
                        padding: 5px;
                        for header_text in column_data : Text {
                            text: header_text;
                            width: 150px;
                        }
                    }
                    // Rows
                    for row in row_data : HorizontalLayout {
                        padding: 5px;
                        for cell_text in row : Text {
                            text: cell_text;
                            width: 150px;
                        }
                    }
                }
            }
        }
    }
}

use slint::{ModelRc, SharedString, VecModel};
use std::rc::Rc;
use regex::Regex;

// GUI 的主函數
pub fn run() {
    // 創建 AppWindow 元件的實例
    let ui = AppWindow::new().unwrap();

    // 設定 open_dir_dialog 回呼的處理邏輯
    ui.on_open_dir_dialog({
        // 使用弱引用以避免循環引用
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            
            // 使用 rfd crate 來開啟系統的目錄選擇對話框
            if let Some(dir_path) = FileDialog::new().pick_folder() {
                ui.set_status_text("Scanning directories...".into());
                
                // 掃描所有日期格式的子目錄 (YYYYMMDD)
                let date_regex = Regex::new(r"^\d{8}$").unwrap();
                let mut date_dirs = Vec::new();
                
                if let Ok(entries) = fs::read_dir(&dir_path) {
                    for entry in entries.flatten() {
                        if let Ok(metadata) = entry.metadata() {
                            if metadata.is_dir() {
                                if let Some(dir_name) = entry.file_name().to_str() {
                                    if date_regex.is_match(dir_name) {
                                        date_dirs.push(entry.path());
                                    }
                                }
                            }
                        }
                    }
                }
                
                date_dirs.sort();
                
                if date_dirs.is_empty() {
                    ui.set_status_text("No date directories found (YYYYMMDD format)".into());
                    ui.set_column_data(Rc::new(VecModel::from(Vec::<SharedString>::new())).into());
                    ui.set_row_data(Rc::new(VecModel::from(Vec::<ModelRc<SharedString>>::new())).into());
                    return;
                }
                
                let status_msg = format!("Found {} date directories, parsing...", date_dirs.len());
                ui.set_status_text(status_msg.into());
                
                let mut all_pki_lines = Vec::new();
                let mut _total_parsed = 0;
                
                // 為每個日期目錄進行解析
                for date_dir in &date_dirs {
                    let req_log_path = date_dir.join("SorReqOrd.log");
                    
                    if req_log_path.exists() {
                        if let Ok(f) = std::fs::File::open(&req_log_path) {
                            let mut reader = BufReader::new(f);
                            let mut parser = Parser::new();
                            read_data_log(&mut reader, &mut parser, "BIG5");
                            
                            // 使用默認搜尋條件
                            let default_conditions = "TwfNew:SesName:SorAPI|TwfChg:SesName:SorAPI|FrfNew:SesName:SorAPI|FrfChg:SesName:SorAPI";
                            
                            // 執行搜尋並輸出 PKI 格式
                            parser.find_by_conditions(default_conditions, "", &false, true, true);
                            
                            // 獲取 PKI 輸出
                            let pki_output = parser.get_pki_output();
                            if !pki_output.is_empty() {
                                all_pki_lines.extend(pki_output.lines().map(|s| s.to_string()));
                                _total_parsed += 1;
                            }
                        }
                    }
                }
                
                // 顯示 PKI 輸出結果
                if !all_pki_lines.is_empty() {
                    let status_msg = format!("Parsed {} directories, {} PKI records", date_dirs.len(), all_pki_lines.len());
                    ui.set_status_text(status_msg.into());
                    
                    // 設置列標題
                    let headers = vec!["Date", "BrkNo", "Ivac", "Type", "FromUID", "Time", "DigsgnHash"];
                    let column_data: Vec<SharedString> = headers.iter().map(|h| (*h).into()).collect();
                    ui.set_column_data(Rc::new(VecModel::from(column_data)).into());
                    
                    // 解析 PKI 行並轉換為表格行
                    let mut rows = Vec::new();
                    for line in all_pki_lines {
                        let parts: Vec<&str> = line.split('|').collect();
                        if parts.len() >= 8 {
                            let row: Vec<SharedString> = vec![
                                parts[1].into(),  // Date
                                parts[2].into(),  // BrkNo
                                parts[3].into(),  // Ivac
                                parts[4].into(),  // Type (O/C/M)
                                parts[5].into(),  // FromUID
                                parts[6].into(),  // Time
                                parts[7].into(),  // Complete digsgn
                            ];
                            rows.push(Rc::new(VecModel::from(row)).into());
                        }
                    }
                    
                    ui.set_row_data(Rc::new(VecModel::from(rows)).into());
                } else {
                    ui.set_status_text("No PKI records found".into());
                    ui.set_column_data(Rc::new(VecModel::from(Vec::<SharedString>::new())).into());
                    ui.set_row_data(Rc::new(VecModel::from(Vec::<ModelRc<SharedString>>::new())).into());
                }
            }
        }
    });

    // 運行 GUI 事件循環
    ui.run().unwrap();
}
