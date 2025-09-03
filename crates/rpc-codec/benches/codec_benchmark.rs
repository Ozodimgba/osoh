use criterion::{Criterion, black_box, criterion_group, criterion_main};
use prost::Message;
use rpc_codec::{BincodeCodec, RpcCodec};
use serde::{Deserialize, Serialize};

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
        id: 12345,
        name: "test_message".to_string(),
        value: 9876543210,
    };

    let bc_data = BincodeTestData {
        id: 12345,
        name: "test_message".to_string(),
        value: 9876543210,
    };

    (pb_data, bc_data)
}

fn benchmark_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode");
    let (pb_data, bc_data) = create_test_data();
    let codec = BincodeCodec::new();

    group.bench_function("protobuf", |b| {
        b.iter(|| black_box(pb_data.encode_to_vec()))
    });

    group.bench_function("bincode", |b| {
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
