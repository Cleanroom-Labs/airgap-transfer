"""Shared fixtures for documentation E2E tests."""

import os
import signal
import socket
import subprocess
import time

import pytest

# Port for the local HTTP server (override with DOCS_TEST_PORT env var).
PORT = int(os.environ.get("DOCS_TEST_PORT", "8765"))

# Path to the built HTML output, relative to the repo root.
HTML_DIR = os.path.join(os.path.dirname(__file__), "..", "_build", "html")


def _wait_for_server(host: str, port: int, timeout: float = 10.0) -> None:
    """Block until the HTTP server is accepting connections."""
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            with socket.create_connection((host, port), timeout=1):
                return
        except OSError:
            time.sleep(0.2)
    raise RuntimeError(f"Server on {host}:{port} did not start within {timeout}s")


@pytest.fixture(scope="session")
def _docs_server():
    """Start a local HTTP server serving the built docs."""
    html_dir = os.path.abspath(HTML_DIR)
    if not os.path.isdir(html_dir):
        pytest.skip(f"Built docs not found at {html_dir} — run `make html` first")

    proc = subprocess.Popen(
        ["python3", "-m", "http.server", str(PORT), "--directory", html_dir],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    _wait_for_server("localhost", PORT)
    yield proc
    proc.send_signal(signal.SIGTERM)
    proc.wait(timeout=5)


@pytest.fixture(scope="session")
def base_url(_docs_server):
    """Base URL for all page navigations."""
    return f"http://localhost:{PORT}"
