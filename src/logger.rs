use log::Logger;
use log::LogRecord;
use std::io::{LineBufferedWriter, File, Writer};

pub struct FileLogger {
    file:   LineBufferedWriter<File>
}

impl FileLogger {
    pub fn new(filename: &String) -> FileLogger {
        FileLogger { file: LineBufferedWriter::new(File::create(&Path::new(filename.clone())).unwrap()) }
    }
}

impl Logger for FileLogger {
    fn log(&mut self, record: &LogRecord) {
        println!("{}:{}: {}",
            record.level, record.module_path, record.args);
        match writeln!(self.file, "{}:{}: {}",
            record.level, record.module_path, record.args) {
            _ => ()
        }
    }
}

impl Drop for FileLogger {
    fn drop(&mut self) {
    }
}
