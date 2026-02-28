"""Content smoke tests: verify project-specific pages load.

Theme-generic smoke tests (console errors, basic page load) are in the
common submodule's test suite.  This file tests only AirGap-specific pages.
"""

import re

import pytest
from playwright.sync_api import Page, expect

DASHBOARD_PAGES = [
    "dashboard/proj-health.html",
    "dashboard/usecase-trace.html",
    "dashboard/req-coverage.html",
    "dashboard/impl-status.html",
    "dashboard/test-status.html",
    "dashboard/coverage-gaps.html",
]

SPEC_PAGES = [
    "requirements/srs.html",
    "design/sdd.html",
    "testing/plan.html",
]

OTHER_PAGES = [
    "roadmap.html",
    "readme.html",
]


def test_index_loads(page: Page, base_url: str) -> None:
    """Index page loads and has the expected title."""
    page.goto(base_url)
    expect(page).to_have_title(re.compile(r"AirGap Transfer"))


@pytest.mark.parametrize("path", DASHBOARD_PAGES, ids=lambda p: p.split("/")[-1])
def test_dashboard_page_loads(page: Page, base_url: str, path: str) -> None:
    """Each dashboard page returns 200 and has content."""
    resp = page.goto(f"{base_url}/{path}")
    assert resp is not None and resp.ok
    expect(page.locator(".wy-nav-content")).to_be_visible()


@pytest.mark.parametrize("path", SPEC_PAGES, ids=lambda p: p.split("/")[-1])
def test_spec_page_loads(page: Page, base_url: str, path: str) -> None:
    """Each specification page returns 200 and has content."""
    resp = page.goto(f"{base_url}/{path}")
    assert resp is not None and resp.ok
    expect(page.locator(".wy-nav-content")).to_be_visible()


@pytest.mark.parametrize("path", OTHER_PAGES, ids=lambda p: p.split("/")[-1])
def test_other_page_loads(page: Page, base_url: str, path: str) -> None:
    """Roadmap, README, etc. return 200 and have content."""
    resp = page.goto(f"{base_url}/{path}")
    assert resp is not None and resp.ok
    expect(page.locator(".wy-nav-content")).to_be_visible()


def test_use_cases_section_loads(page: Page, base_url: str) -> None:
    """At least one use-case page is reachable."""
    resp = page.goto(f"{base_url}/use-cases/")
    assert resp is not None and resp.ok


def test_implementation_section_loads(page: Page, base_url: str) -> None:
    """Implementation section index page loads."""
    resp = page.goto(f"{base_url}/implementation/")
    assert resp is not None and resp.ok
