fn main() {
    embed_resource::compile("res/dll.rc", embed_resource::NONE)
        .manifest_required()
        .unwrap();
}
