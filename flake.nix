{
  description = "dangreco/winfidel-rs environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    flake-parts.url = "github:hercules-ci/flake-parts";
    git-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ inputs.git-hooks.flakeModule ];
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
      perSystem =
        {
          lib,
          config,
          system,
          pkgs,
          ...
        }:
        let
          pkgs' = import inputs.nixpkgs {
            inherit system;
            overlays = [ (import inputs.rust-overlay) ];
          };

          rust = rec {
            version = "1.93.0";
            package = pkgs'.rust-bin.stable.${version}.default.override {
              extensions = [
                "rust-src"
                "clippy"
                "rustfmt"
                "rust-analyzer"
              ];
              targets = [ "riscv32imc-unknown-none-elf" ];
            };
          };
        in
        {
          _module.args.pkgs = pkgs';

          pre-commit.settings.hooks = {
            nixfmt.enable = true;
            yamlfmt.enable = true;
            yamllint.enable = true;
            taplo.enable = true;
            _rustfmt = {
              enable = true;
              name = "rusfmt";
              files = "\\.rs$";
              entry = "${rust.package}/bin/cargo fmt --all";
              pass_filenames = false;
            };
            _clippy = {
              enable = true;
              name = "clippy";
              files = "\\.rs$";
              entry = "${rust.package}/bin/cargo clippy --offline --all-features -- -D warnings";
              pass_filenames = false;
            };
          };

          devShells = {
            default =
              let
                __zed = pkgs.writers.writeJSON "settings.json" {
                  lsp.rust-analyzer.binary.path = "${rust.package}/bin/rust-analyzer";
                  lsp.rust-analyzer.initialization_options.cargo = {
                    allTargets = false;
                    target = "riscv32imc-unknown-none-elf";
                  };
                };
              in
              pkgs.mkShell {
                packages =
                  with pkgs;
                  [
                    nil
                    nixd
                    nixfmt
                    go-task
                    espflash
                    probe-rs-tools
                    esp-generate
                  ]
                  ++ config.pre-commit.settings.enabledPackages;

                buildInputs = [ rust.package ];
                nativeBuildInputs = with pkgs; [ openssl ];

                shellHook = ''
                  mkdir -p .zed
                  ln -sf ${__zed} .zed/settings.json
                  ${config.pre-commit.shellHook}
                '';
              };

            ci = pkgs.mkShell {
              packages =
                with pkgs;
                [
                  go-task
                ]
                ++ config.pre-commit.settings.enabledPackages;
              buildInputs = with pkgs; [ rust.package ];
              nativeBuildInputs = with pkgs; [ openssl ];
              shellHook = ''
                ${config.pre-commit.shellHook}
              '';
            };
          };
        };
    };
}
