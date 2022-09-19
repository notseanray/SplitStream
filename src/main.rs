use std::io::{BufReader, Read, BufRead};
use std::net::UdpSocket;
use std::error::Error;
use std::os::unix::prelude::FileExt;
use std::{env, io};
use std::fs::File;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if env::args().len() > 1 {
        println!("client");
        let socket = UdpSocket::bind("127.0.0.1:1200")?;
        socket.connect("127.0.0.1:1231")?;
        let f = File::open("test.txt")?;
        let split = SplitStreamFile::new(&f, "new_test.txt", 1)?;
        split.send(&f, socket)?;
        // println!("{:?}", split.hash);

    } else {
        println!("host");
        let socket = UdpSocket::bind("127.0.0.1:1231")?;
        let mut buf = [0; 100];
        while let Ok(v) = socket.recv(&mut buf) {
            println!("{:?}", buf);
        }
    }
    Ok(())
}

struct SplitStreamFile<'a> {
    name: &'a str,
    size: u64,
    chunks: u64,
    hash: [u8; 32],
}

impl <'a>SplitStreamFile<'a> {
    pub(crate) fn new(mut f: &'a File, name: &'a str, chunks: u64) -> Result<Self, Box<dyn Error>> {
        if let Ok(data) = f.metadata() {
            let mut hasher = blake3::Hasher::new();
            let _n = io::copy(&mut f, &mut hasher)?;
            let hash = hasher.finalize();
            println!("{:?}", hash.to_hex());
            return Ok(SplitStreamFile {
                name,
                size: data.len(),
                chunks,
                hash: *hash.as_bytes(),
            });
        }
        unimplemented!();
    }

    pub(crate) fn send(&self, file: &File, socket: UdpSocket) -> Result<u64, Box<dyn Error>> {
        let chunk_size = (self.size as f32 / self.chunks as f32).ceil() as usize;
        for c in 0..self.chunks as usize {
            let mut chunk_data = vec![0; chunk_size];
            let _ = file.read_at(&mut chunk_data, c as u64 * chunk_size as u64)?;
            socket.send(&chunk_data)?;
        }
        Ok(0)
    }
}





