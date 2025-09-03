# SOR Log Parser

This is a tool for parsing and analyzing SOR (SorReqOrd) log files. It can be run as a command-line tool or as a graphical user interface (GUI) application.

## Building the Application

To build the application, you need to have Rust and Cargo installed. You can build the project by running the following command in the project's root directory:

```bash
cargo build --release
```

## Usage

### Command-Line Interface (CLI)

To use the CLI, you need to provide the path to the log file as an argument.

```bash
./target/release/sor_logparser /path/to/your/SorReqOrd.log
```

There are several options available for the CLI:

*   `-f, --field <field>`: Search for specific records. Example: `-f TwsNew:SorRID:100001`
*   `-e, --encoding <encoding>`: Specify the encoding of the log file (default: `BIG5`).
*   `-s, --save`: Save the output to a file.
*   `-h, --hide`: Do not print the result list to the console.
*   `-o, --output <savepath>`: Specify the path for the saved output file.
*   `-t, --statistic <table-field>`: Get statistics for a specific field. Example: `-t TwfNew:user`
*   `-w, --flow`: Show the request flow per second.

### Graphical User Interface (GUI)

To launch the GUI, use the `--gui` flag:

```bash
./target/release/sor_logparser --gui
```

This will open a window with a button to "Open Log File". Clicking this button will open a file dialog, allowing you to select a log file for parsing. The results will be displayed in the text area below the button.
