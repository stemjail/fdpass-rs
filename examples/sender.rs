// Copyright (C) 2016 Mickaël Salaün
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, version 3 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

extern crate fdpass;
extern crate unix_socket;

use std::fs::{File, OpenOptions, remove_file};
use std::io::{self, Read, Write};
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::process::exit;
use unix_socket::UnixListener;

static SOCKET_PATH: &'static str = "./tmp_fdpass.sock";
static DATA_PATH: &'static str = "./tmp_fdpass.txt";


fn listen_and_send<T>(socket_path: &str, fd: T) -> io::Result<()> where T: AsRawFd {
    // TODO: Use libc::SO_REUSEADDR for unix socket instead of removing the file
    let _ = remove_file(&socket_path);
    let stream = try!(UnixListener::bind(&socket_path));
    for client in stream.incoming() {
        let mut c = try!(client);
        println!("New client, sending FD {:?}", fd.as_raw_fd());
        // If using the stream for something else than FD passing, we may need to
        // synchronise with a first `send_fd()` before the useful `send_fd()`.
        try!(fdpass::send_fd(&mut c, &[0], &fd));
    }
    Ok(())
}

fn init_data<T>(path: T) -> io::Result<File> where T: AsRef<Path> {
    println!("Enter a string to be written to the {} file:", path.as_ref().display());
    let mut buffer = String::new();
    try!(io::stdin().read_line(&mut buffer));
    let _ = remove_file(&path);
    let mut file = try!(OpenOptions::new().read(true).write(true).create(true).open(path));
    try!(file.write_all(buffer.as_bytes()));
    Ok(file)
}

fn main() {
    println!("You may want to remove the files {} and {} after this process is completed.",
             SOCKET_PATH, DATA_PATH);
    let file = match init_data(DATA_PATH) {
        Ok(f) => f,
        Err(e) => {
            println!("Error: {}", e);
            exit(1);
        }
    };
    println!("Waiting for clients...");
    if let Err(e) = listen_and_send(SOCKET_PATH, file) {
        println!("Error: {}", e);
        exit(1);
    }
}
