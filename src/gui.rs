// 引入所需的 crate
use rfd::FileDialog;
use std::fs::File;
use std::io::BufReader;
use crate::parser::Parser;
use crate::fileread::read_data_log;

// 使用 slint! 巨集來定義 GUI
slint::slint! {
    // 從 "std-widgets.slint" 導入標準元件
    import { Button, ScrollView, TextEdit } from "std-widgets.slint";

    // 定義主視窗元件 AppWindow
    export component AppWindow inherits Window {
        // 設定視窗標題
        title: "SOR Log Parser";
        // 設定視窗寬度
        width: 800px;
        // 設定視窗高度
        height: 600px;

        // 定義一個可以在 Rust 和 Slint 之間雙向綁定的屬性
        // 用於顯示解析結果
        in-out property <string> output_text: "Parsed output will be shown here.";

        // 定義一個回呼函數，用於觸發檔案選擇對話框
        callback open_file_dialog();

        // 使用垂直佈局
        VerticalLayout {
            // 新增一個按鈕
            Button {
                text: "Open Log File";
                // 按鈕被點擊時，觸發 open_file_dialog 回呼
                clicked => { root.open_file_dialog() }
            }
            // 新增一個可滾動的視圖
            ScrollView {
                width: 100%;
                height: 100%;
                // 在可滾動視圖中新增一個文字編輯區
                TextEdit {
                    // 綁定 output_text 屬性到文字編輯區的 text 屬性
                    text: root.output_text;
                    // 設定為唯讀
                    read-only: true;
                }
            }
        }
    }
}

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

                    // 準備一個字串來存放結果
                    let mut summary = String::new();
                    summary.push_str("-=summary=-\n");
                    summary.push_str(parser.get_info());
                    summary.push_str("\n");

                    // 獲取未連結的請求資訊
                    let unlinkreqs_info = parser.list_unlink_req();
                    if !unlinkreqs_info.is_empty() {
                        summary.push_str("there are unlink reqs:\n");
                        summary.push_str(&unlinkreqs_info);
                    }

                    // 更新 GUI 的 output_text 屬性
                    ui.set_output_text(summary.into());
                } else {
                    // 如果檔案開啟失敗，顯示錯誤訊息
                    ui.set_output_text(format!("Error opening file: {}", path.display()).into());
                }
            }
        }
    });

    // 運行 GUI 事件循環
    ui.run().unwrap();
}
