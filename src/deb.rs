use cargo_deb::*;

use std::path::Path;
use std::process;
use std::time;

use crate::Args;

struct CliOptions {
    no_build: bool,
    no_strip: bool,
    separate_debug_symbols: bool,
    fast: bool,
    verbose: bool,
    quiet: bool,
    install: bool,
    package_name: Option<String>,
    output_path: Option<String>,
    variant: Option<String>,
    target: Option<String>,
    manifest_path: Option<String>,
    cargo_build_flags: Vec<String>,
    deb_version: Option<String>,
}

pub fn main(args: &Args) {
    match process(CliOptions {
        no_build: true,
        no_strip: args.no_strip,
        separate_debug_symbols: args.separate_debug_symbols,
        quiet: args.quiet,
        verbose: args.verbose,
        install: args.install,
        // when installing locally it won't be transferred anywhere, so allow faster compression
        fast: args.install || args.fast,
        variant: args.variant.as_ref().map(|x| x.to_owned()),
        target: args.target.as_ref().map(|x| x.to_owned()),
        output_path: Some("target/package/".to_owned()),
        package_name: args.package_name.as_ref().map(|x| x.to_owned()),
        manifest_path: args.manifest_path.as_ref().map(|x| x.to_owned()),
        deb_version: args.deb_version.as_ref().map(|x| x.to_owned()),
        cargo_build_flags: vec![],
    }) {
        Ok(()) => {}
        Err(err) => {
            err_exit(&err);
        }
    }
}

#[allow(deprecated)]
fn err_cause(err: &dyn std::error::Error, max: usize) {
    if let Some(reason) = err.cause() {
        // we use cause(), not source()
        eprintln!("  because: {}", reason);
        if max > 0 {
            err_cause(reason, max - 1);
        }
    }
}

fn err_exit(err: &dyn std::error::Error) -> ! {
    eprintln!("cargo-deb: {}", err);
    err_cause(err, 3);
    process::exit(1);
}

fn process(
    CliOptions {
        manifest_path,
        output_path,
        package_name,
        variant,
        target,
        install,
        no_build,
        no_strip,
        separate_debug_symbols,
        quiet,
        fast,
        verbose,
        mut cargo_build_flags,
        deb_version,
    }: CliOptions,
) -> CDResult<()> {
    let target = target.as_deref();
    let variant = variant.as_deref();

    if install || target.is_none() {
        warn_if_not_linux(); // compiling natively for non-linux = nope
    }

    // `cargo deb` invocation passes the `deb` arg through.
    if cargo_build_flags.first().map_or(false, |arg| arg == "deb") {
        cargo_build_flags.remove(0);
    }

    // Listener conditionally prints warnings
    let mut listener_tmp1;
    let mut listener_tmp2;
    let listener: &mut dyn listener::Listener = if quiet {
        listener_tmp1 = listener::NoOpListener;
        &mut listener_tmp1
    } else {
        listener_tmp2 = listener::StdErrListener { verbose };
        &mut listener_tmp2
    };

    let manifest_path = manifest_path.as_ref().map_or("Cargo.toml", |s| s.as_str());
    let mut options = Config::from_manifest(
        Path::new(manifest_path),
        package_name.as_deref(),
        output_path,
        target,
        variant,
        deb_version,
        listener,
    )?;
    reset_deb_temp_directory(&options)?;

    if !no_build {
        cargo_build(&options, target, &cargo_build_flags, verbose)?;
    }

    options.resolve_assets()?;

    cargo_deb::data::compress_assets(&mut options, listener)?;

    if (options.strip || separate_debug_symbols) && !no_strip {
        strip_binaries(&mut options, target, listener, separate_debug_symbols)?;
    }

    // Obtain the current time which will be used to stamp the generated files in the archives.
    let system_time = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)?
        .as_secs();
    let mut deb_contents = DebArchive::new(&options)?;

    deb_contents.add_data("debian-binary", system_time, b"2.0\n")?;

    // Initailize the contents of the data archive (files that go into the filesystem).
    let (data_archive, asset_hashes) = data::generate_archive(&options, system_time, listener)?;
    let original = data_archive.len();

    let listener_tmp = &mut *listener; // reborrow for the closure
    let options = &options;
    let (control_compressed, data_compressed) = rayon::join(
        move || {
            // The control archive is the metadata for the package manager
            let control_archive =
                control::generate_archive(options, system_time, asset_hashes, listener_tmp)?;
            compress::xz_or_gz(&control_archive, fast)
        },
        move || compress::xz_or_gz(&data_archive, fast),
    );
    let control_compressed = control_compressed?;
    let data_compressed = data_compressed?;

    // Order is important for Debian
    deb_contents.add_data(
        &format!("control.tar.{}", control_compressed.extension()),
        system_time,
        &control_compressed,
    )?;
    drop(control_compressed);
    let compressed = data_compressed.len();
    listener.info(format!(
        "compressed/original ratio {}/{} ({}%)",
        compressed,
        original,
        compressed * 100 / original
    ));
    deb_contents.add_data(
        &format!("data.tar.{}", data_compressed.extension()),
        system_time,
        &data_compressed,
    )?;
    drop(data_compressed);

    let generated = deb_contents.finish()?;
    if !quiet {
        println!("{}", generated.display());
    }

    remove_deb_temp_directory(options);

    if install {
        install_deb(&generated)?;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn warn_if_not_linux() {}

#[cfg(not(target_os = "linux"))]
fn warn_if_not_linux() {
    eprintln!("warning: This command is for Linux only, and will not make sense when run on other systems");
}
