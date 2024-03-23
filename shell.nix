{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  # rustup
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

  packages = with pkgs; [ bashInteractive terraform jq protobuf rustc rustfmt rust-analyzer cargo clippy grpcurl vscode ];
}
