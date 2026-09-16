{
  description = "nx - A nix helper CLI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        nx = pkgs.callPackage ./package.nix { };
      in
      {
        packages = {
          default = nx;
          inherit nx;
        };

        apps = {
          default = flake-utils.lib.mkApp {
            drv = nx;
            exePath = "/bin/nx";
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
          ];
        };
      }
    );
}
