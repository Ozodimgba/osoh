use criterion::{Criterion, black_box, criterion_group, criterion_main};
use prost::Message;
use rpc_codec::{BincodeCodec, RpcCodec};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Clone, PartialEq, prost::Message)]
pub struct ProtobufTestData {
    #[prost(uint32, tag = "1")]
    pub id: u32,
    #[prost(string, tag = "2")]
    pub name: String,
    #[prost(uint64, tag = "3")]
    pub value: u64,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BincodeTestData {
    pub id: u32,
    pub name: String,
    pub value: u64,
}

fn create_test_data() -> (ProtobufTestData, BincodeTestData) {
    let pb_data = ProtobufTestData {
        id: u32::MAX,
        name: "test_message".to_string(),
        value: u64::MAX,
    };

    let bc_data = BincodeTestData {
        id: u32::MAX,
        name: "test_message".to_string(),
        value: u64::MAX,
    };

    (pb_data, bc_data)
}

fn benchmark_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode");
    let (pb_data, bc_data) = create_test_data();
    let codec = BincodeCodec::new();

    group.bench_function("protobuf", |b| {
        let mut buf = [0u8; 32];
        b.iter(|| {
            let mut slice = &mut buf[..]; // Convert to &mut [u8] due to UB in BufMut trait: pointer advancement after input
            black_box(pb_data.encode(&mut slice).unwrap())
        });
    });

    group.bench_function("bincode", |b| {
        let config = bincode::config::standard().with_fixed_int_encoding();
        let mut buffer = [0u8; 32];

        b.iter(|| {
            bincode::serde::encode_into_slice(&bc_data, &mut buffer[..], config).unwrap();
            black_box(&mut buffer);
        })
    });

    group.bench_function("protobuf_heap_alloc", |b| {
        b.iter(|| black_box(pb_data.encode_to_vec()))
    });

    group.bench_function("bincode_heap_alloc", |b| {
        b.iter(|| black_box(codec.encode(&bc_data).unwrap()))
    });

    group.finish();
}

fn benchmark_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode");
    let (pb_data, bc_data) = create_test_data();
    let codec = BincodeCodec::new();

    let pb_bytes = pb_data.encode_to_vec();
    let bc_bytes = codec.encode(&bc_data).unwrap();

    println!("Data sizes:");
    println!("  Protobuf: {} bytes", pb_bytes.len());
    println!("  Bincode:  {} bytes", bc_bytes.len());

    group.bench_function("protobuf", |b| {
        b.iter(|| ProtobufTestData::decode(black_box(&pb_bytes[..])).unwrap())
    });

    group.bench_function("bincode", |b| {
        b.iter(|| {
            codec
                .decode::<BincodeTestData>(black_box(&bc_bytes))
                .unwrap()
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_encode, benchmark_decode);
criterion_main!(benches);
