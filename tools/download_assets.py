#!/usr/bin/env python3
"""Download recommended Destiny Forge asset packs."""

from __future__ import annotations

import json
import re
import shutil
import sys
import time
import urllib.parse
import urllib.request
import zipfile
from http.cookiejar import CookieJar
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "assets" / "source"
ASSETS = ROOT / "assets"

# (page_url, preferred filename substring, destination dir under assets/source/)
PACKS = [
    (
        "https://anokolisa.itch.io/free-pixel-art-asset-pack-topdown-tileset-rpg-16x16-sprites",
        "Pixel Crawler",
        SOURCE / "pixel_crawler",
    ),
    (
        "https://0x72.itch.io/dungeontileset-ii",
        "0x72_DungeonTilesetII",
        SOURCE / "0x72",
    ),
    (
        "https://stealthix.itch.io/animated-slimes",
        "Slimes.zip",
        SOURCE / "slimes",
    ),
]


def build_opener() -> urllib.request.OpenerDirector:
    return urllib.request.build_opener(urllib.request.HTTPCookieProcessor(CookieJar()))


def fetch(
    opener: urllib.request.OpenerDirector,
    url: str,
    data: bytes | None = None,
    *,
    method: str = "GET",
    headers: dict[str, str] | None = None,
    referer: str | None = None,
) -> bytes:
    hdrs = dict(headers or {})
    if referer:
        hdrs.setdefault("Referer", referer)
    req = urllib.request.Request(url, data=data, method=method, headers=hdrs)
    with opener.open(req, timeout=120) as resp:
        return resp.read()


def csrf_from_html(html: str) -> str:
    match = re.search(r'name="csrf_token"\s+value="([^"]+)"', html)
    if not match:
        raise RuntimeError("csrf_token not found")
    return match.group(1)


def upload_entries(download_page_html: str) -> list[tuple[str, str]]:
    pattern = re.compile(
        r'data-upload_id="(\d+)".*?<strong[^>]*title="([^"]+)"',
        re.S,
    )
    return [(upload_id, title) for upload_id, title in pattern.findall(download_page_html)]


def download_token_page(opener: urllib.request.OpenerDirector, page_url: str, csrf: str) -> str:
    headers = {
        "Content-Type": "application/x-www-form-urlencoded",
        "X-CSRF-Token": csrf,
        "Referer": page_url,
        "Accept": "application/json",
        "User-Agent": "DestinyForge-AssetSetup/1.0",
    }
    token_response = fetch(
        opener,
        f"{page_url}/download_url",
        urllib.parse.urlencode({"csrf_token": csrf}).encode(),
        method="POST",
        headers=headers,
    )
    payload = json.loads(token_response.decode())
    return fetch(opener, payload["url"], referer=page_url).decode("utf-8", errors="replace")


def download_upload(
    opener: urllib.request.OpenerDirector,
    page_url: str,
    csrf: str,
    upload_id: str,
) -> bytes:
    query = urllib.parse.urlencode({"source": "game_download"})
    api = f"{page_url}/file/{upload_id}?{query}"
    headers = {
        "Content-Type": "application/x-www-form-urlencoded",
        "X-CSRF-Token": csrf,
        "Referer": page_url,
        "Accept": "application/json",
        "User-Agent": "DestinyForge-AssetSetup/1.0",
    }
    response = json.loads(
        fetch(
            opener,
            api,
            urllib.parse.urlencode({"csrf_token": csrf}).encode(),
            method="POST",
            headers=headers,
        ).decode()
    )
    if "errors" in response:
        raise RuntimeError(response["errors"])
    return fetch(opener, response["url"], referer=page_url)


def pick_upload(entries: list[tuple[str, str]], preferred: str) -> tuple[str, str]:
    preferred_lower = preferred.lower()
    for upload_id, title in entries:
        if preferred_lower in title.lower():
            return upload_id, title
    if len(entries) == 1:
        return entries[0]
    raise RuntimeError(f"No upload matching '{preferred}' in {[t for _, t in entries]}")


def extract_archive(archive: Path, destination: Path) -> None:
    destination.mkdir(parents=True, exist_ok=True)
    if zipfile.is_zipfile(archive):
        with zipfile.ZipFile(archive) as zf:
            zf.extractall(destination)
        return
    shutil.copy2(archive, destination / archive.name)


def download_pack(
    opener: urllib.request.OpenerDirector,
    page_url: str,
    preferred_name: str,
    destination: Path,
) -> Path:
    html = fetch(opener, page_url).decode("utf-8", errors="replace")
    csrf = csrf_from_html(html)
    download_page = download_token_page(opener, page_url, csrf)
    upload_id, title = pick_upload(upload_entries(download_page), preferred_name)

    SOURCE.mkdir(parents=True, exist_ok=True)
    archive = SOURCE / re.sub(r"[^\w.\-]+", "_", title)
    if not archive.suffix:
        archive = archive.with_suffix(".bin")

    print(f"  file: {title} (upload {upload_id})")
    if not archive.exists() or archive.stat().st_size < 10_000:
        archive.write_bytes(download_upload(opener, page_url, csrf, upload_id))
        print(f"  saved archive: {archive.name} ({archive.stat().st_size:,} bytes)")
    else:
        print(f"  reusing archive: {archive.name}")

    if destination.exists():
        shutil.rmtree(destination)
    extract_archive(archive, destination)
    print(f"  extracted -> {destination.relative_to(ROOT)}")
    return archive


def main() -> int:
    opener = build_opener()
    for index, (url, preferred, dest) in enumerate(PACKS):
        if index:
            time.sleep(2)
        print(f"downloading {dest.name} ...")
        try:
            download_pack(opener, url, preferred, dest)
        except Exception as exc:  # noqa: BLE001
            print(f"FAILED {dest.name}: {exc}", file=sys.stderr)
            return 1
    print("done")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())