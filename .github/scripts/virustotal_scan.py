#!/usr/bin/env python3
"""
VirusTotal Scanner with 409 Error Handling

This script uploads files to VirusTotal for malware scanning.
When a file already exists (409 error), it fetches the existing report instead.

Usage:
    python virustotal_scan.py <api_key> <file1> <file2> ...

Environment Variables:
    VT_REQUEST_RATE: Max requests per minute (default: 4)

Output:
    Prints analysis results in format: file1=url1,file2=url2,...
    Sets GitHub Actions output variable 'analysis' with the same format
"""

import sys
import os
import time
import hashlib
import requests
from pathlib import Path

MAX_RATE_LIMIT_RETRIES = 3


def compute_sha256(file_path):
    """Compute SHA256 hash of a file."""
    sha256_hash = hashlib.sha256()
    with open(file_path, "rb") as f:
        for byte_block in iter(lambda: f.read(4096), b""):
            sha256_hash.update(byte_block)
    return sha256_hash.hexdigest()


def get_upload_url(api_key, retry_count=0):
    """Get the upload URL required by VirusTotal for files larger than 32 MiB."""
    url = "https://www.virustotal.com/api/v3/files/upload_url"
    headers = {"x-apikey": api_key}

    try:
        response = requests.get(url, headers=headers, timeout=30)
        if response.status_code == 200:
            upload_url = response.json().get("data")
            if upload_url:
                return upload_url
            print("  ✗ Upload URL response did not contain a URL")
            return None
        if response.status_code == 429:
            if retry_count >= MAX_RATE_LIMIT_RETRIES:
                print("  ✗ Rate limit retry exhausted while getting upload URL")
                return None
            print("  ⚠ Rate limit exceeded (429). Waiting 60 seconds...")
            time.sleep(60)
            return get_upload_url(api_key, retry_count + 1)

        print(f"  ✗ Could not get large-file upload URL: HTTP {response.status_code}")
        print(f"  Response: {response.text}")
        return None
    except requests.exceptions.RequestException as e:
        print(f"  ✗ Upload URL request failed: {e}")
        return None


def upload_file(api_key, file_path, retry_count=0):
    """
    Upload file to VirusTotal or fetch existing report.

    Returns:
        str: Analysis URL or None on error
    """
    file_size = os.path.getsize(file_path)
    if file_size > 32 * 1024 * 1024:
        url = get_upload_url(api_key)
        if not url:
            return None
        print(f"  Using large-file upload endpoint for {file_size} bytes")
        request_rate = int(os.environ.get("VT_REQUEST_RATE", "4"))
        delay_between_requests = 60.0 / request_rate if request_rate > 0 else 15.0
        time.sleep(delay_between_requests)
    else:
        url = "https://www.virustotal.com/api/v3/files"
    headers = {"x-apikey": api_key}

    print(f"Processing {file_path}...")

    # Compute file hash first
    file_hash = compute_sha256(file_path)
    print(f"  SHA256: {file_hash}")

    try:
        # Try to upload the file
        with open(file_path, "rb") as f:
            files = {"file": (os.path.basename(file_path), f)}
            response = requests.post(url, files=files, headers=headers, timeout=120)

        if response.status_code == 200:
            # Upload successful
            data = response.json()
            analysis_id = data.get("data", {}).get("id")
            if analysis_id:
                analysis_url = f"https://www.virustotal.com/gui/file-analysis/{analysis_id}/detection"
                print(f"  ✓ Upload successful: {analysis_url}")
                return analysis_url
            else:
                print(f"  ✗ Upload succeeded but no analysis ID returned")
                return None

        elif response.status_code == 409:
            # File already exists - fetch existing report
            print(f"  ⚠ File already exists (409). Fetching existing report...")
            return fetch_existing_report(api_key, file_hash)

        elif response.status_code == 429:
            if retry_count >= MAX_RATE_LIMIT_RETRIES:
                print("  ✗ Rate limit retry exhausted while uploading file")
                return None
            # Rate limit exceeded
            print(f"  ⚠ Rate limit exceeded (429). Waiting 60 seconds...")
            time.sleep(60)
            # Retry upload
            return upload_file(api_key, file_path, retry_count + 1)

        else:
            # Other error
            print(f"  ✗ Upload failed: HTTP {response.status_code}")
            print(f"  Response: {response.text}")
            # Try fetching existing report as fallback
            return fetch_existing_report(api_key, file_hash)

    except requests.exceptions.RequestException as e:
        print(f"  ✗ Request failed: {e}")
        # Try fetching existing report as fallback
        return fetch_existing_report(api_key, file_hash)


def fetch_existing_report(api_key, file_hash, retry_count=0):
    """
    Fetch existing VirusTotal report by file hash.

    Returns:
        str: Analysis URL or None if not found
    """
    url = f"https://www.virustotal.com/api/v3/files/{file_hash}"
    headers = {"x-apikey": api_key}

    try:
        response = requests.get(url, headers=headers, timeout=30)

        if response.status_code == 200:
            # Report found
            analysis_url = f"https://www.virustotal.com/gui/file/{file_hash}/detection"
            print(f"  ✓ Existing report found: {analysis_url}")
            return analysis_url

        elif response.status_code == 404:
            # Not found in VirusTotal database
            print(f"  ✗ No existing report found (404)")
            return None

        elif response.status_code == 429:
            if retry_count >= MAX_RATE_LIMIT_RETRIES:
                print("  ✗ Rate limit retry exhausted while fetching existing report")
                return None
            # Rate limit exceeded
            print(f"  ⚠ Rate limit exceeded (429). Waiting 60 seconds...")
            time.sleep(60)
            # Retry fetch
            return fetch_existing_report(api_key, file_hash, retry_count + 1)

        else:
            print(f"  ✗ Fetch failed: HTTP {response.status_code}")
            print(f"  Response: {response.text}")
            return None

    except requests.exceptions.RequestException as e:
        print(f"  ✗ Request failed: {e}")
        return None


def main():
    if len(sys.argv) < 3:
        print("Usage: virustotal_scan.py <api_key> <file1> <file2> ...")
        sys.exit(1)

    api_key = sys.argv[1]
    files = sys.argv[2:]

    # Get rate limit from environment (default: 4 requests/min for free tier)
    request_rate = int(os.environ.get("VT_REQUEST_RATE", "4"))
    delay_between_requests = 60.0 / request_rate if request_rate > 0 else 15.0

    print(f"Scanning {len(files)} file(s) with rate limit {request_rate}/min")
    print(f"Delay between requests: {delay_between_requests:.2f} seconds\n")

    results = []
    failed = []

    for i, file_path in enumerate(files):
        # Check if file exists
        if not os.path.isfile(file_path):
            print(f"✗ File not found: {file_path}")
            failed.append(file_path)
            continue

        # Upload or fetch existing report
        analysis_url = upload_file(api_key, file_path)

        if analysis_url:
            results.append(f"{file_path}={analysis_url}")
        else:
            failed.append(file_path)

        # Rate limiting: wait between requests (except for last file)
        if i < len(files) - 1:
            print(f"\n⏳ Waiting {delay_between_requests:.2f}s before next request...\n")
            time.sleep(delay_between_requests)

    # Format output for GitHub Actions
    # Format: file1=url1,file2=url2,...
    output = ",".join(results)

    print("\n" + "="*80)
    print("SCAN COMPLETE")
    print("="*80)
    print(f"Successfully scanned: {len(results)}/{len(files)} files")

    if failed:
        print(f"\nFailed to scan {len(failed)} file(s):")
        for failed_file in failed:
            print(f"  - {failed_file}")
    print()

    # Set GitHub Actions output
    github_output = os.environ.get("GITHUB_OUTPUT")
    if github_output:
        with open(github_output, "a") as f:
            f.write(f"analysis={output}\n")
        print(f"GitHub Actions output set: analysis={output[:100]}...")
    else:
        print(f"Output: {output}")

    if failed:
        print("\n✗ ERROR: One or more release files could not be scanned")
        sys.exit(1)

    print("\n✓ SUCCESS")


if __name__ == "__main__":
    main()
