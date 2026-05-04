{
  description = "Slideshow Stack";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    zig-overlay = {
      url = "github:mitchellh/zig-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, zig-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustVersion = "1.93.0";
        toolchain = (pkgs.rust-bin.stable."${rustVersion}".default.override {
          targets = [
            "x86_64-pc-windows-gnu"
            "x86_64-pc-windows-msvc"
            "armv7-unknown-linux-gnueabihf"
          ];
        });
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            toolchain
            pkgs.rust-bin.stable.latest.default

            # Node.js build tool
            pkgs.bun

            # Native dependencies for Rust crates
            pkgs.pkg-config
            zig-overlay.packages.${system}."0.15.2"
          ];

          shellHook = ''
            echo "Slideshow Stack Development Shell"
            rustc --version
            cargo --version
          '';
        };
      }
    );
}
