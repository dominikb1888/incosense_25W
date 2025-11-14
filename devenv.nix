{ pkgs, lib, config, inputs, ... }:

{
  # https://devenv.sh/basics/
  env.GREET = "devenv";

  # https://devenv.sh/packages/
  packages = [
    pkgs.git
    pkgs.openssl
    pkgs.llvm
    pkgs.cargo-watch
    pkgs.cargo-tarpaulin
    pkgs.clippy
    pkgs.rustfmt
    pkgs.sqlx-cli
    pkgs.cargo-audit
  ]++ lib.optionals pkgs.stdenv.isDarwin [
    pkgs.libiconv
  ];

  # https://devenv.sh/languages/
  languages.rust.enable = true;

  # https://devenv.sh/processes/
  # processes.dev.exec = "${lib.getExe pkgs.watchexec} -n -- ls -la";

  # https://devenv.sh/services/

  services.postgres = {
    enable = true;
    listen_addresses = "127.0.0.1";
    port = 5432;
    initialScript = "CREATE ROLE postgres SUPERUSER;";
    initialDatabases = [ { name = "incosense"; } ];
  };
  #
  # # https://devenv.sh/processes/
  # processes.backend.exec = "cargo build --release && cargo run";
  #
  # containers."prod".name = "incosense_class";
  # containers."prod".copyToRoot = ./target/release;
  # containers."prod".startupCommand = "/incosense";
  #
  # https://devenv.sh/scripts/
  scripts.hello.exec = ''
    echo hello from $GREET
  '';

  # https://devenv.sh/basics/
  enterShell = ''
    hello         # Run scripts directly
    git --version # Use packages
  '';

  # https://devenv.sh/tasks/
  # tasks = {
  #   "myproj:setup".exec = "mytool build";
  #   "devenv:enterShell".after = [ "myproj:setup" ];
  # };

  # # https://devenv.sh/tests/
  # enterTest = ''
  #   echo "Running tests"
  #   git --version | grep --color=auto "${pkgs.git.version}"
  # '';

  # https://devenv.sh/git-hooks/
  # git-hooks.hooks.shellcheck.enable = true;
# https://devenv.sh/pre-commit-hooks/

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
    # some hooks provide settings
    clippy.settings.allFeatures = true;
    rustfmt.enable = true;
    };
  };

  # devcontainer.enable = true;
  # # See full reference at https://devenv.sh/reference/options/
}
