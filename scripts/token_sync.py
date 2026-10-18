#!/usr/bin/env python3
"""Synchronize local Ash token checkouts using a committed common baseline.

This engine is identical in ash-site/scripts and ash-doc/rendering.
It has no network or third-party dependencies.
"""
import argparse
from contextlib import ExitStack
import difflib
import fcntl
import hashlib
import os
from pathlib import Path
import subprocess
import sys
import tempfile

BASE = '.design-tokens-base.css'


def kind(root):
    if (root / 'public/tokens.css').is_file():
        return 'site', root / 'public/tokens.css'
    if (root / 'rendering/styles/html.scss').is_file():
        return 'doc', root / 'rendering/styles/ash-tokens.css'
    raise ValueError(f'{root}: not an Ash token checkout')


def discover(root, role):
    result = subprocess.run(['git', '-C', str(root), 'rev-parse',
                             '--path-format=absolute', '--git-common-dir'],
                            text=True, capture_output=True)
    primary = Path(result.stdout.strip()).parent if result.returncode == 0 else root
    return primary.parent / ('ash-doc' if role == 'site' else 'ash-site')


def atomic_write(path, content):
    with tempfile.NamedTemporaryFile(dir=path.parent, delete=False) as temporary:
        temporary.write(content)
        name = Path(temporary.name)
    try:
        name.chmod(path.stat().st_mode & 0o777)
        name.replace(path)
    finally:
        name.unlink(missing_ok=True)


def synchronize(root, peer=None, check=False):
    root = Path(root).resolve()
    role, local = kind(root)
    # Validate the local contract even when the peer is absent.
    local.read_bytes()
    (root / BASE).read_bytes()
    if os.environ.get('ASH_TOKENS_STANDALONE') == '1':
        print('Design tokens: standalone mode; using local tokens')
        return
    configured = peer or os.environ.get('ASH_DOC_CHECKOUT' if role == 'site' else 'ASH_SITE_CHECKOUT')
    other = Path(configured).expanduser().resolve() if configured else discover(root, role).resolve()
    if not other.exists() and not configured:
        print('Design tokens: standalone build; sibling checkout absent')
        return
    try:
        peer_role, remote = kind(other)
        if peer_role == role:
            raise ValueError('expected the other Ash repository')
        remote.read_bytes()
        (other / BASE).read_bytes()
    except (OSError, ValueError) as error:
        raise ValueError(f'Invalid token peer {other}: {error}') from error

    # Lock each checkout rather than each pair, so two worktrees cannot race
    # while updating a shared peer. Locks live outside the repositories.
    with ExitStack() as stack:
        for path in sorted({root, other}):
            key = hashlib.sha256(os.fsencode(path)).hexdigest()
            lock = stack.enter_context(open(Path(tempfile.gettempdir()) / f'ash-tokens-{key}.lock', 'a'))
            fcntl.flock(lock, fcntl.LOCK_EX)
        ours, theirs = local.read_bytes(), remote.read_bytes()
        ours_base, theirs_base = (root / BASE).read_bytes(), (other / BASE).read_bytes()
        if check:
            if not (ours == theirs == ours_base == theirs_base):
                raise ValueError('Design token drift: run the synchronization command (check made no changes)')
            print('Design tokens and baselines match')
            return
        if ours == theirs:
            selected = ours
        elif ours_base != theirs_base:
            raise ValueError('Design token baseline conflict: reconcile token files explicitly; no files changed')
        elif ours == ours_base:
            selected = theirs
        elif theirs == theirs_base:
            selected = ours
        else:
            diff = ''.join(difflib.unified_diff(ours.decode().splitlines(True), theirs.decode().splitlines(True),
                                               fromfile=str(local), tofile=str(remote)))
            raise ValueError('Design token conflict: both files changed; no files changed.\n' + diff)
        for path in [local, remote, root / BASE, other / BASE]:
            if path.read_bytes() != selected:
                atomic_write(path, selected)
        print('Design tokens synchronized locally')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument('--peer', type=Path)
    parser.add_argument('--check', action='store_true', help='report drift without changing files')
    args = parser.parse_args()
    try:
        synchronize(args.root, args.peer, args.check)
    except (OSError, ValueError) as error:
        print(error, file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
