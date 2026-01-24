#!/bin/bash

# src下の全てをビルドするスクリプト

set -e

echo "Building rs_maze_maker project..."

# ライブラリのビルド
echo "Building library..."
cargo build --lib

# 全てのバイナリをビルド
echo "Building all binaries..."
cargo build --bins

# リリースビルド（オプション）
if [[ "$1" == "--release" ]]; then
    echo "Building in release mode..."
    cargo build --release --all
fi

echo "Build completed successfully!"
