use std::error::Error;
mod tcp_communication;
use clap::App as clapapp;
use clap::Arg;
use keri::{database::lmdb::LmdbEventDatabase, keri::Keri, signer::CryptoBox};
use tempfile::tempdir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments.
    let matches = clapapp::new("get-command-line-args")
        .arg(
            Arg::with_name("host")
                .short('H'.to_string())
                .help("hostname on which we would listen, default: localhost")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short('P'.to_string())
                .help("port on which we would open TCP connections, default: 5621")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("bob")
                .short('B'.to_string())
                .help("run bob demo")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("sam")
                .short('S'.to_string())
                .help("run sam demo")
                .takes_value(false),
        )
        .get_matches();

    let host = matches.value_of("host").unwrap_or("localhost");
    let port = matches.value_of("port").unwrap_or("5621");
    let address = [host, ":", port].concat();

    let dir = tempdir()?;
    if matches.is_present("bob") {
        let key_manager = CryptoBox::derive_from_seed(&[
            "ArwXoACJgOleVZ2PY7kXn7rA0II0mHYDhc6WrBH8fDAc",
            "A6zz7M08-HQSFq92sJ8KJOT2cZ47x7pXFQLPB0pckB3Q",
            "AcwFTk-wgk3ZT2buPRIbK-zxgPx-TKbaegQvPEivN90Y",
        ])?;
        let db = LmdbEventDatabase::new(dir.path())?;
        let mut bob = Keri::new(
            db,
            key_manager,
            "EH7Oq9oxCgYa-nnNLvwhp9sFZpALILlRYyB-6n4WDi7w".parse()?,
        )?;

        let bob_icp = &bob.incept()?.serialize()?;

        tcp_communication::send(bob_icp, &address, &bob).await?;

        let bob_rot = &bob.rotate()?.serialize()?;

        tcp_communication::send(bob_rot, &address, &bob).await?;

        let bob_ixn = &bob.make_ixn(None)?.serialize()?;

        tcp_communication::send(bob_ixn, &address, &bob).await?;
    } else if matches.is_present("sam") {
        let key_manager = CryptoBox::derive_from_seed(&[
            "ArwXoACJgOleVZ2PY7kXn7rA0II0mHYDhc6WrBH8fDAc",
            "A6zz7M08-HQSFq92sJ8KJOT2cZ47x7pXFQLPB0pckB3Q",
            "AcwFTk-wgk3ZT2buPRIbK-zxgPx-TKbaegQvPEivN90Y",
        ])?;
        let db = LmdbEventDatabase::new(dir.path())?;
        let mut sam = Keri::new(
            db,
            key_manager,
            "EH7Oq9oxCgYa-nnNLvwhp9sFZpALILlRYyB-6n4WDi7w".parse()?,
        )?;

        let bob_icp = &sam.incept()?.serialize()?;

        tcp_communication::send(bob_icp, &address, &sam).await?;

        let bob_ixn = &sam.make_ixn(None)?.serialize()?;

        tcp_communication::send(bob_ixn, &address, &sam).await?;

        let bob_rot = &sam.rotate()?.serialize()?;

        tcp_communication::send(bob_rot, &address, &sam).await?;
    } else {
        let key_manager = CryptoBox::derive_from_seed(&[
            "AgjD4nRlycmM5cPcAkfOATAp8wVldRsnc9f1tiwctXlw",
            "AKUotEE0eAheKdDJh9QvNmSEmO_bjIav8V_GmctGpuCQ",
        ])?;
        let db = LmdbEventDatabase::new(dir.path())?;
        let mut eve = Keri::new(
            db,
            key_manager,
            "EpDA1n-WiBA0A8YOqnKrB-wWQYYC49i5zY_qrIZIicQg".parse()?,
        )?;
        eve.incept()?;

        tcp_communication::run(&address, eve).await?;
    }
    dir.close()?;
    Ok(())
}
