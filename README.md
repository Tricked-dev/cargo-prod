# Cargo-prod

package your rust application for linux distributions

## Installing

```sh
$ cargo install cargo-package
```

## Usage

you need to have [`appimagetool`](https://appimage.github.io/appimagetool/) installed(to bin) for appimages to work.

```sh
$ cargo package
```

## Credits

this project uses the source code of the following cargo crates

- [Cargo-aur](https://github.com/fosskers/cargo-aur) (MIT)
- [cargo-deb](https://github.com/kornelski/cargo-deb) (MIT)
- [Cargo-appimage](https://github.com/StratusFearMe21/cargo-appimage) (GPL-3.0)

Thanks for making this project possible!

## TODO

- add rpm support rpm is currently blocked by not being able to use the cargo-generate-rpm crate not having a lib.rs
- test the xbps template....
- generate gentoo build scripts
- create a zip when this package is used on windows
- create a tar.gz when used on macos

## License

This project is licensed under the GPL-3.0 License to comply with the cargo-appimage license, it will be changed to apache-2 if appimages.rs gets rewritten
