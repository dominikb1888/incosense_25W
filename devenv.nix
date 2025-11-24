{ lib, config, inputs, ... }:

let
  # Local packages
  pkgs = inputs.nixpkgs.legacyPackages.aarch64-darwin;

  # Database config entirely from environment
  dbHost = builtins.getEnv "APP__DATABASE__HOST";
  dbPort = builtins.getEnv "APP__DATABASE__PORT";
  dbUser = builtins.getEnv "APP__DATABASE__USERNAME";
  dbPass = builtins.getEnv "APP__DATABASE__PASSWORD";
  dbName = builtins.getEnv "APP__DATABASE__DATABASE_NAME";
in
{
  # --------------------------
  # Environment
  # --------------------------
  dotenv.enable = true;

  env.APP__DATABASE__HOST          = dbHost;
  env.APP__DATABASE__PORT          = dbPort;
  env.APP__DATABASE__USERNAME      = dbUser;
  env.APP__DATABASE__PASSWORD      = dbPass;
  env.APP__DATABASE__DATABASE_NAME = dbName;

  # --------------------------
  # Dev packages (restored)
  # --------------------------
  packages = [
    pkgs.git
    pkgs.rainfrog
    pkgs.openssl
    pkgs.llvm
    pkgs.cargo-chef
    pkgs.cargo-watch
    pkgs.cargo-tarpaulin
    pkgs.clippy
    pkgs.evcxr
    pkgs.rustfmt
    pkgs.sqlx-cli
    pkgs.python3
    pkgs.cargo-audit
  ] ++ lib.optionals pkgs.stdenv.isDarwin [
    pkgs.libiconv
  ];

  # --------------------------
  # Rust language
  # --------------------------
  languages.rust.enable = true;

  # --------------------------
  # Scripts
  # --------------------------
  scripts.hello.exec = ''
    echo hello from $GREET
  '';

  # --------------------------
  # Services: only Postgres
  # --------------------------
  services.postgres = {
    enable = true;
    listen_addresses = "127.0.0.1";
    port = 5432;

    # Initialize role and database dynamically using env variables
    initialScript = ''
    CREATE ROLE ${dbUser} SUPERUSER;
    CREATE DATABASE ${dbName};
    '';

    initialDatabases = [ { name = dbName; } ];
  };

  # --------------------------
  # Git hooks
  # --------------------------
  git-hooks = {
    enable = true;
    hooks = {
      cargo-check.enable = true;

      cargo-test = {
        enable = true;
        entry = "bash -c 'cd $(git rev-parse --show-toplevel) && cargo test --workspace --all-targets'";
        language = "system";
        pass_filenames = false;
      };

      clippy.enable = true;
      clippy.packageOverrides.cargo = pkgs.cargo;
      clippy.packageOverrides.clippy = pkgs.clippy;
      clippy.settings.allFeatures = true;

      rustfmt.enable = true;
    };
  };

  # --------------------------
  # Enter shell
  # --------------------------
  enterShell = ''
    hello
    git --version
  '';
}

