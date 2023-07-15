# track-work

track-work is a command line utility to track the time spend working on different projects.

Currently only windows is supported. If you need support for any other operating system, feel 
free to raise an issue and or pull request.

Features:
- Configure project lists
- Map project to clients (or billing elements)
- Automatically switch the active project based on window title prefixes
- Shows a weekly report on hours spend per client/billing element.
- Data is stored on disk as a set of JSON files (one per week)
- Use hotkeys for everything (inspired by [k9s](https://github.com/derailed/k9s))
- Lightweight (less than <2MB of memory and no measurable cpu usage)

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Contributions of any kind are welcome and encouraged!

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

Commit messages must follow [Conventional Commits Specification](https://www.conventionalcommits.org/en/v1.0.0/)

## Build instructions

- run `cargo run`
- build `cargo build`
- release `cargo build -r`
- lint `cargo clippy`
