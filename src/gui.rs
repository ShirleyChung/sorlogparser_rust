// 引入所需的 crate
use rfd::FileDialog;
use std::fs::File;
use std::io::BufReader;
use crate::parser::Parser;
use crate::fileread::read_data_log;

// 使用 slint! 巨集來定義 GUI
slint::slint! {
    import { Button, ScrollView } from "std-widgets.slint";

    export component AppWindow inherits Window {
        title: "SOR Log Parser";
        width: 1024px;
        height: 768px;

        in-out property <[string]> column_data: [];
        in-out property <[[string]]> row_data: [[]];

        callback open_file_dialog();

        VerticalLayout {
            Button {
                text: "Open Log File";
                clicked => { root.open_file_dialog() }
            }
            ScrollView {
                VerticalLayout {
                    // Header
                    HorizontalLayout {
                        padding: 5px;
                        for header_text in column_data : Text {
                            text: header_text;
                            width: 120px; // Fixed width for alignment
                        }
                    }
                    // Rows
                    for row in row_data : HorizontalLayout {
                        padding: 5px;
                        for cell_text in row : Text {
                            text: cell_text;
                            width: 120px; // Fixed width for alignment
                        }
                    }
                }
            }
        }
    }
}

use slint::{ModelRc, SharedString, VecModel};
use std::rc::Rc;
use std::collections::HashSet;

// GUI 的主函數
pub fn run() {
    // 創建 AppWindow 元件的實例
    let ui = AppWindow::new().unwrap();

    // 設定 open_file_dialog 回呼的處理邏輯
    ui.on_open_file_dialog({
        // 使用弱引用以避免循環引用
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            // 使用 rfd crate 來開啟系統的檔案選擇對話框
            if let Some(path) = FileDialog::new().pick_file() {
                // 如果成功選擇了一個檔案
                if let Ok(f) = File::open(&path) {
                    // 開啟檔案並創建 BufReader
                    let mut reader = BufReader::new(f);
                    // 創建 Parser 實例
                    let mut parser = Parser::new();
                    // 讀取並解析日誌檔案，預設使用 BIG5 編碼
                    read_data_log(&mut reader, &mut parser, "BIG5");

                    // 1. Extract Column Headers
                    let mut column_set = HashSet::new();
                    for table in parser.ord_rec.tables.values() {
                        for key in table.index.keys() {
                            column_set.insert(key.clone());
                        }
                    }
                    let mut sorted_headers: Vec<SharedString> = column_set.into_iter().map(|s| s.into()).collect();
                    sorted_headers.sort();
                    let header_strings = sorted_headers.clone();
                    ui.set_column_data(Rc::new(VecModel::from(sorted_headers)).into());

                    // 2. Extract Row Data
                    let mut all_rows = Vec::new();
                    let all_recs = parser.ord_rec.reqs.values()
                        .chain(parser.ord_rec.ords.values().flatten());

                    for rec in all_recs {
                        let mut row_vec: Vec<SharedString> = Vec::new();
                        for header in &header_strings {
                            let value = parser.ord_rec.get_value(rec, header.as_str());
                            row_vec.push(value.into());
                        }
                        all_rows.push(Rc::new(VecModel::from(row_vec)).into());
                    }

                    ui.set_row_data(Rc::new(VecModel::from(all_rows)).into());

                } else {
                    // Handle file open error if necessary, maybe with a dialog
                    // For now, we'll just clear the table
                    ui.set_column_data(Rc::new(VecModel::from(Vec::<SharedString>::new())).into());
                    ui.set_row_data(Rc::new(VecModel::from(Vec::<ModelRc<SharedString>>::new())).into());
                }
            }
        }
    });

    // 運行 GUI 事件循環
    ui.run().unwrap();
}
