use windows::{
    Win32::System::Diagnostics::Debug::OutputDebugStringW,
    core::{HSTRING, PCWSTR},
};

struct DbgLogger;

impl log::Log for DbgLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let s = format!(
            "[{}] {}({}): {}\r\n",
            record.level(),
            record.file().unwrap_or("<unknown>"),
            record.line().unwrap_or(0),
            record.args()
        );
        output_debug_string(&s);
    }

    fn flush(&self) {}
}

static LOGGER: DbgLogger = DbgLogger;

pub fn output_debug_string(s: &str) {
    let hs = HSTRING::from(s);
    unsafe {
        OutputDebugStringW(PCWSTR(hs.as_ptr()));
    }
}

pub fn init_dbg_logger() {
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(log::LevelFilter::Trace);
}
