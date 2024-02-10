test:
    RUST_BACKTRACE=0 cargo test --doc --all-features

doc:
    # cargo build
    RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --all-features
    firefox ./target/doc/parsa/index.html
