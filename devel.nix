{ pkgs }:

let
  rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
    extensions = [
      "clippy"
      "rust-src"
      "rustfmt"
    ];
  };
in
pkgs.mkShell {
  packages = [
    rustToolchain
  ];

  RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
}
