use log::Logger;
use log::LogRecord;
use std::old_io::{ LineBufferedWriter, File, Writer };

pub struct FileLogger {
    file: Option<LineBufferedWriter<File>>,
}

impl FileLogger {
    pub fn new(filename: &String, mute: bool) -> FileLogger {
        if mute {
            FileLogger {
                file: None,
            }
        } else {
            FileLogger {
                file: Some(LineBufferedWriter::new(File::create(&Path::new(filename.clone())).unwrap())),
            }
        }
    }
}

impl Logger for FileLogger {
    fn log(&mut self, record: &LogRecord) {
        if let Some(ref mut f) = self.file {
            println!("{}:{}: {}",
                record.level, record.module_path, record.args);
            f.write_line(format!("{}:{}: {}",
                                record.level,
                                record.module_path,
                                record.args).as_slice()).unwrap();;
        }
    }
}

impl Drop for FileLogger {
    fn drop(&mut self) {
        if let Some(ref mut f) = self.file {
            match f.flush() {
                Err(e) => panic!("failed to flush a logger: {}", e),
                Ok(()) => () //panic!("DROPPING! {}", self.mute)
            }
        }
    }
}
