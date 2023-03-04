{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  # rustup
  packages = with pkgs; [ terraform jq protobuf rustc rustfmt rust-analyzer cargo grpcurl ];
}

