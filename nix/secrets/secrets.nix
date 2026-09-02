let
  sharedKey = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIP4U52jGBSCW85mjmk7N+25/C8IZuYwja/xwOp4ZXt8M";
in
{
  "token-signing-key.age".publicKeys = [ sharedKey ];
  "state-signing-key.age".publicKeys = [ sharedKey ];
  "globus-client-id.age".publicKeys = [ sharedKey ];
  "globus-client-secret.age".publicKeys = [ sharedKey ];
  "tacc-resource-provider-client-id.age".publicKeys = [ sharedKey ];
  "tacc-resource-provider-client-secret.age".publicKeys = [ sharedKey ];
}
