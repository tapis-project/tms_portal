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
}
