#!/usr/bin/env python3
"""Build a per-user, disk-selectable Windows installer. No Minecraft files included."""
import argparse
import base64
import hashlib
import json
from pathlib import Path, PurePosixPath
import shutil
import struct
import subprocess
import zipfile

from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey
from cryptography.exceptions import InvalidSignature

ROOT = Path(__file__).resolve().parents[1]
TAG = 'v0.2.98'
GAME_TAG = 'nightly-2026-10-30'
LAUNCHER_TAG = 'launcher@0.2.28'


def digest(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def verify_archive(archive, sums_file, signature_file, signed_for, asset):
    """Same Ed25519 message as horizon-release-sign, never an unsigned fallback."""
    sums = Path(sums_file).read_bytes()
    signatures = [base64.b64decode(s, validate=True) for s in Path(signature_file).read_text().splitlines() if s.strip()]
    keys = [base64.b64decode(s, validate=True) for s in
            (ROOT / 'crates/horizon-release-sign/release-keys.txt').read_text().splitlines()
            if s.strip() and not s.startswith('#')]
    message = f'horizon release {signed_for}\n'.encode() + sums
    valid = False
    for key in keys:
        for signature in signatures:
            try:
                Ed25519PublicKey.from_public_bytes(key).verify(signature, message)
                valid = True
            except InvalidSignature:
                pass
    if not valid:
        raise ValueError('Release signature is not trusted')
    entries = {line.split()[1].lstrip('*'): line.split()[0] for line in sums.decode().splitlines() if line.strip()}
    if entries.get(asset) != digest(archive):
        raise ValueError(f'{asset} does not match its signed checksum')


def extract_archive(archive, destination):
    with zipfile.ZipFile(archive) as zipped:
        if sum(i.file_size for i in zipped.infolist()) > 512 * 1024 * 1024:
            raise ValueError('Unexpected archive size')
        for item in zipped.infolist():
            path = PurePosixPath(item.filename.replace('\\', '/'))
            if path.is_absolute() or '..' in path.parts or any(':' in p for p in path.parts):
                raise ValueError('Unsafe archive path')
            if (item.external_attr >> 16) & 0o170000 == 0o120000:
                raise ValueError('Archive symlinks are not allowed')
            if path.suffix.lower() == '.jar' or '.local' in path.parts:
                raise ValueError('Game files must not be packaged')
        zipped.extractall(destination)


def require_windows_x64(path, min_stack=0):
    with Path(path).open('rb') as file:
        if file.read(2) != b'MZ':
            raise ValueError(f'{path} is not an executable')
        file.seek(0x3c)
        offset, = struct.unpack('<I', file.read(4))
        file.seek(offset)
        if file.read(4) != b'PE\0\0' or file.read(2) != b'\x64\x86':
            raise ValueError(f'{path} is not a Windows x64 PE executable')
        if min_stack:
            file.seek(offset + 24)
            if file.read(2) != b'\x0b\x02':
                raise ValueError(f'{path} does not have a PE32+ optional header')
            file.seek(offset + 24 + 72)
            reserve, = struct.unpack('<Q', file.read(8))
            if reserve < min_stack:
                raise ValueError(f'{path} main-thread stack reserve is too small ({reserve} bytes)')


def package(args):
    verify_archive(args.game, args.game_sums, args.game_signature, 'nightly', 'horizon-windows-x64.zip')
    verify_archive(args.launcher, args.launcher_sums, args.launcher_signature, LAUNCHER_TAG, 'horizon-launcher-windows-x64.zip')
    for binary in ['horizon-client.exe', 'horizon-ctl.exe', 'horizon-light-profile.exe']:
        require_windows_x64(args.engine_dir / binary,
                            min_stack=16 * 1024 * 1024 if binary == 'horizon-client.exe' else 0)
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    work = out / 'build'
    if work.exists():
        shutil.rmtree(work)
    work.mkdir()
    extract_archive(args.game, work / 'official-game')
    extract_archive(args.launcher, work / 'official-launcher')
    payload = work / 'payload'
    version = payload / 'data' / 'launcher' / 'versions' / TAG
    shutil.copytree(work / 'official-game' / 'horizon-windows-x64', version)
    for name in ['horizon-client.exe', 'horizon-ctl.exe']:
        shutil.copy2(args.engine_dir / name, version / name)
    # Completion is written by the installer only after all extraction succeeds.
    (version / '.installed').unlink(missing_ok=True)
    # Original Horizon-format packs only; no Minecraft textures or external shader code.
    for source, name in [('optimized_light', 'Horizon-Luz-Rapida'),
                         ('optimized_glow', 'Horizon-Leve-Bonito'),
                         ('golden_hour', 'Golden-Hour')]:
        shutil.copytree(ROOT / 'crates/horizon-shader-api/packs' / source,
                        payload / 'data/game/shaderpacks' / name)
    runtime = payload / 'runtime'
    runtime.mkdir()
    launcher = work / 'official-launcher' / 'horizon-launcher.exe'
    require_windows_x64(launcher)
    shutil.copy2(launcher, runtime / launcher.name)
    shutil.copy2(args.engine_dir / 'horizon-light-profile.exe', runtime / 'horizon-light-profile.exe')
    settings = work / 'settings.json'
    settings.write_text(json.dumps({'selected': TAG, 'install_new_versions': False, 'prune_old_versions': False}, indent=2))
    for name in ['LICENSE', 'THIRD_PARTY_NOTICES.md']:
        shutil.copy2(ROOT / name, payload / name)
    readme = ROOT / 'packaging/windows-optimized/LEIA-ME.txt'
    shutil.copy2(readme, payload / readme.name)
    manifest = {
        'engine_source': args.source_commit,
        'game_support_release': GAME_TAG,
        'launcher_release': LAUNCHER_TAG,
        'launcher_unmodified': True,
        'upstream_game_archive_sha256': digest(args.game),
        'upstream_launcher_archive_sha256': digest(args.launcher),
        'files': {str(p.relative_to(payload)).replace('\\', '/'): digest(p)
                  for p in sorted(payload.rglob('*')) if p.is_file()},
    }
    (payload / 'BUILD-INFO.json').write_text(json.dumps(manifest, indent=2) + '\n')
    scripts = ROOT / 'packaging/windows-optimized'
    full = payload / 'Horizon-Completo.exe'
    subprocess.run([str(args.makensis), '-V2', f'-DPROJECT_DIR={ROOT}', f'-DOUTPUT_FILE={full}', str(scripts / 'launcher.nsi')], check=True)
    bootstrap = payload / 'Horizon-Optimized.exe'
    subprocess.run([str(args.makensis), '-V2', f'-DPROJECT_DIR={ROOT}', f'-DOUTPUT_FILE={bootstrap}', str(scripts / 'light-launcher.nsi')], check=True)
    installer = out / 'Horizon-Optimized-Setup.exe'
    subprocess.run([str(args.makensis), '-V2', f'-DPROJECT_DIR={ROOT}', f'-DOUTPUT_FILE={installer}',
                    f'-DPAYLOAD_DIR={payload}', f'-DDEFAULT_SETTINGS={settings}', str(scripts / 'setup.nsi')], check=True)
    (out / 'SHA256SUMS.txt').write_text(f'{digest(installer)}  {installer.name}\n')
    print(installer)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['game', 'game-sums', 'game-signature', 'launcher', 'launcher-sums', 'launcher-signature', 'engine-dir', 'makensis', 'out']:
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--source-commit', required=True)
    package(parser.parse_args())


if __name__ == '__main__':
    main()
