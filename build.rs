fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .build_transport(true)
        .compile_protos(
            &[
                "proto/etcdv3/rpc.proto",
                // "proto/etcdv3/auth.proto",
                // "proto/etcdv3/kv.proto",
                // "proto/etcdv3/membership.proto",
            ],
            &["proto/etcdv3"],
        )?;
    Ok(())
}
