use std::sync::Arc;

use clap::command;
use futures::{channel::mpsc::channel, SinkExt};
use fvm_ipld_car::CarHeader;
use libipld::{
    cid::CidGeneric,
    multihash::{Code, MultihashDigest},
};
use tokio::sync::RwLock;

const RAW: u64 = 0x55;
const _DAG_CBOR: u64 = 0x71;

async fn new_car(size: usize) -> anyhow::Result<Vec<u8>> {
    let block: Vec<u8> = vec![1; size];
    let hash = Code::Blake2b256.digest(&block);
    let cid = CidGeneric::new_v1(RAW, hash);

    let buf = Arc::new(RwLock::new(Vec::new()));
    let header = CarHeader {
        roots: vec![cid],
        version: 1,
    };

    let (mut tx, mut rx) = channel(1);

    let buf_cloned = buf.clone();
    let write_task = tokio::task::spawn(async move {
        header
            .write_stream_async(&mut *buf_cloned.write().await, &mut rx)
            .await
    });

    tx.send((cid, block)).await?;
    drop(tx);
    write_task.await??;

    let buf = buf.write().await;
    Ok(buf.to_owned())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let matches = command!() // requires `cargo` feature
        .arg(
            clap::Arg::new("size")
                .value_parser(clap::value_parser!(usize))
                .required(true),
        )
        .get_matches();

    let size = *matches.get_one::<usize>("size").unwrap_or(&10);
    let contents = new_car(size).await?;

    std::fs::write("file.car", contents)?;

    println!("created new car file");

    Ok(())
}
