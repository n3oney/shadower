{
  description = "A simple CLI utility to add rounded borders, padding, and shadows to images.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    naersk = {
      url = "github:nmattia/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    naersk,
  }: let
    cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
    supportedSystems = ["x86_64-linux" "aarch64-linux"];
    forAllSystems = f: nixpkgs.lib.genAttrs supportedSystems (system: f system);
  in {
    overlay = final: prev: {
      "${cargoToml.package.name}" = final.callPackage ./. {};
    };

    packages = forAllSystems (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [self.overlay];
      };
    in {
      "${cargoToml.package.name}" = pkgs."${cargoToml.package.name}";
    });

    defaultPackage = forAllSystems (system:
      (import nixpkgs {
        inherit system;
        overlays = [self.overlay];
      })
      ."${cargoToml.package.name}");

    checks = forAllSystems (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          self.overlay
        ];
      };
    in {
      format =
        pkgs.runCommand "check-format"
        {
          buildInputs = with pkgs; [rustfmt cargo];
        } ''
          ${pkgs.rustfmt}/bin/cargo-fmt fmt --manifest-path ${./.}/Cargo.toml -- --check
          ${pkgs.nixpkgs-fmt}/bin/nixpkgs-fmt --check ${./.}
          touch $out # it worked!
        '';
      "${cargoToml.package.name}" = pkgs."${cargoToml.package.name}";
    });
    devShell = forAllSystems (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [self.overlay];
      };
    in
      pkgs.mkShell.override { stdenv = pkgs.clang15Stdenv; } {
        inputsFrom = [
          pkgs."${cargoToml.package.name}"
        ];
        NIX_LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
          pkgs.clangStdenv.cc.cc
          pkgs.openssl
          #pkgs.skia
        ];

        NIX_LD = "/run/current-system/sw/share/nix-ld/lib/ld.so";
        SKIA_NINJA_COMMAND = "${pkgs.ninja}/bin/ninja";
        SKIA_GN_COMMAND = "${pkgs.gn}/bin/gn";
        
        buildInputs = with pkgs; [
          rustfmt
          rust-analyzer
          nixpkgs-fmt
          ninja
          clang
        ];
        LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
      });
  };
}
