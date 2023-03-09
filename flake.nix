{
  description = "A very basic flake";
  inputs = {
    esp32-rust.url = "github:thiskappaisgrey/nixpkgs-esp-dev-rust";
    # flake-utils.url = "github:numtide/flake-utils";
   
  };
  outputs = { self, esp32-rust }: {

    # packages.x86_64-linux.hello = nixpkgs.legacyPackages.x86_64-linux.hello;

    # packages.x86_64-linux.default = self.packages.x86_64-linux.hello;
    devShell = {
      # TODO need to be able to add packages to this devshell..
      x86_64-linux = esp32-rust.devShells.x86_64-linux.esp32s2-idf-rust;
    };

  };
}
