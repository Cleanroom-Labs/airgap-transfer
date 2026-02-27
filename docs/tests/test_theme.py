"""Theme tests: branding, responsive layout, and sidebar persistence."""

import re

from playwright.sync_api import Page, expect


def test_rtd_theme_loaded(page: Page, base_url: str) -> None:
    """The Read the Docs theme's main content container is present."""
    page.goto(base_url)
    expect(page.locator(".wy-nav-content")).to_be_visible()


def test_custom_css_loaded(page: Page, base_url: str) -> None:
    """Custom CSS stylesheet is loaded."""
    page.goto(base_url)
    custom_link = page.locator("link[href*='custom.css']")
    expect(custom_link).to_have_count(1)


def test_favicon_present(page: Page, base_url: str) -> None:
    """A favicon link element exists in the page head."""
    page.goto(base_url)
    favicon = page.locator("link[rel='icon'], link[rel='shortcut icon']")
    assert favicon.count() >= 1, "No favicon link found"


def test_standalone_header_branding(page: Page, base_url: str) -> None:
    """Standalone docs show project-specific branding in the nav bar."""
    page.goto(base_url)
    brand = page.locator(".nav-brand")
    expect(brand).to_be_visible()
    expect(brand).to_contain_text("AirGap Transfer")


def test_project_icon_visible(page: Page, base_url: str) -> None:
    """The project SVG icon is rendered in the nav bar."""
    page.goto(base_url)
    logo = page.locator(".site-nav-logo svg")
    expect(logo).to_be_visible()


def test_github_edit_link(page: Page, base_url: str) -> None:
    """Content pages have a GitHub 'Edit on GitHub' link."""
    page.goto(f"{base_url}/readme.html")
    # RTD theme renders "Edit on GitHub" in the breadcrumb area
    edit_link = page.locator("a", has_text="Edit on GitHub")
    expect(edit_link).to_be_visible()


def test_version_indicator(page: Page, base_url: str) -> None:
    """The version sub-bar is visible and shows a version."""
    page.goto(base_url)
    version_bar = page.locator(".version-sub-bar")
    expect(version_bar).to_be_visible()
    version_select = page.locator("#version-select")
    expect(version_select).to_be_visible()


def test_mobile_sidebar_hidden(page: Page, base_url: str) -> None:
    """On a mobile viewport, the sidebar is not visible by default."""
    page.set_viewport_size({"width": 375, "height": 667})
    page.goto(base_url)
    sidebar = page.locator(".wy-nav-side")
    expect(sidebar).not_to_be_in_viewport()


def test_mobile_nav_toggle(page: Page, base_url: str) -> None:
    """On mobile, the RTD theme's hamburger menu toggle is visible."""
    page.set_viewport_size({"width": 375, "height": 667})
    page.goto(base_url)
    # The custom sidebar-toggle is hidden on mobile; RTD uses its own nav toggle
    hamburger = page.locator("[data-toggle='wy-nav-top']")
    expect(hamburger).to_be_visible()


def test_sidebar_persistence(page: Page, base_url: str) -> None:
    """Collapsing the sidebar persists across page navigations."""
    page.goto(base_url)
    html = page.locator("html")
    toggle = page.locator("#sidebar-toggle")

    # Collapse sidebar
    toggle.click()
    expect(html).to_have_class(re.compile(r"sidebar-collapsed"))

    # Navigate to another page
    page.goto(f"{base_url}/dashboard/proj-health.html")
    page.wait_for_load_state("domcontentloaded")

    # Sidebar should still be collapsed (localStorage persistence)
    expect(page.locator("html")).to_have_class(re.compile(r"sidebar-collapsed"))
