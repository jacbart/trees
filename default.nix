{
  pkgs ? import <nixpkgs> { },
  rustVersion,
  self,
  version,
  pname,
  ...
}:
let
  inherit (pkgs) lib;
  outputHashes = {
    "ff-1.0.3" = "sha256-GqylWMGjcmizQulaoIlMDKVULWIYU+v7FUHFHal6NBo=";
  };
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
    git
  ];

  nativeBuildInputs = with pkgs; [
    git
    pkg-config
  ];

  cargoLock = {
    lockFile = ./Cargo.lock;
    inherit outputHashes;
  };
  meta = with lib; {
    inherit pname;
    description = "git worktrees simplifed";
    homepage = "https://github.com/jacbart/trees";
    license = with licenses; [ mit ];
    maintainers = with maintainers; [ jacbart ];
  };
}
