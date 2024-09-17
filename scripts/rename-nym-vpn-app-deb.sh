#!/bin/bash

# Ensure the input .deb file is provided
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <deb-file>"
    exit 1
fi

deb_file="$1"

# Check if the file exists
if [ ! -f "$deb_file" ]; then
    echo "File not found: $deb_file"
    exit 1
fi

# Extract the package name and version from the filename
filename=$(basename "$deb_file")
version=$(echo "$filename" | sed -n 's/nym-vpn_\([0-9a-zA-Z.-]*\)_.*\.deb/\1/p')

if [ -z "$version" ]; then
    echo "Could not extract version from filename"
    exit 1
fi

# Create a temporary directory for extraction
tmpdir=$(mktemp -d)

# Extract the .deb package contents
dpkg-deb -R "$deb_file" "$tmpdir"

# Update the control file: change the Package name from nym-vpn to nym-vpn-app
control_file="$tmpdir/DEBIAN/control"
sed -i 's/Package: nym-vpn/Package: nym-vpn-app/' "$control_file"

# Repack the .deb package with the updated control file
new_deb_file="nym-vpn-app_${version}_amd64.deb"
dpkg-deb -Zxz -b "$tmpdir" "$new_deb_file"

# Clean up the temporary directory
#rm -rf "$tmpdir"

echo "Renamed package created: $new_deb_file"
