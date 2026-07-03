#!/bin/bash
# Check server GLIBC version
# Run this on your GCP server to see what version you have

echo "GLIBC version:"
ldd --version | head -1

echo -e "\nOS version:"
cat /etc/os-release | grep -E "^(NAME|VERSION)="
