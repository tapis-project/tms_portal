{ inputs, flake-parts-lib, ... }:
{
  options = {
    perSystem = flake-parts-lib.mkPerSystemOption ({ lib, ... }: {
      options = {
        tms-portal.postgres = {
          port = lib.mkOption {
            type = lib.types.port;
            default = 5432;
            description = "Port for local Postgres server";
          };
          address = lib.mkOption {
            type = lib.types.str;
            default = "0.0.0.0";
            description = "Address where Postgres server listens";
          };
          user = lib.mkOption {
            type = lib.types.str;
            default = "tms_portal_user";
            description = "User for TMS Portal in Postgres";
          };
          password = lib.mkOption {
            type = lib.types.str;
            default = "tms_portal_password";
            description = "Password for TMS Portal in Postgres";
          };
          database = lib.mkOption {
            type = lib.types.str;
            default = "tms_portal_db";
            description = "Database for TMS Portal in Postgres";
          };
          admin_user = lib.mkOption {
            type = lib.types.str;
            default = "postgres";
            description = "Admin user for Postgres";
          };
          admin_password = lib.mkOption {
            type = lib.types.str;
            default = "pg_admin_pass";
            description = "Password for admin user for Postgres";
          };
          admin.port = lib.mkOption {
            type = lib.types.port;
            default = 5050;
            description = "Port for Web Postgres Admin interface";
          };
          admin.email = lib.mkOption {
            type = lib.types.str;
            default = "admin@local";
            description = "Login email for Web Postgres Admin interface";
          };
          admin.password = lib.mkOption {
            type = lib.types.str;
            default = "password";
            description = "Password for Web Postgres Admin interface";
          };
        };
      };
    });
  };
  config = {
    perSystem = { pkgs, config, ... }: {
      process-compose."postgres-stack" = {
        imports = [
          inputs.services-flake.processComposeModules.default
        ];
        settings.processes.hello.command = "sleep 1000000";
        cli.options.port = config.tms-portal.process-compose-server.port;
        cli.options.no-server = !config.tms-portal.process-compose-server.enable;
        cli.environment.PC_DISABLE_TUI = true;
        services.postgres."pg" = { name, ... }: {
          enable = true;
          dataDir = "./.data/${name}";
          port = config.tms-portal.postgres.port;
          listen_addresses = config.tms-portal.postgres.address;
          superuser = config.tms-portal.postgres.admin_user;
          initialDatabases = [
            { name = config.tms-portal.postgres.database; }
          ];
          initialScript = {
            after = with config.tms-portal.postgres; ''
              CREATE USER ${user} with encrypted password '${password}';
              ALTER DATABASE ${database} OWNER TO ${user};
            '';
          };
        };
        services.pgadmin."pgadm" = { name, ... }: {
          dataDir = "./.data/${name}";
          enable = true;
          package = pkgs.pgadmin4;
          port = config.tms-portal.postgres.admin.port;
          initialEmail = config.tms-portal.postgres.admin.email;
          initialPassword = config.tms-portal.postgres.admin.password;
        };
      };
    };
  };
}
