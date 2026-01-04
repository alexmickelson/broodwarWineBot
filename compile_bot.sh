#!/usr/bin/env nix-shell
#!nix-shell -i bash -p gnumake pkgsCross.mingwW64.stdenv.cc

make
