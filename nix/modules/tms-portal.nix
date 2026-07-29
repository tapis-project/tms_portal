{ flake-parts-lib, ... }:
{
  options.perSystem = flake-parts-lib.mkPerSystemOption
    ({ lib, ... }: {
      options = {
        tms-portal = {
          git_url = lib.mkOption {
            type = lib.types.str;
            default = "https://github.com/tapis-project/tms_portal";
            description = "URL for the remote Git repository";
          };
          process-compose-server = {
            enable = lib.mkOption {
              type = lib.types.bool;
              default = true;
              description = "Whether to enable the server for process compose";
            };
            port = lib.mkOption {
              type = lib.types.port;
              default = 1234;
              description = "Port for process compose server";
            };
          };
        };
      };
    });
  config.perSystem = { lib, pkgs, config, ... }:
    let
      tms-portal =
        let
          src =
            let
              migrationsFilter = path: _type: builtins.match ".*sql$" path != null;
            in
            lib.cleanSourceWith {
              src = ./../..;
              filter = path: type: (migrationsFilter path type) || (config.rust.craneLib.filterCargoSources path type);
              name = "source";
            };
          commonArgs = {
            inherit src;
            version = "0.1.0";
            name = "tms-portal";
            pname = "tms-portal";
            buildInputs = with pkgs; [
              pkg-config
              sqlx-cli
              openssl
              git
            ] ++ lib.optionals pkgs.stdenv.isDarwin [ pkgs.libiconv ];
          };
          cargoArtifacts = config.rust.craneLib.buildDepsOnly commonArgs;
        in
        config.rust.craneLib.buildPackage (commonArgs //
          {
            inherit cargoArtifacts;
            meta = {
              description = "TMS Portal";
              mainProgram = "tms_portal";
            };
          });
      wrapped-tms-portal = pkgs.stdenv.mkDerivation {
        name = "tms-portal";
        nativeBuildInputs = [ pkgs.makeWrapper ];
        dontUnpack = true;
        installPhase = ''
          mkdir -p $out/bin
          makeWrapper ${tms-portal}/bin/tms_portal $out/bin/tms-portal \
            --set TMS_PORTAL_DB_HOST "${config.tms-portal.postgres.address}" \
            --set TMS_PORTAL_DB_PORT "${toString config.tms-portal.postgres.port}" \
            --set TMS_PORTAL_DB_PASSWORD "${config.tms-portal.postgres.password}"
        '';
      };
    in
    {
      config = {
        packages = {
          inherit tms-portal wrapped-tms-portal;
        };
      };
    };
}
