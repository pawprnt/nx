{ pkgs, lib, rustPlatform }:

rustPlatform.buildRustPackage {
  pname = "nx";
  version = "0.1.0";
  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;

  nativeBuildInputs = with pkgs; [
    pkg-config
  ];

  buildInputs = with pkgs; [
    openssl
  ];

  meta = with lib; {
    description = "A nix helper CLI";
    homepage = "https://github.com/nix-community/nx";
    license = licenses.mit;
    maintainers = [ ];
    platforms = platforms.all;
    mainProgram = "nx";
  };
}
