{
  description = "A Discord Bot to Remotely Control Different Things";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    crane,
    ...
  }: let
    workspaceToml = builtins.fromTOML (builtins.readFile "${self}/Cargo.toml");
    cargoToml = builtins.fromTOML (builtins.readFile "${self}/crates/server/Cargo.toml");
    name = workspaceToml.workspace.metadata.crane.name;
    version = cargoToml.package.version;
  in
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [rust-overlay.overlays.default];
      };
      lib = pkgs.lib;

      rustToolchain = (pkgs.rust-bin.fromRustupToolchainFile (self + /rust-toolchain.toml)).override {
        extensions = ["rust-src" "rust-analyzer" "clippy"];
      };
      craneLib = crane.mkLib pkgs;

      unfilteredRoot = ./.;
      src = lib.fileset.toSource {
        root = unfilteredRoot;
        fileset = lib.fileset.unions [
          ./assets
          ./migrations
          ./tailwind.config.js
          (craneLib.fileset.commonCargoSources unfilteredRoot)
        ];
      };

      tailwindcss = pkgs.nodePackages.tailwindcss.overrideAttrs (_: {
        plugins = [
          pkgs.nodePackages."@tailwindcss/aspect-ratio"
          pkgs.nodePackages."@tailwindcss/forms"
          pkgs.nodePackages."@tailwindcss/language-server"
          pkgs.nodePackages."@tailwindcss/line-clamp"
          pkgs.nodePackages."@tailwindcss/typography"
        ];
      });

      craneBuild = rec {
        args = {
          inherit src name version;
          buildInputs = [
            pkgs.binaryen
            pkgs.cargo-leptos
            pkgs.libiconv
            pkgs.lld
            pkgs.openssl
            pkgs.pkg-config
            pkgs.sqlx-cli
            tailwindcss
          ];
          env = {
            DATABASE_URL = "sqlite://wallpapers.db";
          };
          preBuild = ''
            echo "Running build-time SQLX migrations for schema reflection..."
            sqlx database setup --source ${src}/migrations
            echo "Build-time migrations complete."
          '';
        };
        cargoArtifacts = craneLib.buildDepsOnly args;
        buildArgs =
          args
          // {
            inherit cargoArtifacts;
            buildPhaseCargoCommand = "cargo leptos build --release -vvv";
            cargoTestCommand = "cargo leptos test --release -vvv";
            nativeBuildInputs = [pkgs.makeWrapper];
            cargoExtraArgs = "";
            doNotPostBuildInstallCargoBinaries = true;
            installPhaseCommand = ''
              mkdir -p $out/bin
              cp target/release/${name}-server $out/bin/${name}-server
              cp -r target/site $out/bin/site
              wrapProgram $out/bin/${name}-server \
                --set LEPTOS_SITE_ROOT $out/bin/site
            '';
          };
        package = craneLib.buildPackage buildArgs;
        check = craneLib.cargoClippy (args
          // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets --all-features";
          });
        doc = craneLib.cargoDoc (args // {inherit cargoArtifacts;});
      };

      devShell = pkgs.mkShell {
        nativeBuildInputs = [rustToolchain];
        buildInputs = [
          pkgs.binaryen
          pkgs.cargo-leptos
          pkgs.git
          pkgs.leptosfmt
          pkgs.libiconv
          pkgs.openssl
          pkgs.pkg-config
          pkgs.sqlite
          pkgs.sqlx-cli
          tailwindcss
        ];
        shellHook = ''
          local flake_root=$(git rev-parse --show-toplevel 2>/dev/null || echo "$PWD")
          export DATABASE_URL="sqlite://$flake_root/wallpapers.db"
          export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"
        '';
      };
    in {
      apps.default = {
        type = "app";
        program = "${self.packages.${system}.default}/bin/${name}-server";
      };
      packages.default = craneBuild.package;
      packages."${name}-doc" = craneBuild.doc;
      checks."${name}-clippy" = craneBuild.check;
      devShells.default = devShell;
    })
    // {
      nixosModules.default = {
        lib,
        config,
        pkgs,
        ...
      }: let
        workingDir = "/var/lib/remote-bot";
        runtimeDbPath = "${workingDir}/wallpapers.db";
        settingsFile = "${workingDir}/settings.toml";
        cfg = config.services.remote-bot;
      in {
        options.services.remote-bot = {
          enable = lib.mkEnableOption "Enable the remote-bot service.";
          environmentFile = lib.mkOption {
            type = lib.types.nullOr lib.types.path;
            default = null;
            description = "Path to an environment file to keep secrets out of the nix store.";
          };
          address = lib.mkOption {
            type = lib.types.str;
            default = "0.0.0.0";
            description = "Bind address for the HTTP server.";
          };
          port = lib.mkOption {
            type = lib.types.port;
            default = 3000;
            description = "Port for the HTTP server.";
          };
          settings = {
            discord_token = lib.mkOption {
              type = lib.types.nullOr lib.types.str;
              default = null;
              description = "Discord token";
            };
            recipient_email = lib.mkOption {
              type = lib.types.nullOr lib.types.str;
              default = null;
              description = "Recipient email";
            };
            sender_domain = lib.mkOption {
              type = lib.types.nullOr lib.types.str;
              default = null;
              description = "Sender domain";
            };
            smtp_password = lib.mkOption {
              type = lib.types.nullOr lib.types.str;
              default = null;
              description = "SMTP password";
            };
            smtp_server = lib.mkOption {
              type = lib.types.nullOr lib.types.str;
              default = null;
              description = "SMTP server";
            };
            smtp_username = lib.mkOption {
              type = lib.types.nullOr lib.types.str;
              default = null;
              description = "SMTP username";
            };
            timezone = lib.mkOption {
              type = lib.types.nullOr lib.types.str;
              default = null;
              description = "Timezone";
            };
          };
        };

        config = lib.mkIf cfg.enable {
          assertions = [
            {
              assertion =
                cfg.environmentFile
                != null
                || (
                  cfg.settings.discord_token
                  != null
                  && cfg.settings.recipient_email != null
                  && cfg.settings.sender_domain != null
                  && cfg.settings.smtp_password != null
                  && cfg.settings.smtp_server != null
                  && cfg.settings.smtp_username != null
                  && cfg.settings.timezone != null
                );
              message = "Service remote-bot requires all settings to be explicitly set unless an environmentFile is specified.";
            }
          ];

          users.users.remote-bot = {
            description = "User for remote-bot service";
            group = "remote-bot";
            home = workingDir;
            createHome = true;
            isSystemUser = true;
          };
          users.groups.remote-bot = {};

          systemd.services.remote-bot = {
            description = "Remote Bot Service";
            after = ["network.target"];
            wantedBy = ["multi-user.target"];
            serviceConfig = {
              Type = "simple";
              User = config.users.users.remote-bot.name;
              Group = config.users.users.remote-bot.group;
              WorkingDirectory = workingDir;
              ExecStart = "${self.packages.${pkgs.system}.default}/bin/${name}-server";
              Restart = "on-failure";
              Environment = [
                "LEPTOS_SITE_ADDR=${cfg.address}:${toString cfg.port}"
                "DATABASE_URL=sqlite://${runtimeDbPath}"
              ];
              inherit (lib.optionalAttrs (cfg.environmentFile != null) {EnvironmentFile = cfg.environmentFile;}) EnvironmentFile;

              CapabilityBoundingSet = [""];
              LockPersonality = true;
              NoNewPrivileges = true;
              PrivateDevices = true;
              PrivateTmp = true;
              ProcSubset = "pid";
              ProtectClock = true;
              ProtectControlGroups = true;
              ProtectHome = true;
              ProtectHostname = true;
              ProtectKernelLogs = true;
              ProtectKernelModules = true;
              ProtectKernelTunables = true;
              ProtectSystem = "strict";
              ReadWritePaths = [workingDir];
              RemoveIPC = true;
              RestrictAddressFamilies = ["AF_INET" "AF_INET6" "AF_UNIX"];
              RestrictNamespaces = true;
              RestrictRealtime = true;
              RestrictSUIDSGID = true;
              SystemCallArchitectures = "native";
              SystemCallFilter = ["@system-service" "~@resources" "~@privileged"];
            };
            preStart = let
              format = pkgs.formats.toml {};
              settingsWithOutNull = lib.filterAttrsRecursive (name: value: value != null) cfg.settings;
              config = format.generate "settings.toml" settingsWithOutNull;
            in ''
              mkdir -p ${workingDir}
              ln -sf ${config} ${settingsFile}
              touch ${runtimeDbPath}
              echo "Starting ${name}-server service..."
            '';
          };
        };
      };
    };
}
