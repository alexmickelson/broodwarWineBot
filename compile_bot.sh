#!/usr/bin/env nix-shell
#!nix-shell -i bash -p gnumake pkgsCross.mingw32.stdenv.cc

make
