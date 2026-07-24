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
        };
      };
    });
}
