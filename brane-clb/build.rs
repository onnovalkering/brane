fn main() -> Result<(), std::io::Error> {
    tonic_build::configure()
    .format(false)
    .compile(
        &["proto/callback.proto"],
        &["proto"],
    )
}
