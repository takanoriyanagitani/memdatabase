use std::io;

fn main() -> Result<(), io::Error> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile(
            &[
                "memdatabase/v1/dget.proto",
                "memdatabase/v1/dset.proto",
                "memdatabase/v1/get.proto",
                "memdatabase/v1/set.proto",
                "memdatabase/v1/pop.proto",
                "memdatabase/v1/push.proto",
                "memdatabase/v1/sadd.proto",
                "memdatabase/v1/sdel.proto",
                "memdatabase/v1/svc.proto",
            ],
            &["memdatabase-proto/"],
        )?;
    Ok(())
}
