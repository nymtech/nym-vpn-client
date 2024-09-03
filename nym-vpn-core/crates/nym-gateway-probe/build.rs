fn main() {
    rust2go::Builder::new()
        .with_go_src("./netstack_ping")
        .build();
}
