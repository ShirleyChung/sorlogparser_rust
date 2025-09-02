use rfd::FileDialog;
use std::fs::File;
use std::io::BufReader;
use crate::parser::Parser;
use crate::fileread::read_data_log;

slint::slint! {
    import { Button, ScrollView, TextEdit } from "std-widgets.slint";

    export component AppWindow inherits Window {
        title: "SOR Log Parser";
        width: 800px;
        height: 600px;

        in-out property <string> output_text: "Parsed output will be shown here.";

        callback open_file_dialog();

        VerticalLayout {
            Button {
                text: "Open Log File";
                clicked => { root.open_file_dialog() }
            }
            ScrollView {
                width: 100%;
                height: 100%;
                TextEdit {
                    text: root.output_text;
                    read-only: true;
                }
            }
        }
    }
}

pub fn run() {
    let ui = AppWindow::new().unwrap();

    ui.on_open_file_dialog({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            if let Some(path) = FileDialog::new().pick_file() {
                if let Ok(f) = File::open(&path) {
                    let mut reader = BufReader::new(f);
                    let mut parser = Parser::new();
                    read_data_log(&mut reader, &mut parser, "BIG5");

                    let mut summary = String::new();
                    summary.push_str("-=summary=-\n");
                    summary.push_str(parser.get_info());
                    summary.push_str("\n");

                    let unlinkreqs_info = parser.list_unlink_req();
                    if !unlinkreqs_info.is_empty() {
                        summary.push_str("there are unlink reqs:\n");
                        summary.push_str(&unlinkreqs_info);
                    }

                    ui.set_output_text(summary.into());
                } else {
                    ui.set_output_text(format!("Error opening file: {}", path.display()).into());
                }
            }
        }
    });

    ui.run().unwrap();
}
