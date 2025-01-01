# WARNING DO NOT USE

with (import <nixpkgs> { });
mkShell {
  buildInputs = [

    fontconfig
    glib
    libxkbcommon
    openssl_3
    pkg-config
    vulkan-tools
    wayland-scanner
    xorg.libxcb
    glib
    openssl_3
    vulkan-headers
    vulkan-loader
    wayland
    wayland-protocols

    udev
    alsa-lib
    vulkan-loader
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr # To use the x11 feature
    libxkbcommon
    wayland
  ];
}
