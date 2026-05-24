fn main() {
    protobuf_codegen::Codegen::new()
        .pure()
        .includes(&["proto"])
        .inputs(&["proto/protocol.proto"])
        .cargo_out_dir("protos")
        .run_from_script();
}
