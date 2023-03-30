{pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    rustfmt
    rust-analyzer
    clippy
    pkgconfig
    udev
    libinput
  ];
  RUST_BACKTRACE = 1;
}
