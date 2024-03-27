#!/bin/bash

set -e

if [ ! $INSTALL_DIR ]
then 
    INSTALL_DIR=/bin
fi

echo "Building cryptboot"
cargo build --release

echo "Do you want to install cryptboot in $INSTALL_DIR ? [y|n]"
read confirm

if [[ $(grep -iP "(yes|y)" <<<$confirm) ]]
then
    sudo cp ./target/release/cryptboot $INSTALL_DIR
fi
