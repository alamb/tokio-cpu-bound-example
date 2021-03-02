// Shell 1:
// cargo run
//
// Shell 2:
// time echo foo | socat -t30 - TCP:localhost:1234
//
// Set WORKER_PRIORITY to 0 and you'll see ^^^ being very slow
// Set it to 10 and you'll see a bounded slowdown (from 800ms to 1100ms on my machine).

use std::io::Result;
use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicUsize, Arc};
use std::{thread, time};
use tokio::io::{copy, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::{self, Handle};

const NUM_WORKERS: usize = 60;
const WORKER_PRIORITY: i32 = 10;

async fn process_socket(mut socket: TcpStream) -> Result<()> {
    println!("processing socket");

    let (mut reader, mut writer) = socket.split();

    //writer.write_all(b"some heavy computing...").await?;
    writer.write_all(b"processing ping...").await?;
    //heavy_stuff(get_count().await);
    writer.write_all(b"done. echoing\n").await?;

    copy(&mut reader, &mut writer).await?;

    Ok(())
}


async fn aworker() {
    println!("Running a worker job");
    loop {
        // Give other tasks a chance to work
        let count = get_count().await;
        heavy_stuff(count);
        tokio::task::yield_now().await;
    }
}

async fn get_count() -> u64 {
    //20000000
    200000
}

fn heavy_stuff(count: u64) -> u64 {
    let mut acc = 0;
    for _i in 0..count {
        acc += 1;
    }
    acc
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:1234").await?;

    // Model running workers on the main tokio thread
    for _i in 0..NUM_WORKERS {
        tokio::task::spawn(async {
            aworker().await
        });
    }

    loop {
        let (socket, _) = listener.accept().await?;
        process_socket(socket).await?;
    }
}
