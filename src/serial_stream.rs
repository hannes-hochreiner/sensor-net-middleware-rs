extern crate libc;
use core::pin::Pin;
use futures::{
    stream::Stream,
    task::{Context, Poll},
};
use std::error::Error;
use std::ffi::CString;
use std::mem::MaybeUninit;
use std::result::Result;
use tokio::signal::unix::{signal, Signal, SignalKind};

extern "C" {
    fn open(pathname: *const libc::c_char, mode: libc::c_int) -> libc::c_int;
    fn close(fd: libc::c_int) -> libc::c_int;
    fn tcgetattr(fd: libc::c_int, termios: *mut libc::termios) -> libc::c_int;
    fn tcsetattr(
        fd: libc::c_int,
        optional_actions: libc::c_int,
        termios: *const libc::termios,
    ) -> libc::c_int;
    fn cfsetispeed(termios: *mut libc::termios, speed: libc::speed_t) -> libc::c_int;
    fn cfsetospeed(termios: *mut libc::termios, speed: libc::speed_t) -> libc::c_int;
    fn read(fd: libc::c_int, buf: *mut u8, nbyte: libc::size_t) -> libc::ssize_t;
    fn fcntl(fd: libc::c_int, cmd: libc::c_int, ... /* arg */) -> libc::c_int;
}

pub struct SerialStream {
    fd: libc::c_int,
    signal: Signal,
}

impl SerialStream {
    pub fn new(path: &str) -> Result<SerialStream, Box<dyn Error>> {
        let fd = unsafe {
            open(
                CString::new(path)?.as_ptr(),
                libc::O_RDWR | libc::O_NOCTTY | libc::O_NDELAY,
            )
        };

        if fd == -1 {
            return Err(Box::new(SerialStreamError {
                description: format!("could not open device; error: {}", unsafe {
                    *(libc::__errno_location())
                }),
            }));
        }

        let mut options = MaybeUninit::uninit();

        if unsafe { tcgetattr(fd, options.as_mut_ptr()) } == -1 {
            return Err(Box::new(SerialStreamError {
                description: format!("could not get options; error: {}", unsafe {
                    *(libc::__errno_location())
                }),
            }));
        }

        let mut options = unsafe { options.assume_init() };

        // set 8N1
        options.c_cflag &= !libc::PARENB;
        options.c_cflag &= !libc::CSTOPB;
        options.c_cflag &= !libc::CSIZE;
        options.c_cflag |= libc::CS8;
        // disable hardware flow control
        options.c_cflag &= !libc::CRTSCTS;
        // enable receiver and set local mode
        options.c_cflag |= libc::CLOCAL | libc::CREAD;
        // select raw input
        options.c_lflag &= !(libc::ICANON | libc::ECHO | libc::ECHOE | libc::ISIG);
        // select raw output
        options.c_oflag &= !libc::OPOST;
        // set speed
        if unsafe { cfsetispeed(&mut options, libc::B1000000) } == -1 {
            return Err(Box::new(SerialStreamError {
                description: format!("could not set input speed; error: {}", unsafe {
                    *(libc::__errno_location())
                }),
            }));
        }

        if unsafe { cfsetospeed(&mut options, libc::B1000000) } == -1 {
            return Err(Box::new(SerialStreamError {
                description: format!("could not set output speed; error: {}", unsafe {
                    *(libc::__errno_location())
                }),
            }));
        }
        // set attributes
        if unsafe { tcsetattr(fd, libc::TCSANOW, &mut options) } == -1 {
            return Err(Box::new(SerialStreamError {
                description: format!("could not set options; error: {}", unsafe {
                    *(libc::__errno_location())
                }),
            }));
        }
        // set file status flags
        if unsafe { fcntl(fd, libc::F_SETFL, libc::O_ASYNC | libc::O_NONBLOCK) } == -1 {
            return Err(Box::new(SerialStreamError {
                description: format!("could not set file status flag; error: {}", unsafe {
                    *(libc::__errno_location())
                }),
            }));
        }

        return Ok(SerialStream {
            fd: fd,
            signal: signal(SignalKind::io()).unwrap(),
        });
    }
}

impl Drop for SerialStream {
    fn drop(&mut self) {
        unsafe { close(self.fd) };
    }
}

impl Stream for SerialStream {
    type Item = String;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        match self.signal.poll_recv(cx) {
            Poll::Ready(_) => {
                let mut buffer = Vec::with_capacity(1024);
                let len = unsafe { read(self.fd, buffer.as_mut_ptr(), buffer.capacity()) };

                if len > 0 {
                    unsafe { buffer.set_len(len as usize) };
                    Poll::Ready(Some(String::from_utf8(buffer).unwrap()))
                } else {
                    Poll::Ready(Some(String::new()))
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

#[derive(Debug)]
struct SerialStreamError {
    description: String,
}

impl std::error::Error for SerialStreamError {}

impl std::fmt::Display for SerialStreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SerialStreamError: {}", self.description)
    }
}
