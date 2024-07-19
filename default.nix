{
  lib,
  clangStdenv,
  rustPlatform,
  makeWrapper,
  pkg-config,
  rustfmt,
  cargo,
  rustc,
  fetchFromGitHub,
  runCommand,
  gn,
  ninja,
  removeReferencesTo,
  python3,
  fetchgit,
  linkFarm,
  fontconfig,
  llvmPackages,
}: let
  cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
in
  rustPlatform.buildRustPackage.override {stdenv = clangStdenv;} rec {
    pname = cargoToml.package.name;
    version = cargoToml.package.version;

    src = ./.;

    cargoLock = {
      lockFile = ./Cargo.lock;
    };

    buildInputs = [
      pkg-config
      fontconfig
      llvmPackages.libclang
      llvmPackages.libcxxClang
    ];
    checkInputs = [cargo rustc];

    LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";

    nativeBuildInputs = [
      makeWrapper
      pkg-config
      rustfmt
      rustc
      cargo
      removeReferencesTo
      python3 # for skia :)
    ];

    SKIA_SOURCE_DIR = let
      repo = fetchFromGitHub {
        owner = "rust-skia";
        repo = "skia";
        # see rust-skia:skia-bindings/Cargo.toml#package.metadata skia
        rev = "m114-0.62.1";
        sha256 = "sha256-w5dw/lGm40gKkHPR1ji/L82Oa808Kuh8qaCeiqBLkLw=";
      };
      externals = linkFarm "skia-externals" (lib.mapAttrsToList
        (name: value: {
          inherit name;
          path = fetchgit value;
        })
        (lib.importJSON ./skia-externals.json));
    in
      runCommand "source" {} ''
        cp -R ${repo} $out
        chmod -R +w $out
        ln -s ${externals} $out/third_party/externals
      '';

    SKIA_GN_COMMAND = "${gn}/bin/gn";
    SKIA_NINJA_COMMAND = "${ninja}/bin/ninja";

    doCheck = true;
    CARGO_BUILD_INCREMENTAL = "false";
    RUST_BACKTRACE = "full";

    postFixup = ''
      remove-references-to -t "$SKIA_SOURCE_DIR" \
        $out/bin/shadower
    '';

    disallowedReferences = [SKIA_SOURCE_DIR];

    meta = with lib; {
      description = "A simple CLI utility to add rounded borders, padding, and shadows to images.";
      homepage = "https://github.com/n3oney/shadower";
      license = with licenses; [gpl3];
      maintainers = [
        {
          email = "neo@neoney.dev";
          github = "n3oney";
          githubId = 30625554;
          name = "Michał Minarowski";
        }
      ];
      mainProgram = "shadower";
    };
  }
