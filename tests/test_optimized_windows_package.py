"""Installer inputs are authenticated, x64 executables and safe archive paths."""
import importlib.util
from pathlib import Path
import struct
import tempfile
import unittest
import zipfile

if importlib.util.find_spec('cryptography') is None:
    raise unittest.SkipTest('Windows packager needs the optional cryptography package')

from tools.package_optimized_windows import extract_archive, require_windows_x64, verify_archive


class WindowsPackageTests(unittest.TestCase):
    def test_rejects_traversal_symlinks_and_game_jars(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name in ['../escape', '/absolute', 'C:/escape', 'a/../../escape', 'server.jar', '.local/client.jar']:
                archive = root / 'bad.zip'
                with zipfile.ZipFile(archive, 'w') as z:
                    z.writestr(name, b'bad')
                with self.assertRaises(ValueError):
                    extract_archive(archive, root / 'unpacked')
            with zipfile.ZipFile(root / 'link.zip', 'w') as z:
                info = zipfile.ZipInfo('link')
                info.external_attr = 0o120777 << 16
                z.writestr(info, '../escape')
            with self.assertRaises(ValueError):
                extract_archive(root / 'link.zip', root / 'unpacked')
            self.assertFalse((root / 'unpacked').exists())

    def test_extracts_original_exporters_without_changing_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            name = 'exporters/client/dev/horizonlab/parity/Example.class'
            with zipfile.ZipFile(root / 'ok.zip', 'w') as z:
                z.writestr(name, b'original exporter')
            extract_archive(root / 'ok.zip', root / 'out')
            self.assertEqual((root / 'out' / name).read_bytes(), b'original exporter')

    def test_requires_x64_pe_not_linux_or_x86(self):
        with tempfile.TemporaryDirectory() as directory:
            file = Path(directory) / 'game.exe'
            header = bytearray(128)
            header[:2] = b'MZ'
            struct.pack_into('<I', header, 0x3c, 64)
            header[64:70] = b'PE\0\0\x64\x86'
            file.write_bytes(header)
            require_windows_x64(file)
            header[68:70] = b'\x4c\x01'
            file.write_bytes(header)
            with self.assertRaises(ValueError):
                require_windows_x64(file)
            file.write_bytes(b'\x7fELF' + b'\0' * 124)
            with self.assertRaises(ValueError):
                require_windows_x64(file)

    def test_signature_is_mandatory_before_archive_extraction(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'archive').write_bytes(b'anything')
            (root / 'sums').write_text('0' * 64 + '  asset.zip\n')
            (root / 'signature').write_text('')
            with self.assertRaisesRegex(ValueError, 'signature'):
                verify_archive(root / 'archive', root / 'sums', root / 'signature', 'nightly', 'asset.zip')

    def test_rejects_old_one_megabyte_client_stack(self):
        with tempfile.TemporaryDirectory() as directory:
            file = Path(directory) / 'client.exe'
            header = bytearray(256)
            header[:2] = b'MZ'
            struct.pack_into('<I', header, 0x3c, 64)
            header[64:70] = b'PE\0\0\x64\x86'
            struct.pack_into('<H', header, 64 + 24, 0x20b)
            struct.pack_into('<Q', header, 64 + 24 + 72, 1024 * 1024)
            file.write_bytes(header)
            with self.assertRaisesRegex(ValueError, 'stack reserve'):
                require_windows_x64(file, min_stack=16 * 1024 * 1024)
            struct.pack_into('<Q', header, 64 + 24 + 72, 16 * 1024 * 1024)
            file.write_bytes(header)
            require_windows_x64(file, min_stack=16 * 1024 * 1024)


if __name__ == '__main__':
    unittest.main()
