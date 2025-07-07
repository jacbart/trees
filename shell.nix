{ pkgs ? import <nixpkgs> {}
, pname
, rustVersion
, version
, ... }:

pkgs.mkShell {
  name = "${pname}-${version}";
  buildInputs = with pkgs; [
    (rustVersion.override { extensions = [ "rust-src" ]; })
    bacon
    pkg-config
    openssl
    rust-analyzer
  ];
  RUST_LOG = "debug";
  RUST_BACKTRACE = 1;
}
