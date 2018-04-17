#!/usr/bin/env sh
if [ `id -u` != 0 ]; then
    echo "Run the script as root/sudo!"
    exit 1
fi

REPO_USERNAME=guangie88
REPO_NAME=terraform-zap

LATEST_TAG=$(curl -sSf "https://api.github.com/repos/$REPO_USERNAME/$REPO_NAME/releases/latest" \
    | grep tag_name \
    | sed -n 's/.*"\(v.*\)".*/\1/p')

BINARY_FILE=terraform-zap
ZIP_SUFFIX=`uname -s | tr '[:upper:]' '[:lower:]'`-`uname -i`
ZIP_FILE=$BINARY_NAME-$TRAVIS_TAG-$ZIP_SUFFIX.zip

BIN_DIR=/usr/local/bin

# unzip cannot work on Unix pipe
echo "Downloading '"$ZIP_FILE"'..."
curl -sSfLO "https://github.com/guangie88/terraform-zap/releases/download/$LATEST_TAG/$ZIP_FILE"

echo "Unzipping..."
unzip -qq "$ZIP_FILE" 
rm "$ZIP_FILE"

echo "Moving binary file '"$BINARY_FILE"' to $BIN_DIR/..."
mv $BINARY_FILE $BIN_DIR/
chmod +x $BIN_DIR/$BINARY_FILE

echo "DONE!\n\nterraform-zap requires terraform, be sure to place terraform in PATH.\n"
