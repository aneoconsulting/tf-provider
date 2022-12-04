{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
   packages = with pkgs; [ terraform jq protobuf rustup grpcurl ];
}

