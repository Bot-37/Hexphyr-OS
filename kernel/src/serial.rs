use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use log::{Level, LevelFilter, Log, Metadata, Record};

const COM1: u16 = 0x3F8;

pub struct SerialPort {
    port: u16,
}

impl SerialPort {
    pub const unsafe fn new(port: u16) -> Self {
        SerialPort { port }
    }
    
    pub fn init(&mut self) {
        unsafe {
            // Disable interrupts
            self.outb(1, 0x00);
            
            // Set baud rate (38400)
            self.outb(3, 0x80); // Enable DLAB
            self.outb(0, 0x03); // Divisor low byte
            self.outb(1, 0x00); // Divisor high byte
            
            // 8 bits, no parity, one stop bit
            self.outb(3, 0x03);
            
            // Enable FIFO, clear buffers, 14-byte threshold
            self.outb(2, 0xC7);
            
            // Enable interrupts
            self.outb(4, 0x0B);
        }
    }
    
    pub fn write_byte(&mut self, byte: u8) {
        unsafe {
            // Wait for transmit buffer empty
            while self.inb(5) & 0x20 == 0 {}
            self.outb(0, byte);
        }
    }
    
    pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }
    
    unsafe fn outb(&mut self, reg: u16, value: u8) {
        x86_64::instructions::port::Port::new(self.port + reg).write(value);
    }
    
    unsafe fn inb(&mut self, reg: u16) -> u8 {
        x86_64::instructions::port::Port::new(self.port + reg).read()
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref COM1_PORT: Mutex<SerialPort> = Mutex::new(unsafe { SerialPort::new(COM1) });
}

pub struct SerialLogger;

impl Log for SerialLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }
    
    fn log(&self, record: &Record) {
        let level = record.level();
        let args = record.args();
        
        COM1_PORT.lock().write_str(match level {
            Level::Error => "[ERROR] ",
            Level::Warn => "[WARN]  ",
            Level::Info => "[INFO]  ",
            Level::Debug => "[DEBUG] ",
            Level::Trace => "[TRACE] ",
        });
        
        use core::fmt::Write;
        let _ = write!(COM1_PORT.lock(), "{}\n", args);
    }
    
    fn flush(&self) {}
}

pub static LOGGER: SerialLogger = SerialLogger;

pub fn init() {
    COM1_PORT.lock().init();
}