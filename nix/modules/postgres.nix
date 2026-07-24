{ inputs, ... }:
{
  perSystem = {self', ...}: {
    process-compose."postgres" = {
      imports = [
        inputs.services-flake.processComposeModules.default
      ];
      settings.processes.hello.command = "sleep 1000000";
      cli.options.port = 8080;
      cli.options.no-server = false;
      services.postgres."pg" = { name, ...}: {
        enable = true;
        dataDir = "./.data/${name}";
      };
    };
  };
}
