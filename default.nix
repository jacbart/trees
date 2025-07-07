{ pkgs ? import <nixpkgs> {}
, rustVersion
, self
, version
, pname
, ... }:
let
  inherit (pkgs) lib;
  # outputHashes = { "package-version" = "sha256-xxx"; };
  rustPlatform = pkgs.makeRustPlatform {
    cargo = rustVersion;
    rustc = rustVersion;
  };
in
rustPlatform.buildRustPackage {
  inherit pname version;
  src = lib.cleanSource self;

  buildInputs = with pkgs; [
    openssl
    # rustls-libssl
  ];

  nativeBuildInputs = with pkgs; [
    pkg-config
  ];
  
  cargoLock = {
    lockFile = ./Cargo.lock;
    # inherit outputHashes;
  };
  meta = with lib; {
    inherit pname;
    description = "git worktrees simplifed";
    homepage = "https://github.com/jacbart/trees";
    license = with licenses; [ mpl20 ];
    maintainers = with maintainers; [ jacbart ];
  };
}
