{ pkgs ? import <nixpkgs> {}}:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    rustfmt
    rust-analyzer
    clippy

    openssl
    
    vault
  ];

  RUST_BACKTRACE = 1;
}
