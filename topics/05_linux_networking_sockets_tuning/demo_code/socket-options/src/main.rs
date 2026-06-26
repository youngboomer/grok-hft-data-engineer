//! Educational example: setting important socket options for low latency.
//! In real code you would do this on the underlying TCP stream of your WS library.

use std::net::TcpStream;
use std::os::unix::io::AsRawFd;

fn main() {
    let stream = TcpStream::connect("127.0.0.1:12345").unwrap_or_else(|_| {
        // We don't actually need a live connection for the demo of setsockopt.
        TcpStream::connect("example.com:80").expect("need some TCP target for demo")
    });

    let fd = stream.as_raw_fd();

    // TCP_NODELAY
    let on: libc::c_int = 1;
    unsafe {
        libc::setsockopt(
            fd,
            libc::IPPROTO_TCP,
            libc::TCP_NODELAY,
            &on as *const _ as *const libc::c_void,
            std::mem::size_of_val(&on) as libc::socklen_t,
        );
    }
    println!("TCP_NODELAY set (if you have permissions).");
    println!("In production: set this + tune SO_RCVBUF/SO_SNDBUF + consider busy poll.");
}
