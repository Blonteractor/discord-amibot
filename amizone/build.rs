fn main() {
    tonic_build::configure()
        .build_server(false)
        .protoc_arg("--experimental_allow_proto3_optional")
        .include_file("_includes.rs")
        .compile(
            &["./proto/v1/amizone.proto"],
            &["./proto/googleapis", "./proto/grpc-gateway", "./proto/v1"],
        )
        .unwrap_or_else(|e| panic!("Failed to compile protos: {:?}", e));
}
